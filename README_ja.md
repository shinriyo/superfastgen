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

### 基本的なコード生成

```bash
superfastgen generate --input lib/ --output lib/gen/
```

### アセット管理

```bash
superfastgen assets --input assets/ --output lib/gen/
```

### 詳細オプション

```bash
superfastgen generate \
  --input lib/ \
  --output lib/gen/ \
  --freezed \
  --json \
  --riverpod \
  --verbose
```

## 設定

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
