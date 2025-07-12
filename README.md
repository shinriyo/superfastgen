# SuperFastGen

A high-performance code generator for Flutter/Dart projects written in Rust.

## 🌍 Available Languages / 利用可能な言語 / Ngôn ngữ có sẵn / 可用语言 / 可用語言 / 사용 가능한 언어 / Langues disponibles

- [English](README.md) (Default)
- [日本語](README.ja.md)
- [Tiếng Việt](README.vi.md)
- [简体中文](README.zh-CN.md)
- [繁體中文](README.zh-TW.md)
- [한국어](README.ko.md)
- [Français](README.fr.md)

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

# Initialize submodules (for tree-sitter-dart)
git submodule update --init

# Build the project
cargo build

# Run the application
cargo run
```

### Updating tree-sitter-dart

```bash
# Manual update
git submodule update --remote tree-sitter-dart
cargo build

# Or use the automated script
./scripts/update-tree-sitter.sh
```

## Usage

### CLI Subcommands

SuperFastGen supports individual generation commands for better control:

```bash
# Generate only Freezed code
superfastgen generate --type freezed

# Generate only JSON serialization
superfastgen generate --type json

# Generate only Riverpod providers
superfastgen generate --type riverpod

# Generate all code types (freezed, json, riverpod)
superfastgen generate --type all

# Generate only assets
superfastgen assets

# Generate everything (code and assets)
superfastgen all

# Run in watch mode (automatically regenerates on file changes)
superfastgen --watch
```

### Basic Usage

```bash
# Run the code generator (generates all types)
cargo run

# Run in watch mode (automatically regenerates on file changes)
cargo run -- --watch
```

### Configuration

Create a `superfastgen.yaml` file in your project root to customize settings:

```yaml
generate:
  input: lib/
  output: lib/gen/
  freezed: true
  json: true
  riverpod: true

assets:
  input: assets/
  output: lib/gen/
  include_images: true
  include_fonts: true
  include_icons: true
```

### What it does

1. **Asset Generation**:

   - Reads `pubspec.yaml`
   - Scans assets directories recursively
   - Generates `lib/gen/assets.gen.dart`

2. **Code Generation**:

   - Scans `lib/` for Dart files
   - Detects annotations: `@freezed`, `@JsonSerializable`, `@riverpod`
   - Generates corresponding `.g.dart` files

3. **Watch Mode**:
   - Monitors `lib/` and `pubspec.yaml` for changes
   - Automatically regenerates code when files are modified
   - Similar to `flutter pub run build_runner watch`

### Example Output

After running `superfastgen generate --type freezed`, you'll get:

```
Generating Freezed code from lib/ to lib/gen/...
Generated: lib/user.g.dart
Generated 1 .g.dart files for freezed
```

After running `superfastgen assets`, you'll get:

```
Generating assets from assets/ to lib/gen/...
Generated assets.gen.dart with 6 asset constants
```

### Generated Files

- `lib/user.g.dart` - Freezed code generation
- `lib/product.g.dart` - JSON serialization
- `lib/provider.g.dart` - Riverpod providers
- `lib/gen/assets.gen.dart` - Asset constants

### Custom Paths

You can specify custom input and output paths:

```bash
# Use custom paths
superfastgen generate --type freezed --input src/ --output generated/

# Generate assets with custom paths
superfastgen assets --assets my-assets/ --output lib/generated/
```

### Custom Project Path

To use with your own Flutter project:

```rust
// In your Rust code
use superfastgen::commands::assets;
use superfastgen::commands::generate;

// Generate assets for your project
assets::generate_assets_from_path("your_flutter_project");

// Generate code for your project
generate::generate_freezed(); // Uses "test_flutter_app/lib" by default
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

- [x] CLI interface with subcommands
- [x] Freezed code generation
- [x] JSON serialization
- [x] Riverpod provider generation
- [x] Asset processing
- [x] Tree-sitter integration
- [x] YAML configuration
- [ ] Template system
- [ ] Plugin architecture
- [ ] CLI arguments for custom paths
- [ ] Watch mode for automatic regeneration
