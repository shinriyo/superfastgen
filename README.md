# SuperFastGen

A high-performance code generator for Flutter/Dart projects written in Rust.

## Features

- **Freezed Code Generation**: Generate immutable data classes with Freezed
- **JSON Serialization**: Automatic JSON serialization/deserialization
- **Riverpod Integration**: Generate Riverpod providers and state management
- **Asset Management**: Process and generate asset files
- **Tree-sitter Parsing**: Advanced code parsing and AST manipulation
- **YAML Processing**: Parse and generate pubspec.yaml files

## Project Structure

```
superfastgen/
├── Cargo.toml
├── src/
│   ├── main.rs              # Main entry point
│   ├── commands/
│   │   ├── mod.rs           # Module definitions
│   │   ├── generate.rs      # Freezed/JSON/Riverpod generation
│   │   └── assets.rs        # Asset generator
│   └── utils/
│       ├── mod.rs           # Module definitions
│       ├── parser.rs        # Tree-sitter parsing utilities
│       └── yaml.rs          # pubspec.yaml utilities
```

## Installation

```bash
# Clone the repository
git clone https://github.com/shinriyo/superfastgen.git
cd superfastgen

# Build the project
cargo build

# Run the application
cargo run
```

## Usage

```bash
# Run the code generator
cargo run

# Build for release
cargo build --release
```

## Development

### Prerequisites

- Rust 1.70+
- Cargo

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test
```

## Dependencies

- `clap`: Command-line argument parsing
- `serde`: Serialization framework
- `serde_yaml`: YAML serialization
- `tree-sitter`: Code parsing
- `tera`: Template engine

## License

MIT License

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Roadmap

- [ ] CLI interface with subcommands
- [ ] Freezed code generation
- [ ] JSON serialization
- [ ] Riverpod provider generation
- [ ] Asset processing
- [ ] Tree-sitter integration
- [ ] YAML configuration
- [ ] Template system
- [ ] Plugin architecture
