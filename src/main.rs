use clap::{Parser, Subcommand};

mod commands;
mod utils;

use commands::{generate, assets};
use utils::{parser, yaml};

#[derive(Parser)]
#[command(name = "SuperFastGen")]
#[command(about = "Blazing fast codegen for Dart/Flutter", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generate,
    Assets,
}

fn main() {
    println!("SuperFastGen - Code Generator");
    
    // Example usage
    generate::generate_freezed();
    generate::generate_json();
    generate::generate_riverpod();
    
    assets::generate_assets();
    assets::process_images();
    assets::process_fonts();
    
    let _ = parser::parse_code("example code");
    let _ = yaml::parse_pubspec_yaml("example yaml");
}
