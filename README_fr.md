# SuperFastGen

SuperFastGen est un générateur de code haute performance pour les projets Flutter/Dart. Il utilise tree-sitter-dart pour analyser le code Dart et prend en charge la génération de code Freezed, la sérialisation JSON, les providers Riverpod ainsi que la gestion des assets.

## Fonctionnalités

- **Génération de code Freezed**: Génération de classes de données et d'objets immutables
- **Sérialisation JSON**: Génération automatique des méthodes fromJson/toJson
- **Providers Riverpod**: Génération de providers de gestion d'état
- **Gestion des assets**: Détection et gestion automatiques des images, polices, icônes
- **Haute performance**: Génération de code rapide implémentée en Rust
- **Analyse Tree-sitter**: Analyse précise du code Dart

## Installation

```bash
cargo install --path .
```

## Utilisation

### Génération de code de base

```bash
superfastgen generate --input lib/ --output lib/gen/
```

### Gestion des assets

```bash
superfastgen assets --input assets/ --output lib/gen/
```

### Options détaillées

```bash
superfastgen generate \
  --input lib/ \
  --output lib/gen/ \
  --freezed \
  --json \
  --riverpod \
  --verbose
```

## Configuration

Créez un fichier `superfastgen.yaml` à la racine du projet pour personnaliser la configuration :

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

## Développement

### Installation des dépendances

```bash
cargo build
```

### Exécution des tests

```bash
cargo test
```

### Configuration du niveau de log

```bash
RUST_LOG=debug cargo run -- generate --input lib/ --output lib/gen/
```

## Mise à jour de Tree-sitter-dart

### Mise à jour manuelle

Pour mettre à jour le sous-module tree-sitter-dart :

```bash
# Mettre à jour le sous-module vers la dernière version
git submodule update --remote tree-sitter-dart

# Commiter les changements
git add tree-sitter-dart
git commit -m "Update tree-sitter-dart to latest version"

# Reconstruire le projet
cargo clean
cargo build

# Exécuter les tests pour vérifier
cargo test
```

### Mise à jour automatique

Utilisez le script intégré pour la mise à jour automatique :

```bash
# Exécuter le script
cargo run --bin update-tree-sitter

# Ou exécuter directement
./scripts/update-tree-sitter.sh
```

Ce script exécute automatiquement :

- Mise à jour du sous-module tree-sitter-dart
- Reconstruction du projet
- Exécution des tests
- Commit des changements

## Licence

MIT License

---

[English](README.md) | [日本語](README_ja.md) | [Tiếng Việt](README_vi.md) | [简体中文](README_zh_cn.md) | [繁體中文](README_zh_tw.md) | [한국어](README_ko.md) | [Français](README_fr.md)
