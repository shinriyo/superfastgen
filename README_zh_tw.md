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

### CLI 子命令

SuperFastGen 支援單獨的生成命令以提供更好的控制：

```bash
# 僅生成Freezed程式碼
superfastgen generate --type freezed

# 僅生成JSON序列化
superfastgen generate --type json

# 僅生成Riverpod提供者
superfastgen generate --type riverpod

# 生成所有程式碼類型（freezed、json、riverpod）
superfastgen generate --type all

# 僅生成資源
superfastgen assets

# 生成所有內容（程式碼和資源）
superfastgen all

# 以監視模式執行（檔案變更時自動重新生成）
superfastgen --watch
```

### 基本用法

```bash
# 執行程式碼產生器（生成所有類型）
cargo run

# 以監視模式執行（檔案變更時自動重新生成）
cargo run -- --watch
```

### 設定

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

### 功能

1. **資源生成**：

   - 讀取`pubspec.yaml`
   - 遞迴掃描資源目錄
   - 生成`lib/gen/assets.gen.dart`

2. **程式碼生成**：

   - 掃描`lib/`中的 Dart 檔案
   - 檢測註解：`@freezed`、`@JsonSerializable`、`@riverpod`
   - 生成相應的`.g.dart`檔案

3. **監視模式**：
   - 監視`lib/`和`pubspec.yaml`的變更
   - 檔案修改時自動重新生成程式碼
   - 類似於`flutter pub run build_runner watch`

### 輸出範例

執行`superfastgen generate --type freezed`後，您將得到：

```
Generating Freezed code from lib/ to lib/gen/...
Generated: lib/user.g.dart
Generated 1 .g.dart files for freezed
```

執行`superfastgen assets`後，您將得到：

```
Generating assets from assets/ to lib/gen/...
Generated assets.gen.dart with 6 asset constants
```

### 生成的檔案

- `lib/user.g.dart` - Freezed 程式碼生成
- `lib/product.g.dart` - JSON 序列化
- `lib/provider.g.dart` - Riverpod 提供者
- `lib/gen/assets.gen.dart` - 資源常數

### 自訂路徑

您可以指定自訂的輸入和輸出路徑：

```bash
# 使用自訂路徑
superfastgen generate --type freezed --input src/ --output generated/

# 使用自訂路徑生成資源
superfastgen assets --assets my-assets/ --output lib/generated/
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
