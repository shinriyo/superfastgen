# SuperFastGen

Rust로 작성된 고성능 Flutter/Dart 프로젝트 코드 생성기입니다.

## 기능

- **Freezed 코드 생성**: Freezed를 사용한 불변 데이터 클래스 생성
- **JSON 직렬화**: 자동 JSON 직렬화/역직렬화
- **Riverpod 통합**: Riverpod 제공자 및 상태 관리 생성
- **자산 관리**: 자산 파일 처리 및 생성
- **Tree-sitter 파싱**: 고급 코드 파싱 및 AST 조작
- **YAML 처리**: pubspec.yaml 파일 파싱 및 생성

## 프로젝트 구조

```
superfastgen/
├── Cargo.toml
├── src/
│   ├── main.rs              # 메인 진입점
│   ├── commands/
│   │   ├── mod.rs           # 모듈 정의
│   │   ├── generate.rs      # Freezed/JSON/Riverpod 생성
│   │   └── assets.rs        # 자산 생성기
│   └── utils/
│       ├── mod.rs           # 모듈 정의
│       ├── parser.rs        # Tree-sitter 파싱 유틸리티
│       └── yaml.rs          # pubspec.yaml 유틸리티
```

## 설치

```bash
# 저장소 클론
git clone https://github.com/shinriyo/superfastgen.git
cd superfastgen

# 서브모듈 초기화 (tree-sitter-dart용)
git submodule update --init

# 프로젝트 빌드
cargo build

# 애플리케이션 실행
cargo run
```

## 사용법

### 기본 사용법

```bash
# 코드 생성기 실행 (모든 타입 생성)
cargo run

# 감시 모드로 실행 (파일 변경 시 자동 재생성)
cargo run -- --watch
```

### 기능 설명

1. **자산 생성**:

   - `test_flutter_app/pubspec.yaml` 읽기
   - 자산 디렉토리 재귀적 스캔
   - `test_flutter_app/lib/gen/assets.gen.dart` 생성

2. **코드 생성**:

   - `test_flutter_app/lib/` 내 Dart 파일 스캔
   - `@freezed`, `@JsonSerializable`, `@riverpod` 어노테이션 감지
   - 해당하는 `.g.dart` 파일 생성

3. **감시 모드**:
   - `test_flutter_app/lib/` 및 `pubspec.yaml` 변경 감시
   - 파일 수정 시 자동으로 코드 재생성
   - `flutter pub run build_runner watch`와 유사

### 출력 예시

`cargo run` 실행 후 다음과 같이 출력됩니다:

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

### 생성된 파일

- `test_flutter_app/lib/user.g.dart` - Freezed 코드 생성
- `test_flutter_app/lib/product.g.dart` - JSON 직렬화
- `test_flutter_app/lib/provider.g.dart` - Riverpod 제공자
- `test_flutter_app/lib/gen/assets.gen.dart` - 자산 상수

### 사용자 정의 프로젝트 경로

자신의 Flutter 프로젝트에서 사용하려면:

```rust
// Rust 코드에서
use superfastgen::commands::assets;
use superfastgen::commands::generate;

// 프로젝트용 자산 생성
assets::generate_assets_from_path("your_flutter_project");

// 프로젝트용 코드 생성
generate::generate_freezed(); // 기본값은 "test_flutter_app/lib"
```

## 개발

### 요구사항

- Rust 1.70+
- Cargo

### 빌드

```bash
# 개발 빌드
cargo build

# 릴리스 빌드
cargo build --release

# 테스트 실행
cargo test
```

## 의존성

- `clap`: 명령줄 인수 파싱
- `serde`: 직렬화 프레임워크
- `serde_yaml`: YAML 직렬화
- `tree-sitter`: 코드 파싱
- `tera`: 템플릿 엔진

## 라이선스

MIT License

## 기여

1. 저장소 포크
2. 기능 브랜치 생성
3. 변경사항 작성
4. 해당하는 경우 테스트 추가
5. 풀 리퀘스트 제출

## 로드맵

- [x] 서브명령어가 있는 CLI 인터페이스
- [x] Freezed 코드 생성
- [x] JSON 직렬화
- [x] Riverpod 제공자 생성
- [x] 자산 처리
- [x] Tree-sitter 통합
- [x] YAML 구성
- [ ] 템플릿 시스템
- [ ] 플러그인 아키텍처
- [ ] 사용자 정의 경로용 CLI 인수
- [ ] 자동 재생성용 감시 모드
