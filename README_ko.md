# SuperFastGen

SuperFastGen은 Flutter/Dart 프로젝트용 고성능 코드 생성기입니다. tree-sitter-dart를 사용하여 Dart 코드를 파싱하고, Freezed, JSON 직렬화, Riverpod 제공자 코드 및 자산 관리를 지원합니다.

## 기능

- **Freezed 코드 생성**: 데이터 클래스와 불변 객체 생성
- **JSON 직렬화**: fromJson/toJson 메서드 자동 생성
- **Riverpod 제공자**: 상태 관리 제공자 생성
- **자산 관리**: 이미지, 폰트, 아이콘 자동 감지 및 관리
- **고성능**: Rust로 구현된 고속 코드 생성
- **Tree-sitter 파싱**: 정확한 Dart 코드 파싱

## 설치

```bash
cargo install --path .
```

## 사용법

### CLI 서브커맨드

SuperFastGen은 더 나은 제어를 위해 개별 생성 명령을 지원합니다:

```bash
# Freezed 코드만 생성
superfastgen generate --type freezed

# JSON 직렬화만 생성
superfastgen generate --type json

# Riverpod 제공자만 생성
superfastgen generate --type riverpod

# 모든 코드 타입 생성 (freezed, json, riverpod)
superfastgen generate --type all

# 자산만 생성
superfastgen assets

# 모든 것 생성 (코드와 자산)
superfastgen all

# 감시 모드로 실행 (파일 변경 시 자동 재생성)
superfastgen --watch
```

### 기본 사용법

```bash
# 코드 생성기 실행 (모든 타입 생성)
cargo run

# 감시 모드로 실행 (파일 변경 시 자동 재생성)
cargo run -- --watch
```

### 설정

프로젝트 루트에 `superfastgen.yaml` 파일을 생성하여 설정을 사용자 정의할 수 있습니다:

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

### 기능

1. **자산 생성**:

   - `pubspec.yaml` 읽기
   - 자산 디렉토리 재귀적으로 스캔
   - `lib/gen/assets.gen.dart` 생성

2. **코드 생성**:

   - `lib/`의 Dart 파일 스캔
   - 주석 감지: `@freezed`, `@JsonSerializable`, `@riverpod`
   - 해당하는 `.g.dart` 파일 생성

3. **감시 모드**:
   - `lib/`와 `pubspec.yaml`의 변경 사항 모니터링
   - 파일이 수정되면 자동으로 코드 재생성
   - `flutter pub run build_runner watch`와 유사

### 출력 예시

`superfastgen generate --type freezed`를 실행한 후:

```
Generating Freezed code from lib/ to lib/gen/...
Generated: lib/user.g.dart
Generated 1 .g.dart files for freezed
```

`superfastgen assets`를 실행한 후:

```
Generating assets from assets/ to lib/gen/...
Generated assets.gen.dart with 6 asset constants
```

### 생성된 파일

- `lib/user.g.dart` - Freezed 코드 생성
- `lib/product.g.dart` - JSON 직렬화
- `lib/provider.g.dart` - Riverpod 제공자
- `lib/gen/assets.gen.dart` - 자산 상수

### 사용자 정의 경로

사용자 정의 입력 및 출력 경로를 지정할 수 있습니다:

```bash
# 사용자 정의 경로 사용
superfastgen generate --type freezed --input src/ --output generated/

# 사용자 정의 경로로 자산 생성
superfastgen assets --assets my-assets/ --output lib/generated/
```

## 개발

### 의존성 설치

```bash
cargo build
```

### 테스트 실행

```bash
cargo test
```

### 로그 레벨 설정

```bash
RUST_LOG=debug cargo run -- generate --input lib/ --output lib/gen/
```

## Tree-sitter-dart 업데이트

### 수동 업데이트

tree-sitter-dart 서브모듈을 업데이트하려면:

```bash
# 서브모듈을 최신 상태로 업데이트
git submodule update --remote tree-sitter-dart

# 변경사항 커밋
git add tree-sitter-dart
git commit -m "Update tree-sitter-dart to latest version"

# 프로젝트 재빌드
cargo clean
cargo build

# 테스트 실행하여 확인
cargo test
```

### 자동 업데이트

내장 스크립트를 사용하여 자동 업데이트:

```bash
# 스크립트 실행
cargo run --bin update-tree-sitter

# 또는 직접 실행
./scripts/update-tree-sitter.sh
```

이 스크립트는 다음을 자동으로 실행합니다:

- tree-sitter-dart 서브모듈 업데이트
- 프로젝트 재빌드
- 테스트 실행
- 변경사항 커밋

## 라이선스

MIT License

---

[English](README.md) | [日本語](README_ja.md) | [Tiếng Việt](README_vi.md) | [简体中文](README_zh_cn.md) | [繁體中文](README_zh_tw.md) | [한국어](README_ko.md) | [Français](README_fr.md)
