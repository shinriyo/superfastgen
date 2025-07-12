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

### Sous-commandes CLI

SuperFastGen prend en charge des commandes de génération individuelles pour un meilleur contrôle :

```bash
# Générer uniquement le code Freezed
superfastgen generate --type freezed

# Générer uniquement la sérialisation JSON
superfastgen generate --type json

# Générer uniquement les providers Riverpod
superfastgen generate --type riverpod

# Générer tous les types de code (freezed, json, riverpod)
superfastgen generate --type all

# Générer uniquement les assets
superfastgen assets

# Générer tout (code et assets)
superfastgen all

# Exécuter en mode watch (régénération automatique lors des changements de fichiers)
superfastgen --watch
```

### Utilisation de base

```bash
# Exécuter le générateur de code (génère tous les types)
cargo run

# Exécuter en mode watch (régénération automatique lors des changements de fichiers)
cargo run -- --watch
```

### Configuration

Créez un fichier `superfastgen.yaml` à la racine de votre projet pour personnaliser les paramètres :

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

### Fonctionnalités

1. **Génération d'assets** :

   - Lit `pubspec.yaml`
   - Scanne récursivement les répertoires d'assets
   - Génère `lib/gen/assets.gen.dart`

2. **Génération de code** :

   - Scanne `lib/` pour les fichiers Dart
   - Détecte les annotations : `@freezed`, `@JsonSerializable`, `@riverpod`
   - Génère les fichiers `.g.dart` correspondants

3. **Mode watch** :
   - Surveille `lib/` et `pubspec.yaml` pour les changements
   - Régénère automatiquement le code lorsque les fichiers sont modifiés
   - Similaire à `flutter pub run build_runner watch`

### Exemple de sortie

Après avoir exécuté `superfastgen generate --type freezed`, vous obtiendrez :

```
Generating Freezed code from lib/ to lib/gen/...
Generated: lib/user.g.dart
Generated 1 .g.dart files for freezed
```

Après avoir exécuté `superfastgen assets`, vous obtiendrez :

```
Generating assets from assets/ to lib/gen/...
Generated assets.gen.dart with 6 asset constants
```

### Fichiers générés

- `lib/user.g.dart` - Génération de code Freezed
- `lib/product.g.dart` - Sérialisation JSON
- `lib/provider.g.dart` - Providers Riverpod
- `lib/gen/assets.gen.dart` - Constantes d'assets

### Chemins personnalisés

Vous pouvez spécifier des chemins d'entrée et de sortie personnalisés :

```bash
# Utiliser des chemins personnalisés
superfastgen generate --type freezed --input src/ --output generated/

# Générer des assets avec des chemins personnalisés
superfastgen assets --assets my-assets/ --output lib/generated/
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
