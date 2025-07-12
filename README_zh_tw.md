# SuperFastGen

SuperFastGen 是一個高性能的 Flutter/Dart 專案程式碼產生器。使用 tree-sitter-dart 解析 Dart 程式碼，支援產生 Freezed、JSON 序列化、Riverpod 提供者程式碼以及資源管理。

## 功能

- **Freezed 程式碼產生**: 產生資料類別和不可變物件
- **JSON 序列化**: 自動產生 fromJson/toJson 方法
- **Riverpod 提供者**: 產生狀態管理提供者
- **資源管理**: 自動偵測和管理圖片、字型、圖示
- **高性能**: 用 Rust 實現的高速程式碼產生
- **Tree-sitter 解析**: 準確的 Dart 程式碼解析

## 安裝

```bash
cargo install --path .
```

## 使用方法

### 基本程式碼產生

```bash
superfastgen generate --input lib/ --output lib/gen/
```

### 資源管理

```bash
superfastgen assets --input assets/ --output lib/gen/
```

### 詳細選項

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

在專案根目錄建立`superfastgen.yaml`檔案來自訂設定：

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

## 開發

### 安裝依賴

```bash
cargo build
```

### 執行測試

```bash
cargo test
```

### 設定日誌等級

```bash
RUST_LOG=debug cargo run -- generate --input lib/ --output lib/gen/
```

## 更新 Tree-sitter-dart

### 手動更新

要更新 tree-sitter-dart 子模組：

```bash
# 更新子模組到最新版本
git submodule update --remote tree-sitter-dart

# 提交變更
git add tree-sitter-dart
git commit -m "Update tree-sitter-dart to latest version"

# 重新建置專案
cargo clean
cargo build

# 執行測試驗證
cargo test
```

### 自動更新

使用內建腳本進行自動更新：

```bash
# 執行腳本
cargo run --bin update-tree-sitter

# 或直接執行
./scripts/update-tree-sitter.sh
```

此腳本將自動執行：

- 更新 tree-sitter-dart 子模組
- 重新建置專案
- 執行測試
- 提交變更

## 授權

MIT License

---

[English](README.md) | [日本語](README_ja.md) | [Tiếng Việt](README_vi.md) | [简体中文](README_zh_cn.md) | [繁體中文](README_zh_tw.md) | [한국어](README_ko.md) | [Français](README_fr.md)
