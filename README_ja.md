# SuperFastGen

SuperFastGen は、Flutter/Dart プロジェクト用の高性能コードジェネレーターです。Tree-sitter-dart を使用して Dart コードを解析し、Freezed、JSON シリアライゼーション、Riverpod プロバイダーコード、およびアセット管理の生成をサポートします。

## 機能

- **Freezed コード生成**: データクラスとイミュータブルオブジェクトの生成
- **JSON シリアライゼーション**: fromJson/toJson メソッドの自動生成
- **Riverpod プロバイダー**: 状態管理プロバイダーの生成
- **アセット管理**: 画像、フォント、アイコンの自動検出と管理
- **高性能**: Rust で実装された高速なコード生成
- **Tree-sitter 解析**: 正確な Dart コード解析

## インストール

```bash
cargo install --path .
```

## 使用方法

### CLI サブコマンド

SuperFastGen は個別の生成コマンドをサポートして、より細かい制御が可能です：

```bash
# Freezedコードのみ生成
superfastgen generate --type freezed

# JSONシリアライゼーションのみ生成
superfastgen generate --type json

# Riverpodプロバイダーのみ生成
superfastgen generate --type riverpod

# すべてのコードタイプを生成（freezed、json、riverpod）
superfastgen generate --type all

# アセットのみ生成
superfastgen assets

# すべて生成（コードとアセット）
superfastgen all

# ウォッチモードで実行（ファイル変更時に自動再生成）
superfastgen --watch
```

### 基本的な使用方法

```bash
# コードジェネレーターを実行（すべてのタイプを生成）
cargo run

# ウォッチモードで実行（ファイル変更時に自動再生成）
cargo run -- --watch
```

### 設定

プロジェクトのルートに`superfastgen.yaml`ファイルを作成して設定をカスタマイズできます：

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

### 機能

1. **アセット生成**:

   - `pubspec.yaml`を読み込み
   - アセットディレクトリを再帰的にスキャン
   - `lib/gen/assets.gen.dart`を生成

2. **コード生成**:

   - `lib/`の Dart ファイルをスキャン
   - アノテーションを検出: `@freezed`、`@JsonSerializable`、`@riverpod`
   - 対応する`.g.dart`ファイルを生成

3. **ウォッチモード**:
   - `lib/`と`pubspec.yaml`の変更を監視
   - ファイルが変更されると自動的にコードを再生成
   - `flutter pub run build_runner watch`と同様

### 出力例

`superfastgen generate --type freezed`を実行すると：

```
Generating Freezed code from lib/ to lib/gen/...
Generated: lib/user.g.dart
Generated 1 .g.dart files for freezed
```

`superfastgen assets`を実行すると：

```
Generating assets from assets/ to lib/gen/...
Generated assets.gen.dart with 6 asset constants
```

### 生成されるファイル

- `lib/user.g.dart` - Freezed コード生成
- `lib/product.g.dart` - JSON シリアライゼーション
- `lib/provider.g.dart` - Riverpod プロバイダー
- `lib/gen/assets.gen.dart` - アセット定数

### カスタムパス

カスタムの入力・出力パスを指定できます：

```bash
# カスタムパスを使用
superfastgen generate --type freezed --input src/ --output generated/

# カスタムパスでアセット生成
superfastgen assets --assets my-assets/ --output lib/generated/
```

## 開発

### 依存関係のインストール

```bash
cargo build
```

### テストの実行

```bash
cargo test
```

### ログレベルの設定

```bash
RUST_LOG=debug cargo run -- generate --input lib/ --output lib/gen/
```

## Tree-sitter-dart の更新

### 手動更新

tree-sitter-dart サブモジュールを更新するには：

```bash
# サブモジュールを最新の状態に更新
git submodule update --remote tree-sitter-dart

# 変更をコミット
git add tree-sitter-dart
git commit -m "Update tree-sitter-dart to latest version"

# プロジェクトを再ビルド
cargo clean
cargo build

# テストを実行して動作確認
cargo test
```

### 自動更新

組み込みのスクリプトを使用して自動更新：

```bash
# スクリプトを実行
cargo run --bin update-tree-sitter

# または直接実行
./scripts/update-tree-sitter.sh
```

このスクリプトは以下を自動実行します：

- tree-sitter-dart サブモジュールの更新
- プロジェクトの再ビルド
- テストの実行
- 変更のコミット

## ライセンス

MIT License

---

[English](README.md) | [日本語](README_ja.md) | [Tiếng Việt](README_vi.md) | [简体中文](README_zh_cn.md) | [繁體中文](README_zh_tw.md) | [한국어](README_ko.md) | [Français](README_fr.md)
