# SuperFastGen

用 Rust 編寫的高性能 Flutter/Dart 專案程式碼產生器。

## 功能特性

- **Freezed 程式碼產生**: 使用 Freezed 產生不可變資料類別
- **JSON 序列化**: 自動 JSON 序列化/反序列化
- **Riverpod 整合**: 產生 Riverpod 提供者和狀態管理
- **資源管理**: 處理和產生資源檔案
- **Tree-sitter 解析**: 進階程式碼解析和 AST 操作
- **YAML 處理**: 解析和產生 pubspec.yaml 檔案

## 專案結構

```
superfastgen/
├── Cargo.toml
├── src/
│   ├── main.rs              # 主進入點
│   ├── commands/
│   │   ├── mod.rs           # 模組定義
│   │   ├── generate.rs      # Freezed/JSON/Riverpod產生
│   │   └── assets.rs        # 資源產生器
│   └── utils/
│       ├── mod.rs           # 模組定義
│       ├── parser.rs        # Tree-sitter解析工具
│       └── yaml.rs          # pubspec.yaml工具
```

## 安裝

```bash
# 複製儲存庫
git clone https://github.com/shinriyo/superfastgen.git
cd superfastgen

# 初始化子模組（用於tree-sitter-dart）
git submodule update --init

# 建置專案
cargo build

# 執行應用程式
cargo run
```

## 使用方法

### 基本用法

```bash
# 執行程式碼產生器（產生所有類型）
cargo run

# 以監視模式執行（檔案變更時自動重新產生）
cargo run -- --watch
```

### 功能說明

1. **資源產生**:

   - 讀取 `test_flutter_app/pubspec.yaml`
   - 遞迴掃描資源目錄
   - 產生 `test_flutter_app/lib/gen/assets.gen.dart`

2. **程式碼產生**:

   - 掃描 `test_flutter_app/lib/` 中的 Dart 檔案
   - 偵測註解: `@freezed`, `@JsonSerializable`, `@riverpod`
   - 產生相應的 `.g.dart` 檔案

3. **監視模式**:
   - 監視 `test_flutter_app/lib/` 和 `pubspec.yaml` 的變更
   - 檔案修改時自動重新產生程式碼
   - 類似於 `flutter pub run build_runner watch`

### 輸出範例

執行 `cargo run` 後，您將看到:

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

### 產生的檔案

- `test_flutter_app/lib/user.g.dart` - Freezed 程式碼產生
- `test_flutter_app/lib/product.g.dart` - JSON 序列化
- `test_flutter_app/lib/provider.g.dart` - Riverpod 提供者
- `test_flutter_app/lib/gen/assets.gen.dart` - 資源常數

### 自訂專案路徑

要在您自己的 Flutter 專案中使用:

```rust
// 在您的Rust程式碼中
use superfastgen::commands::assets;
use superfastgen::commands::generate;

// 為您的專案產生資源
assets::generate_assets_from_path("your_flutter_project");

// 為您的專案產生程式碼
generate::generate_freezed(); // 預設使用 "test_flutter_app/lib"
```

## 開發

### 前置需求

- Rust 1.70+
- Cargo

### 建置

```bash
# 開發建置
cargo build

# 發布建置
cargo build --release

# 執行測試
cargo test
```

## 相依項目

- `clap`: 命令列參數解析
- `serde`: 序列化框架
- `serde_yaml`: YAML 序列化
- `tree-sitter`: 程式碼解析
- `tera`: 範本引擎

## 授權

MIT License

## 貢獻

1. Fork 儲存庫
2. 建立功能分支
3. 進行變更
4. 如果適用，新增測試
5. 提交拉取請求

## 路線圖

- [x] 帶子命令的 CLI 介面
- [x] Freezed 程式碼產生
- [x] JSON 序列化
- [x] Riverpod 提供者產生
- [x] 資源處理
- [x] Tree-sitter 整合
- [x] YAML 設定
- [ ] 範本系統
- [ ] 外掛架構
- [ ] 自訂路徑的 CLI 參數
- [ ] 自動重新產生的監視模式
