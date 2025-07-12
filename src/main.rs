use clap::{Parser, Subcommand, ValueEnum};

mod commands;
mod utils;

use commands::{generate, assets};
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
    #[arg(long, default_value = "lib")]
    input: String,
    /// Output directory for generated files
    #[arg(long, default_value = "lib/gen")]
    output: String,
    /// Assets directory
    #[arg(long, default_value = "assets")]
    assets: String,
    /// Watch mode for file changes
    #[arg(long)]
    watch: bool,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Generate code for a specific type
    Generate {
        #[arg(long, value_enum, default_value = "all")]
        r#type: GenType,
    },
    /// Generate only assets
    Assets,
    /// Generate everything (code and assets)
    All,
}

#[derive(ValueEnum, Debug, Clone)]
enum GenType {
    Freezed,
    Json,
    Riverpod,
    All,
}

#[derive(Debug, Clone)]
struct EffectiveConfig {
    input: String,
    output: String,
    assets: String,
    watch: bool,
}

fn main() {
    env_logger::init();
    info!("SuperFastGen - Code Generator");
    let cli = Cli::parse();
    let yaml_config = yaml::parse_superfastgen_yaml("superfastgen.yaml");
    let effective = merge_config(&cli, yaml_config);

    match &cli.command {
        Some(Commands::Generate { r#type }) => {
            match r#type {
                GenType::Freezed => generate::generate_freezed_with_paths(&effective.input, &effective.output),
                GenType::Json => generate::generate_json_with_paths(&effective.input, &effective.output),
                GenType::Riverpod => generate::generate_riverpod_with_paths(&effective.input, &effective.output),
                GenType::All => run_generators(&effective),
            }
        }
        Some(Commands::Assets) => {
            assets::generate_assets_with_paths(&effective.assets, &effective.output);
        }
        Some(Commands::All) => {
            run_generators(&effective);
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
        input: if cli.input != "lib" {
            cli.input.clone()
        } else {
            yaml_gen.input.unwrap_or("lib".to_string())
        },
        output: if cli.output != "lib/gen" {
            cli.output.clone()
        } else {
            yaml_gen.output.unwrap_or("lib/gen".to_string())
        },
        assets: if cli.assets != "assets" {
            cli.assets.clone()
        } else {
            yaml_assets.input.unwrap_or("assets".to_string())
        },
        // Watch mode controlled only by CLI --watch flag
        watch: cli.watch,
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
    
    // Use configuration to determine which generators to run
    if yaml_gen.freezed.unwrap_or(true) {
        generate::generate_freezed_with_paths(&cfg.input, &cfg.output);
    }
    if yaml_gen.json.unwrap_or(true) {
        generate::generate_json_with_paths(&cfg.input, &cfg.output);
    }
    if yaml_gen.riverpod.unwrap_or(true) {
        generate::generate_riverpod_with_paths(&cfg.input, &cfg.output);
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_run_generators() {
        let cfg = EffectiveConfig {
            input: "lib".to_string(),
            output: "lib/gen".to_string(),
            assets: "assets".to_string(),
            watch: false,
        };
        run_generators(&cfg);
    }
}
