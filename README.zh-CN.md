# SuperFastGen

用 Rust 编写的高性能 Flutter/Dart 项目代码生成器。

## 功能特性

- **Freezed 代码生成**: 使用 Freezed 生成不可变数据类
- **JSON 序列化**: 自动 JSON 序列化/反序列化
- **Riverpod 集成**: 生成 Riverpod 提供者和状态管理
- **资源管理**: 处理和生成资源文件
- **Tree-sitter 解析**: 高级代码解析和 AST 操作
- **YAML 处理**: 解析和生成 pubspec.yaml 文件

## 项目结构

```
superfastgen/
├── Cargo.toml
├── src/
│   ├── main.rs              # 主入口点
│   ├── commands/
│   │   ├── mod.rs           # 模块定义
│   │   ├── generate.rs      # Freezed/JSON/Riverpod生成
│   │   └── assets.rs        # 资源生成器
│   └── utils/
│       ├── mod.rs           # 模块定义
│       ├── parser.rs        # Tree-sitter解析工具
│       └── yaml.rs          # pubspec.yaml工具
```

## 安装

```bash
# 克隆仓库
git clone https://github.com/shinriyo/superfastgen.git
cd superfastgen

# 初始化子模块（用于tree-sitter-dart）
git submodule update --init

# 构建项目
cargo build

# 运行应用
cargo run
```

## 使用方法

### 基本用法

```bash
# 运行代码生成器（生成所有类型）
cargo run

# 以监视模式运行（文件更改时自动重新生成）
cargo run -- --watch
```

### 功能说明

1. **资源生成**:

   - 读取 `test_flutter_app/pubspec.yaml`
   - 递归扫描资源目录
   - 生成 `test_flutter_app/lib/gen/assets.gen.dart`

2. **代码生成**:

   - 扫描 `test_flutter_app/lib/` 中的 Dart 文件
   - 检测注解: `@freezed`, `@JsonSerializable`, `@riverpod`
   - 生成相应的 `.g.dart` 文件

3. **监视模式**:
   - 监视 `test_flutter_app/lib/` 和 `pubspec.yaml` 的更改
   - 文件修改时自动重新生成代码
   - 类似于 `flutter pub run build_runner watch`

### 输出示例

运行 `cargo run` 后，您将看到:

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

### 生成的文件

- `test_flutter_app/lib/user.g.dart` - Freezed 代码生成
- `test_flutter_app/lib/product.g.dart` - JSON 序列化
- `test_flutter_app/lib/provider.g.dart` - Riverpod 提供者
- `test_flutter_app/lib/gen/assets.gen.dart` - 资源常量

### 自定义项目路径

要在您自己的 Flutter 项目中使用:

```rust
// 在您的Rust代码中
use superfastgen::commands::assets;
use superfastgen::commands::generate;

// 为您的项目生成资源
assets::generate_assets_from_path("your_flutter_project");

// 为您的项目生成代码
generate::generate_freezed(); // 默认使用 "test_flutter_app/lib"
```

## 开发

### 前置要求

- Rust 1.70+
- Cargo

### 构建

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 运行测试
cargo test
```

## 依赖项

- `clap`: 命令行参数解析
- `serde`: 序列化框架
- `serde_yaml`: YAML 序列化
- `tree-sitter`: 代码解析
- `tera`: 模板引擎

## 许可证

MIT License

## 贡献

1. Fork 仓库
2. 创建功能分支
3. 进行更改
4. 如果适用，添加测试
5. 提交拉取请求

## 路线图

- [x] 带子命令的 CLI 接口
- [x] Freezed 代码生成
- [x] JSON 序列化
- [x] Riverpod 提供者生成
- [x] 资源处理
- [x] Tree-sitter 集成
- [x] YAML 配置
- [ ] 模板系统
- [ ] 插件架构
- [ ] 自定义路径的 CLI 参数
- [ ] 自动重新生成的监视模式
