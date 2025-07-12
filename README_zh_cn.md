# SuperFastGen

SuperFastGen 是一个高性能的 Flutter/Dart 项目代码生成器。使用 tree-sitter-dart 解析 Dart 代码，支持生成 Freezed、JSON 序列化、Riverpod 提供者代码以及资源管理。

## 功能

- **Freezed 代码生成**: 生成数据类和不可变对象
- **JSON 序列化**: 自动生成 fromJson/toJson 方法
- **Riverpod 提供者**: 生成状态管理提供者
- **资源管理**: 自动检测和管理图片、字体、图标
- **高性能**: 用 Rust 实现的高速代码生成
- **Tree-sitter 解析**: 准确的 Dart 代码解析

## 安装

```bash
cargo install --path .
```

## 使用方法

### CLI 子命令

SuperFastGen 支持单独的生成命令以提供更好的控制：

```bash
# 仅生成Freezed代码
superfastgen generate --type freezed

# 仅生成JSON序列化
superfastgen generate --type json

# 仅生成Riverpod提供者
superfastgen generate --type riverpod

# 生成所有代码类型（freezed、json、riverpod）
superfastgen generate --type all

# 仅生成资源
superfastgen assets

# 生成所有内容（代码和资源）
superfastgen all

# 以监视模式运行（文件更改时自动重新生成）
superfastgen --watch
```

### 基本用法

```bash
# 运行代码生成器（生成所有类型）
cargo run

# 以监视模式运行（文件更改时自动重新生成）
cargo run -- --watch
```

### 配置

在项目根目录创建`superfastgen.yaml`文件来自定义配置：

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

1. **资源生成**：

   - 读取`pubspec.yaml`
   - 递归扫描资源目录
   - 生成`lib/gen/assets.gen.dart`

2. **代码生成**：

   - 扫描`lib/`中的 Dart 文件
   - 检测注解：`@freezed`、`@JsonSerializable`、`@riverpod`
   - 生成相应的`.g.dart`文件

3. **监视模式**：
   - 监视`lib/`和`pubspec.yaml`的更改
   - 文件修改时自动重新生成代码
   - 类似于`flutter pub run build_runner watch`

### 输出示例

运行`superfastgen generate --type freezed`后，您将得到：

```
Generating Freezed code from lib/ to lib/gen/...
Generated: lib/user.g.dart
Generated 1 .g.dart files for freezed
```

运行`superfastgen assets`后，您将得到：

```
Generating assets from assets/ to lib/gen/...
Generated assets.gen.dart with 6 asset constants
```

### 生成的文件

- `lib/user.g.dart` - Freezed 代码生成
- `lib/product.g.dart` - JSON 序列化
- `lib/provider.g.dart` - Riverpod 提供者
- `lib/gen/assets.gen.dart` - 资源常量

### 自定义路径

您可以指定自定义的输入和输出路径：

```bash
# 使用自定义路径
superfastgen generate --type freezed --input src/ --output generated/

# 使用自定义路径生成资源
superfastgen assets --assets my-assets/ --output lib/generated/
```

## 开发

### 安装依赖

```bash
cargo build
```

### 运行测试

```bash
cargo test
```

### 设置日志级别

```bash
RUST_LOG=debug cargo run -- generate --input lib/ --output lib/gen/
```

## 更新 Tree-sitter-dart

### 手动更新

要更新 tree-sitter-dart 子模块：

```bash
# 更新子模块到最新版本
git submodule update --remote tree-sitter-dart

# 提交更改
git add tree-sitter-dart
git commit -m "Update tree-sitter-dart to latest version"

# 重新构建项目
cargo clean
cargo build

# 运行测试验证
cargo test
```

### 自动更新

使用内置脚本进行自动更新：

```bash
# 运行脚本
cargo run --bin update-tree-sitter

# 或直接运行
./scripts/update-tree-sitter.sh
```

此脚本将自动执行：

- 更新 tree-sitter-dart 子模块
- 重新构建项目
- 运行测试
- 提交更改

## 许可证

MIT License

---

[English](README.md) | [日本語](README_ja.md) | [Tiếng Việt](README_vi.md) | [简体中文](README_zh_cn.md) | [繁體中文](README_zh_tw.md) | [한국어](README_ko.md) | [Français](README_fr.md)
