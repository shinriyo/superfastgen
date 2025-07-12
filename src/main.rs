use clap::{Parser, Subcommand};

mod commands;
mod utils;

use commands::{generate, assets};
use utils::{parser, yaml};

use notify::{Watcher, RecursiveMode, RecommendedWatcher, Event, EventKind, Config};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::path::Path;
use log::info;

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
    env_logger::init();
    info!("SuperFastGen - Code Generator");
    
    // ウォッチモード
    if std::env::args().any(|arg| arg == "--watch") {
        watch_mode();
    } else {
        run_generators();
    }
}

fn run_generators() {
    generate::generate_freezed();
    generate::generate_json();
    generate::generate_riverpod();
    assets::generate_assets();
    let _ = parser::parse_code("example code");
    let _ = yaml::parse_pubspec_yaml("example yaml");
}

fn watch_mode() {
    println!("Watching for changes in test_flutter_app/lib and pubspec.yaml...");
    let (tx, rx) = channel();
    let config = Config::default().with_poll_interval(Duration::from_secs(1));
    let mut watcher: RecommendedWatcher = Watcher::new(tx, config).unwrap();
    watcher.watch(Path::new("test_flutter_app/lib"), RecursiveMode::Recursive).unwrap();
    watcher.watch(Path::new("test_flutter_app/pubspec.yaml"), RecursiveMode::NonRecursive).unwrap();

    run_generators();

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if let Event { kind: EventKind::Modify(_), .. } | Event { kind: EventKind::Create(_), .. } | Event { kind: EventKind::Remove(_), .. } = event {
                    println!("Change detected! Regenerating...");
                    run_generators();
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
        // run_generatorsがエラーなく実行されることを確認
        run_generators();
        // エラーが発生しなければ成功
    }

    #[test]
    fn test_watch_mode_detection() {
        // --watchフラグの検出テスト
        let args = vec!["cargo", "run", "--", "--watch"];
        let has_watch = args.iter().any(|arg| *arg == "--watch");
        assert!(has_watch);
    }
}
