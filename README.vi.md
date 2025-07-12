# SuperFastGen

Một trình tạo mã hiệu suất cao cho dự án Flutter/Dart được viết bằng Rust.

## Tính năng

- **Tạo mã Freezed**: Tạo các lớp dữ liệu bất biến với Freezed
- **Tuần tự hóa JSON**: Tự động tuần tự hóa/giải tuần tự hóa JSON
- **Tích hợp Riverpod**: Tạo các provider và quản lý trạng thái Riverpod
- **Quản lý tài nguyên**: Xử lý và tạo tệp tài nguyên
- **Phân tích cú pháp Tree-sitter**: Phân tích mã nâng cao và thao tác AST
- **Xử lý YAML**: Phân tích và tạo tệp pubspec.yaml

## Cấu trúc dự án

```
superfastgen/
├── Cargo.toml
├── src/
│   ├── main.rs              # Điểm vào chính
│   ├── commands/
│   │   ├── mod.rs           # Định nghĩa module
│   │   ├── generate.rs      # Tạo mã Freezed/JSON/Riverpod
│   │   └── assets.rs        # Trình tạo tài nguyên
│   └── utils/
│       ├── mod.rs           # Định nghĩa module
│       ├── parser.rs        # Tiện ích phân tích cú pháp Tree-sitter
│       └── yaml.rs          # Tiện ích pubspec.yaml
```

## Cài đặt

```bash
# Clone repository
git clone https://github.com/shinriyo/superfastgen.git
cd superfastgen

# Khởi tạo submodules (cho tree-sitter-dart)
git submodule update --init

# Build dự án
cargo build

# Chạy ứng dụng
cargo run
```

## Sử dụng

### Sử dụng cơ bản

```bash
# Chạy trình tạo mã (tạo tất cả các loại)
cargo run

# Chạy ở chế độ watch (tự động tạo lại khi có thay đổi tệp)
cargo run -- --watch
```

### Những gì nó làm

1. **Tạo tài nguyên**:

   - Đọc `test_flutter_app/pubspec.yaml`
   - Quét các thư mục tài nguyên đệ quy
   - Tạo `test_flutter_app/lib/gen/assets.gen.dart`

2. **Tạo mã**:

   - Quét `test_flutter_app/lib/` cho các tệp Dart
   - Phát hiện chú thích: `@freezed`, `@JsonSerializable`, `@riverpod`
   - Tạo các tệp `.g.dart` tương ứng

3. **Chế độ Watch**:
   - Giám sát `test_flutter_app/lib/` và `pubspec.yaml` để thay đổi
   - Tự động tạo lại mã khi tệp được sửa đổi
   - Tương tự như `flutter pub run build_runner watch`

### Ví dụ đầu ra

Sau khi chạy `cargo run`, bạn sẽ nhận được:

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

### Tệp được tạo

- `test_flutter_app/lib/user.g.dart` - Tạo mã Freezed
- `test_flutter_app/lib/product.g.dart` - Tuần tự hóa JSON
- `test_flutter_app/lib/provider.g.dart` - Provider Riverpod
- `test_flutter_app/lib/gen/assets.gen.dart` - Hằng số tài nguyên

### Đường dẫn dự án tùy chỉnh

Để sử dụng với dự án Flutter của riêng bạn:

```rust
// Trong mã Rust của bạn
use superfastgen::commands::assets;
use superfastgen::commands::generate;

// Tạo tài nguyên cho dự án của bạn
assets::generate_assets_from_path("your_flutter_project");

// Tạo mã cho dự án của bạn
generate::generate_freezed(); // Mặc định sử dụng "test_flutter_app/lib"
```

## Phát triển

### Yêu cầu

- Rust 1.70+
- Cargo

### Build

```bash
# Build phát triển
cargo build

# Build release
cargo build --release

# Chạy test
cargo test
```

## Dependencies

- `clap`: Phân tích đối số dòng lệnh
- `serde`: Framework tuần tự hóa
- `serde_yaml`: Tuần tự hóa YAML
- `tree-sitter`: Phân tích mã
- `tera`: Engine template

## Giấy phép

MIT License

## Đóng góp

1. Fork repository
2. Tạo feature branch
3. Thực hiện thay đổi
4. Thêm test nếu cần thiết
5. Gửi pull request

## Lộ trình

- [x] Giao diện CLI với subcommands
- [x] Tạo mã Freezed
- [x] Tuần tự hóa JSON
- [x] Tạo provider Riverpod
- [x] Xử lý tài nguyên
- [x] Tích hợp Tree-sitter
- [x] Cấu hình YAML
- [ ] Hệ thống template
- [ ] Kiến trúc plugin
- [ ] Đối số CLI cho đường dẫn tùy chỉnh
- [ ] Chế độ watch cho tạo lại tự động
