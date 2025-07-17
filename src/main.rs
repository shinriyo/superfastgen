use clap::{Parser, Subcommand, ValueEnum};

mod commands;
mod utils;

use commands::{generate, assets, provider_gen};
use utils::{parser, yaml};

use notify::{Watcher, RecursiveMode, RecommendedWatcher, Event, EventKind, Config};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::path::Path;
use log::info;

// Constants for default paths (compatible with Dart build_runner)
const DEFAULT_LIB_DIR: &str = "lib";
const DEFAULT_OUTPUT_DIR: &str = "generated";
const DEFAULT_ASSETS_DIR: &str = "assets";
const DEFAULT_PUBSPEC_FILE: &str = "pubspec.yaml";

// Computed constants
const DEFAULT_OUTPUT_PATH: &str = "lib/generated";

#[derive(Parser, Debug, Clone)]
#[command(name = "SuperFastGen")]
#[command(about = "Blazing fast codegen for Dart/Flutter", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    /// Output directory for generated files
    #[arg(long, short, default_value = DEFAULT_OUTPUT_PATH)]
    output: String,
    /// Assets directory
    #[arg(long, default_value = DEFAULT_ASSETS_DIR)]
    assets: String,
    /// Watch mode for file changes
    #[arg(long)]
    watch: bool,
    /// Delete conflicting outputs before generation
    #[arg(long)]
    delete_conflicting_outputs: bool,
    /// Build filter for specific files (like Dart build_runner)
    #[arg(long)]
    build_filter: Option<String>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Generate code for a specific type
    Generate {
        /// Type of code to generate
        #[arg(long, value_enum, default_value = "all")]
        r#type: GenType,
        /// Output directory for generated files (overrides global --output)
        #[arg(long)]
        output: Option<String>,
        /// Delete conflicting outputs before generation
        #[arg(long)]
        delete_conflicting_outputs: bool,
    },
    /// Generate only assets
    Assets {
        /// Assets directory (overrides global --assets)
        #[arg(long)]
        assets: Option<String>,
        /// Output directory for generated files (overrides global --output)
        #[arg(long)]
        output: Option<String>,
    },
    /// Generate everything (code and assets)
    All {
        /// Output directory for generated files (overrides global --output)
        #[arg(long)]
        output: Option<String>,
        /// Assets directory (overrides global --assets)
        #[arg(long)]
        assets: Option<String>,
    },
    /// Clean generated files
    Clean {
        /// Output directory to clean (overrides global --output)
        #[arg(long)]
        output: Option<String>,
    },
}

#[derive(ValueEnum, Debug, Clone)]
enum GenType {
    Freezed,
    Json,
    Riverpod,
    Provider,
    All,
}

#[derive(Debug, Clone)]
struct EffectiveConfig {
    output: String,
    assets: String,
    watch: bool,
    delete_conflicting_outputs: bool,
    build_filter: Option<String>,
}

fn main() {
    env_logger::init();
    info!("SuperFastGen - Code Generator");
    let cli = Cli::parse();
    let yaml_config = yaml::parse_superfastgen_yaml("superfastgen.yaml");
    let effective = merge_config(&cli, yaml_config);

    match &cli.command {
        Some(Commands::Generate { r#type, output, delete_conflicting_outputs }) => {
            let effective_output = output.as_ref().cloned().unwrap_or(effective.output.clone());
            let effective_delete_conflicting = *delete_conflicting_outputs || effective.delete_conflicting_outputs;

            let input_path = if let Some(ref filter) = effective.build_filter {
                let path = std::path::Path::new(filter);
                if let Some(parent) = path.parent() {
                    parent.to_string_lossy().to_string()
                } else {
                    DEFAULT_LIB_DIR.to_string()
                }
            } else {
                DEFAULT_LIB_DIR.to_string()
            };
            
            match r#type {
                GenType::Freezed => generate::generate_freezed_with_paths_and_clean(&input_path, &effective_output, effective_delete_conflicting),
                GenType::Json => generate::generate_json_with_paths_and_clean(&input_path, &effective_output, effective_delete_conflicting),
                GenType::Riverpod => generate::generate_riverpod_with_paths_and_clean(&input_path, &effective_output, effective_delete_conflicting),
                GenType::Provider => generate::generate_provider_with_paths_and_clean(&input_path, &effective_output, effective_delete_conflicting),
                GenType::All => {
                    generate::generate_freezed_with_paths_and_clean(&input_path, &effective_output, effective_delete_conflicting);
                    generate::generate_json_with_paths_and_clean(&input_path, &effective_output, effective_delete_conflicting);
                    generate::generate_riverpod_with_paths_and_clean(&input_path, &effective_output, effective_delete_conflicting);
                    generate::generate_provider_with_paths_and_clean(&input_path, &effective_output, effective_delete_conflicting);
                },
            }
        }
        Some(Commands::Assets { assets, output }) => {
            let effective_assets = assets.as_ref().cloned().unwrap_or(effective.assets.clone());
            let effective_output = output.as_ref().cloned().unwrap_or(effective.output.clone());
            assets::generate_assets_with_paths(&effective_assets, &effective_output);
        }
        Some(Commands::All { output, assets }) => {
            let effective_output = output.as_ref().cloned().unwrap_or(effective.output.clone());
            let effective_assets = assets.as_ref().cloned().unwrap_or(effective.assets.clone());
            run_generators(&EffectiveConfig {
                output: effective_output,
                assets: effective_assets,
                watch: effective.watch,
                delete_conflicting_outputs: effective.delete_conflicting_outputs,
                build_filter: effective.build_filter.clone(),
            });
        }
        Some(Commands::Clean { output }) => {
            let effective_output = output.as_ref().cloned().unwrap_or(effective.output.clone());
            clean_generated_files(&EffectiveConfig {
                output: effective_output,
                assets: effective.assets.clone(),
                watch: effective.watch,
                delete_conflicting_outputs: effective.delete_conflicting_outputs,
                build_filter: effective.build_filter.clone(),
            });
        }
        None => {
            // If --watch is specified, run in watch mode
            if effective.watch {
                watch_mode(&effective);
            } else {
                run_generators(&effective);
            }
        }
    }
}

fn merge_config(cli: &Cli, yaml_config: Option<yaml::SuperfastgenConfig>) -> EffectiveConfig {
    let (yaml_gen, yaml_assets) = if let Some(cfg) = yaml_config {
        (cfg.generate.unwrap_or_default(), cfg.assets.unwrap_or_default())
    } else {
        (yaml::GenerateConfig::default(), yaml::AssetsConfig::default())
    };
    
    // Use configuration fields to determine behavior
    let _freezed_enabled = yaml_gen.freezed.unwrap_or(true);
    let _json_enabled = yaml_gen.json.unwrap_or(true);
    let _riverpod_enabled = yaml_gen.riverpod.unwrap_or(true);
    let _images_enabled = yaml_assets.include_images.unwrap_or(true);
    let _fonts_enabled = yaml_assets.include_fonts.unwrap_or(true);
    let _icons_enabled = yaml_assets.include_icons.unwrap_or(true);
    
    EffectiveConfig {
        // Prioritize CLI arguments if they differ from defaults
        output: if cli.output != DEFAULT_OUTPUT_PATH {
            cli.output.clone()
        } else {
            yaml_gen.output.unwrap_or(DEFAULT_OUTPUT_PATH.to_string())
        },
        build_filter: cli.build_filter.clone(),
        assets: if cli.assets != DEFAULT_ASSETS_DIR {
            cli.assets.clone()
        } else {
            yaml_assets.input.unwrap_or(DEFAULT_ASSETS_DIR.to_string())
        },
        // Watch mode controlled only by CLI --watch flag
        watch: cli.watch,
        // Delete conflicting outputs flag
        delete_conflicting_outputs: cli.delete_conflicting_outputs,
    }
}

/// Run all code and asset generators
fn run_generators(cfg: &EffectiveConfig) {
    let yaml_config = yaml::parse_superfastgen_yaml("superfastgen.yaml");
    let (yaml_gen, yaml_assets) = if let Some(config) = yaml_config {
        (config.generate.unwrap_or_default(), config.assets.unwrap_or_default())
    } else {
        (yaml::GenerateConfig::default(), yaml::AssetsConfig::default())
    };
    
    // Use build_filter if specified, otherwise use default lib directory
    let input_path = if let Some(ref filter) = cfg.build_filter {
        // Extract directory from build_filter path
        let path = std::path::Path::new(filter);
        if let Some(parent) = path.parent() {
            parent.to_string_lossy().to_string()
        } else {
            DEFAULT_LIB_DIR.to_string()
        }
    } else {
        DEFAULT_LIB_DIR.to_string()
    };
    
    // Generate code based on configuration
    if yaml_gen.freezed.unwrap_or(true) {
        generate::generate_freezed_with_paths_and_clean(&input_path, &cfg.output, cfg.delete_conflicting_outputs);
    }
    
    if yaml_gen.json.unwrap_or(true) {
        generate::generate_json_with_paths_and_clean(&input_path, &cfg.output, cfg.delete_conflicting_outputs);
    }
    
    if yaml_gen.riverpod.unwrap_or(true) {
        generate::generate_riverpod_with_paths_and_clean(&input_path, &cfg.output, cfg.delete_conflicting_outputs);
    }
    
    if yaml_gen.provider.unwrap_or(true) {
        generate::generate_provider_with_paths_and_clean(&input_path, &cfg.output, cfg.delete_conflicting_outputs);
    }
    
    // Use configuration for assets
    if yaml_assets.include_images.unwrap_or(true) || 
       yaml_assets.include_fonts.unwrap_or(true) || 
       yaml_assets.include_icons.unwrap_or(true) {
        let assets_output = yaml_assets.output.unwrap_or(cfg.output.clone());
        assets::generate_assets_with_paths(&cfg.assets, &assets_output);
    }
    
    let _ = parser::parse_code("example code");
    let _ = yaml::parse_pubspec_yaml("example yaml");
}

/// Watch for file changes and rerun generators
fn watch_mode(cfg: &EffectiveConfig) {
    let input_path = if let Some(ref filter) = cfg.build_filter {
        let path = std::path::Path::new(filter);
        if let Some(parent) = path.parent() {
            parent.to_string_lossy().to_string()
        } else {
            DEFAULT_LIB_DIR.to_string()
        }
    } else {
        DEFAULT_LIB_DIR.to_string()
    };
    
    println!("Watching for changes in {} and pubspec.yaml...", input_path);
    let (tx, rx) = channel();
    let config = Config::default().with_poll_interval(Duration::from_secs(1));
    let mut watcher: RecommendedWatcher = Watcher::new(tx, config).unwrap();
    watcher.watch(Path::new(&input_path), RecursiveMode::Recursive).unwrap();
    watcher.watch(Path::new("pubspec.yaml"), RecursiveMode::NonRecursive).unwrap();

    run_generators(cfg);

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if let Event { kind: EventKind::Modify(_), .. } | Event { kind: EventKind::Create(_), .. } | Event { kind: EventKind::Remove(_), .. } = event {
                    println!("Change detected! Regenerating...");
                    run_generators(cfg);
                }
            }
            Ok(Err(e)) => println!("watch error: {:?}", e),
            Err(e) => println!("channel error: {:?}", e),
        }
    }
}

/// Clean generated files
fn clean_generated_files(cfg: &EffectiveConfig) {
    use std::fs;
    use walkdir::WalkDir;
    
    let input_path = if let Some(ref filter) = cfg.build_filter {
        let path = std::path::Path::new(filter);
        if let Some(parent) = path.parent() {
            parent.to_string_lossy().to_string()
        } else {
            DEFAULT_LIB_DIR.to_string()
        }
    } else {
        DEFAULT_LIB_DIR.to_string()
    };
    
    println!("Cleaning generated files in {}...", input_path);
    
    let mut cleaned_count = 0;
    
    // Walk through the input directory and find generated files
    for entry in WalkDir::new(&input_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Some(file_name) = path.file_name() {
                let file_name_str = file_name.to_string_lossy();
                
                // Check if it's a generated file
                if file_name_str.ends_with(".g.dart") || 
                   file_name_str.ends_with(".freezed.dart") ||
                   file_name_str.ends_with(".config.dart") {
                    
                    match fs::remove_file(path) {
                        Ok(_) => {
                            println!("Removed: {}", path.display());
                            cleaned_count += 1;
                        }
                        Err(e) => {
                            eprintln!("Failed to remove {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }
    }
    
    println!("Cleaned {} generated files", cleaned_count);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_run_generators() {
        let cfg = EffectiveConfig {
            output: DEFAULT_OUTPUT_PATH.to_string(),
            assets: DEFAULT_ASSETS_DIR.to_string(),
            watch: false,
            delete_conflicting_outputs: false,
            build_filter: None,
        };
        run_generators(&cfg);
    }
}
