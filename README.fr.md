# SuperFastGen

Un générateur de code haute performance pour les projets Flutter/Dart écrit en Rust.

## Fonctionnalités

- **Génération de code Freezed**: Générer des classes de données immutables avec Freezed
- **Sérialisation JSON**: Sérialisation/désérialisation JSON automatique
- **Intégration Riverpod**: Générer des providers et gestion d'état Riverpod
- **Gestion des assets**: Traiter et générer des fichiers d'assets
- **Parsing Tree-sitter**: Analyse de code avancée et manipulation AST
- **Traitement YAML**: Parser et générer des fichiers pubspec.yaml

## Structure du projet

```
superfastgen/
├── Cargo.toml
├── src/
│   ├── main.rs              # Point d'entrée principal
│   ├── commands/
│   │   ├── mod.rs           # Définitions de modules
│   │   ├── generate.rs      # Génération Freezed/JSON/Riverpod
│   │   └── assets.rs        # Générateur d'assets
│   └── utils/
│       ├── mod.rs           # Définitions de modules
│       ├── parser.rs        # Utilitaires de parsing Tree-sitter
│       └── yaml.rs          # Utilitaires pubspec.yaml
```

## Installation

```bash
# Cloner le repository
git clone https://github.com/shinriyo/superfastgen.git
cd superfastgen

# Initialiser les sous-modules (pour tree-sitter-dart)
git submodule update --init

# Construire le projet
cargo build

# Exécuter l'application
cargo run
```

## Utilisation

### Utilisation de base

```bash
# Exécuter le générateur de code (génère tous les types)
cargo run

# Exécuter en mode watch (régénère automatiquement lors des changements de fichiers)
cargo run -- --watch
```

### Ce qu'il fait

1. **Génération d'assets**:

   - Lit `test_flutter_app/pubspec.yaml`
   - Scanne récursivement les répertoires d'assets
   - Génère `test_flutter_app/lib/gen/assets.gen.dart`

2. **Génération de code**:

   - Scanne `test_flutter_app/lib/` pour les fichiers Dart
   - Détecte les annotations: `@freezed`, `@JsonSerializable`, `@riverpod`
   - Génère les fichiers `.g.dart` correspondants

3. **Mode Watch**:
   - Surveille `test_flutter_app/lib/` et `pubspec.yaml` pour les changements
   - Régénère automatiquement le code quand les fichiers sont modifiés
   - Similaire à `flutter pub run build_runner watch`

### Exemple de sortie

Après avoir exécuté `cargo run`, vous obtiendrez:

```
Generating Freezed code...
Generated: test_flutter_app/lib/user.g.dart
Generated 1 .g.dart files for freezed

Generating JSON code...
Generated: test_flutter_app/lib/product.g.dart
Generated 1 .g.dart files for json

Generating Riverpod code...
Generated: test_flutter_app/lib/provider.g.dart
Generated 2 .g.dart files for riverpod

Generating assets from test_flutter_app
Generated assets.gen.dart with 6 asset constants
```

### Fichiers générés

- `test_flutter_app/lib/user.g.dart` - Génération de code Freezed
- `test_flutter_app/lib/product.g.dart` - Sérialisation JSON
- `test_flutter_app/lib/provider.g.dart` - Providers Riverpod
- `test_flutter_app/lib/gen/assets.gen.dart` - Constantes d'assets

### Chemin de projet personnalisé

Pour utiliser avec votre propre projet Flutter:

```rust
// Dans votre code Rust
use superfastgen::commands::assets;
use superfastgen::commands::generate;

// Générer des assets pour votre projet
assets::generate_assets_from_path("your_flutter_project");

// Générer du code pour votre projet
generate::generate_freezed(); // Utilise "test_flutter_app/lib" par défaut
```

## Développement

### Prérequis

- Rust 1.70+
- Cargo

### Construction

```bash
# Build de développement
cargo build

# Build de release
cargo build --release

# Exécuter les tests
cargo test
```

## Dépendances

- `clap`: Parsing d'arguments en ligne de commande
- `serde`: Framework de sérialisation
- `serde_yaml`: Sérialisation YAML
- `tree-sitter`: Parsing de code
- `tera`: Moteur de template

## Licence

MIT License

## Contribution

1. Fork le repository
2. Créer une branche de fonctionnalité
3. Faire vos modifications
4. Ajouter des tests si applicable
5. Soumettre une pull request

## Feuille de route

- [x] Interface CLI avec sous-commandes
- [x] Génération de code Freezed
- [x] Sérialisation JSON
- [x] Génération de providers Riverpod
- [x] Traitement d'assets
- [x] Intégration Tree-sitter
- [x] Configuration YAML
- [ ] Système de templates
- [ ] Architecture de plugins
- [ ] Arguments CLI pour des chemins personnalisés
- [ ] Mode watch pour la régénération automatique
