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

### Tạo mã cơ bản

```bash
superfastgen generate --input lib/ --output lib/gen/
```

### Quản lý tài nguyên

```bash
superfastgen assets --input assets/ --output lib/gen/
```

### Tùy chọn chi tiết

```bash
superfastgen generate \
  --input lib/ \
  --output lib/gen/ \
  --freezed \
  --json \
  --riverpod \
  --verbose
```

## Cấu hình

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
