# SuperFastGen

Rust 製の高性能 Flutter/Dart コードジェネレーターです。

## 特徴

- **Freezed コード生成**: 不変データクラスを自動生成
- **JSON シリアライズ**: JSON のシリアライズ/デシリアライズを自動化
- **Riverpod 連携**: Riverpod プロバイダーや状態管理コードを生成
- **アセット管理**: アセットファイルの自動検出と Dart 定数生成
- **Tree-sitter パース**: 高度なコード解析と AST 操作
- **YAML 処理**: pubspec.yaml の解析と生成

## プロジェクト構成

```
superfastgen/
├── Cargo.toml
├── src/
│   ├── main.rs              # メインエントリポイント
│   ├── commands/
│   │   ├── mod.rs           # モジュール定義
│   │   ├── generate.rs      # Freezed/JSON/Riverpod生成
│   │   └── assets.rs        # アセット生成
│   └── utils/
│       ├── mod.rs           # モジュール定義
│       ├── parser.rs        # Tree-sitterユーティリティ
│       └── yaml.rs          # pubspec.yamlユーティリティ
```

## インストール

```bash
# リポジトリをクローン
git clone https://github.com/shinriyo/superfastgen.git
cd superfastgen

# サブモジュール初期化（tree-sitter-dart用）
git submodule update --init

# ビルド
cargo build

# 実行
cargo run
```

## 使い方

### 基本的な使い方

```bash
# コード生成（全てのタイプを生成）
cargo run

# ウォッチモード（ファイル変更時に自動再生成）
cargo run -- --watch
```

### 何ができるか

1. **アセット生成**:

   - `test_flutter_app/pubspec.yaml` を読み込み
   - アセットディレクトリを再帰的にスキャン
   - `test_flutter_app/lib/gen/assets.gen.dart` を生成

2. **コード生成**:

   - `test_flutter_app/lib/` 内の Dart ファイルをスキャン
   - `@freezed`, `@JsonSerializable`, `@riverpod` アノテーションを検出
   - 対応する `.g.dart` ファイルを生成

3. **ウォッチモード**:
   - `test_flutter_app/lib/` や `pubspec.yaml` の変更を監視
   - ファイル変更時に自動で再生成
   - `flutter pub run build_runner watch` に近い動作

### 出力例

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

### 生成ファイル

- `test_flutter_app/lib/user.g.dart` - Freezed 用コード
- `test_flutter_app/lib/product.g.dart` - JSON シリアライズ用
- `test_flutter_app/lib/provider.g.dart` - Riverpod 用
- `test_flutter_app/lib/gen/assets.gen.dart` - アセット定数

### カスタムプロジェクトパス

独自の Flutter プロジェクトで使う場合:

```rust
// Rustコード例
use superfastgen::commands::assets;
use superfastgen::commands::generate;

// アセット生成
assets::generate_assets_from_path("your_flutter_project");

// コード生成
generate::generate_freezed(); // デフォルトは "test_flutter_app/lib"
```

## 開発

### 必要環境

- Rust 1.70+
- Cargo

### ビルド

```bash
# 開発ビルド
cargo build

# リリースビルド
cargo build --release

# テスト実行
cargo test
```

## 依存クレート

- `clap`: コマンドライン引数パーサ
- `serde`: シリアライズ
- `serde_yaml`: YAML シリアライズ
- `tree-sitter`: コードパース
- `tera`: テンプレートエンジン

## ライセンス

MIT License

## コントリビュート

1. フォーク
2. フィーチャーブランチ作成
3. 変更
4. テスト追加（必要なら）
5. プルリクエスト

## ロードマップ

- [x] CLI サブコマンド
- [x] Freezed コード生成
- [x] JSON シリアライズ
- [x] Riverpod 生成
- [x] アセット処理
- [x] Tree-sitter 連携
- [x] YAML 設定
- [ ] テンプレートシステム
- [ ] プラグインアーキテクチャ
- [ ] CLI 引数でカスタムパス
- [ ] 自動再生成のウォッチモード
