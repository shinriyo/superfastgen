# SuperFastGen

SuperFastGen là một trình tạo mã hiệu suất cao cho các dự án Flutter/Dart. Sử dụng tree-sitter-dart để phân tích mã Dart và hỗ trợ tạo mã Freezed, JSON serialization, Riverpod provider, cũng như quản lý tài nguyên.

## Tính năng

- **Tạo mã Freezed**: Tạo data classes và immutable objects
- **JSON Serialization**: Tự động tạo fromJson/toJson methods
- **Riverpod Provider**: Tạo state management providers
- **Quản lý tài nguyên**: Tự động phát hiện và quản lý hình ảnh, font, icons
- **Hiệu suất cao**: Tạo mã nhanh được viết bằng Rust
- **Phân tích Tree-sitter**: Phân tích mã Dart chính xác

## Cài đặt

```bash
cargo install --path .
```

## Cách sử dụng

### CLI Subcommands

SuperFastGen hỗ trợ các lệnh tạo mã riêng biệt để kiểm soát tốt hơn:

```bash
# Chỉ tạo mã Freezed
superfastgen generate --type freezed

# Chỉ tạo JSON serialization
superfastgen generate --type json

# Chỉ tạo Riverpod providers
superfastgen generate --type riverpod

# Tạo tất cả loại mã (freezed, json, riverpod)
superfastgen generate --type all

# Chỉ tạo assets
superfastgen assets

# Tạo tất cả (mã và assets)
superfastgen all

# Chạy ở chế độ watch (tự động tạo lại khi file thay đổi)
superfastgen --watch
```

### Cách sử dụng cơ bản

```bash
# Chạy code generator (tạo tất cả loại)
cargo run

# Chạy ở chế độ watch (tự động tạo lại khi file thay đổi)
cargo run -- --watch
```

### Cấu hình

Tạo file `superfastgen.yaml` ở thư mục gốc của dự án để tùy chỉnh cấu hình:

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

### Chức năng

1. **Tạo Assets**:

   - Đọc `pubspec.yaml`
   - Quét thư mục assets một cách đệ quy
   - Tạo `lib/gen/assets.gen.dart`

2. **Tạo Mã**:

   - Quét `lib/` cho các file Dart
   - Phát hiện annotations: `@freezed`, `@JsonSerializable`, `@riverpod`
   - Tạo các file `.g.dart` tương ứng

3. **Chế độ Watch**:
   - Giám sát `lib/` và `pubspec.yaml` để phát hiện thay đổi
   - Tự động tạo lại mã khi file được sửa đổi
   - Tương tự như `flutter pub run build_runner watch`

### Ví dụ đầu ra

Sau khi chạy `superfastgen generate --type freezed`, bạn sẽ có:

```
Generating Freezed code from lib/ to lib/gen/...
Generated: lib/user.g.dart
Generated 1 .g.dart files for freezed
```

Sau khi chạy `superfastgen assets`, bạn sẽ có:

```
Generating assets from assets/ to lib/gen/...
Generated assets.gen.dart with 6 asset constants
```

### Các file được tạo

- `lib/user.g.dart` - Tạo mã Freezed
- `lib/product.g.dart` - JSON serialization
- `lib/provider.g.dart` - Riverpod providers
- `lib/gen/assets.gen.dart` - Asset constants

### Đường dẫn tùy chỉnh

Bạn có thể chỉ định đường dẫn đầu vào và đầu ra tùy chỉnh:

```bash
# Sử dụng đường dẫn tùy chỉnh
superfastgen generate --type freezed --input src/ --output generated/

# Tạo assets với đường dẫn tùy chỉnh
superfastgen assets --assets my-assets/ --output lib/generated/
```

## Phát triển

### Cài đặt dependencies

```bash
cargo build
```

### Chạy tests

```bash
cargo test
```

### Thiết lập log level

```bash
RUST_LOG=debug cargo run -- generate --input lib/ --output lib/gen/
```

## Cập nhật Tree-sitter-dart

### Cập nhật thủ công

Để cập nhật tree-sitter-dart submodule:

```bash
# Cập nhật submodule lên phiên bản mới nhất
git submodule update --remote tree-sitter-dart

# Commit thay đổi
git add tree-sitter-dart
git commit -m "Update tree-sitter-dart to latest version"

# Rebuild project
cargo clean
cargo build

# Chạy tests để kiểm tra
cargo test
```

### Cập nhật tự động

Sử dụng script tích hợp để cập nhật tự động:

```bash
# Chạy script
cargo run --bin update-tree-sitter

# Hoặc chạy trực tiếp
./scripts/update-tree-sitter.sh
```

Script này sẽ tự động thực hiện:

- Cập nhật tree-sitter-dart submodule
- Rebuild project
- Chạy tests
- Commit thay đổi

## Giấy phép

MIT License

---

[English](README.md) | [日本語](README_ja.md) | [Tiếng Việt](README_vi.md) | [简体中文](README_zh_cn.md) | [繁體中文](README_zh_tw.md) | [한국어](README_ko.md) | [Français](README_fr.md)
