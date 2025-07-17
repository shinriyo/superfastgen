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

#[derive(Parser, Debug, Clone)]
#[command(name = "SuperFastGen")]
#[command(about = "Blazing fast codegen for Dart/Flutter", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    /// Input directory for Dart files
    #[arg(long, default_value = "test_flutter_app/aminomi/lib")]
    input: String,
    /// Output directory for generated files
    #[arg(long, default_value = "test_flutter_app/aminomi/lib/gen")]
    output: String,
    /// Assets directory
    #[arg(long, default_value = "assets")]
    assets: String,
    /// Watch mode for file changes
    #[arg(long)]
    watch: bool,
    /// Delete conflicting outputs before generation
    #[arg(long)]
    delete_conflicting_outputs: bool,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Generate code for a specific type
    Generate {
        /// Type of code to generate
        #[arg(long, value_enum, default_value = "all")]
        r#type: GenType,
        /// Input directory for Dart files (overrides global --input)
        #[arg(long)]
        input: Option<String>,
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
        /// Input directory for Dart files (overrides global --input)
        #[arg(long)]
        input: Option<String>,
        /// Output directory for generated files (overrides global --output)
        #[arg(long)]
        output: Option<String>,
        /// Assets directory (overrides global --assets)
        #[arg(long)]
        assets: Option<String>,
    },
    /// Clean generated files
    Clean {
        /// Input directory to clean (overrides global --input)
        #[arg(long)]
        input: Option<String>,
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
    input: String,
    output: String,
    assets: String,
    watch: bool,
    delete_conflicting_outputs: bool,
}

fn main() {
    env_logger::init();
    info!("SuperFastGen - Code Generator");
    let cli = Cli::parse();
    let yaml_config = yaml::parse_superfastgen_yaml("superfastgen.yaml");
    let effective = merge_config(&cli, yaml_config);

    match &cli.command {
        Some(Commands::Generate { r#type, input, output, delete_conflicting_outputs }) => {
            let effective_input = input.as_ref().cloned().unwrap_or(effective.input.clone());
            let effective_output = output.as_ref().cloned().unwrap_or(effective.output.clone());
            let effective_delete_conflicting = *delete_conflicting_outputs || effective.delete_conflicting_outputs;

            match r#type {
                GenType::Freezed => generate::generate_freezed_with_paths_and_clean(&effective_input, &effective_output, effective_delete_conflicting),
                GenType::Json => generate::generate_json_with_paths_and_clean(&effective_input, &effective_output, effective_delete_conflicting),
                GenType::Riverpod => generate::generate_riverpod_with_paths_and_clean(&effective_input, &effective_output, effective_delete_conflicting),
                GenType::Provider => generate::generate_provider_with_paths_and_clean(&effective_input, &effective_output, effective_delete_conflicting),
                GenType::All => {
                    generate::generate_freezed_with_paths_and_clean(&effective_input, &effective_output, effective_delete_conflicting);
                    generate::generate_json_with_paths_and_clean(&effective_input, &effective_output, effective_delete_conflicting);
                    generate::generate_riverpod_with_paths_and_clean(&effective_input, &effective_output, effective_delete_conflicting);
                    generate::generate_provider_with_paths_and_clean(&effective_input, &effective_output, effective_delete_conflicting);
                },
            }
        }
        Some(Commands::Assets { assets, output }) => {
            let effective_assets = assets.as_ref().cloned().unwrap_or(effective.assets.clone());
            let effective_output = output.as_ref().cloned().unwrap_or(effective.output.clone());
            assets::generate_assets_with_paths(&effective_assets, &effective_output);
        }
        Some(Commands::All { input, output, assets }) => {
            let effective_input = input.as_ref().cloned().unwrap_or(effective.input.clone());
            let effective_output = output.as_ref().cloned().unwrap_or(effective.output.clone());
            let effective_assets = assets.as_ref().cloned().unwrap_or(effective.assets.clone());
            run_generators(&EffectiveConfig {
                input: effective_input,
                output: effective_output,
                assets: effective_assets,
                watch: effective.watch,
                delete_conflicting_outputs: effective.delete_conflicting_outputs,
            });
        }
        Some(Commands::Clean { input, output }) => {
            let effective_input = input.as_ref().cloned().unwrap_or(effective.input.clone());
            let effective_output = output.as_ref().cloned().unwrap_or(effective.output.clone());
            clean_generated_files(&EffectiveConfig {
                input: effective_input,
                output: effective_output,
                assets: effective.assets.clone(),
                watch: effective.watch,
                delete_conflicting_outputs: effective.delete_conflicting_outputs,
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
        input: if cli.input != "test_flutter_app/aminomi/lib" {
            cli.input.clone()
        } else {
            yaml_gen.input.unwrap_or("test_flutter_app/aminomi/lib".to_string())
        },
        output: if cli.output != "test_flutter_app/aminomi/lib/gen" {
            cli.output.clone()
        } else {
            yaml_gen.output.unwrap_or("test_flutter_app/aminomi/lib/gen".to_string())
        },
        assets: if cli.assets != "assets" {
            cli.assets.clone()
        } else {
            yaml_assets.input.unwrap_or("assets".to_string())
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
    
    // Generate code based on configuration
    if yaml_gen.freezed.unwrap_or(true) {
        generate::generate_freezed_with_paths_and_clean(&cfg.input, &cfg.output, cfg.delete_conflicting_outputs);
    }
    
    if yaml_gen.json.unwrap_or(true) {
        generate::generate_json_with_paths_and_clean(&cfg.input, &cfg.output, cfg.delete_conflicting_outputs);
    }
    
    if yaml_gen.riverpod.unwrap_or(true) {
        generate::generate_riverpod_with_paths_and_clean(&cfg.input, &cfg.output, cfg.delete_conflicting_outputs);
    }
    
    if yaml_gen.provider.unwrap_or(true) {
        generate::generate_provider_with_paths_and_clean(&cfg.input, &cfg.output, cfg.delete_conflicting_outputs);
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
    println!("Watching for changes in {} and pubspec.yaml...", cfg.input);
    let (tx, rx) = channel();
    let config = Config::default().with_poll_interval(Duration::from_secs(1));
    let mut watcher: RecommendedWatcher = Watcher::new(tx, config).unwrap();
    watcher.watch(Path::new(&cfg.input), RecursiveMode::Recursive).unwrap();
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
    
    println!("Cleaning generated files in {}...", cfg.input);
    
    let mut cleaned_count = 0;
    
    // Walk through the input directory and find generated files
    for entry in WalkDir::new(&cfg.input).into_iter().filter_map(|e| e.ok()) {
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
            input: "test_flutter_app/aminomi/lib".to_string(),
            output: "test_flutter_app/aminomi/lib/gen".to_string(),
            assets: "assets".to_string(),
            watch: false,
            delete_conflicting_outputs: false,
        };
        run_generators(&cfg);
    }
}
