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

### 基本代码生成

```bash
superfastgen generate --input lib/ --output lib/gen/
```

### 资源管理

```bash
superfastgen assets --input assets/ --output lib/gen/
```

### 详细选项

```bash
superfastgen generate \
  --input lib/ \
  --output lib/gen/ \
  --freezed \
  --json \
  --riverpod \
  --verbose
```

## 配置

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
