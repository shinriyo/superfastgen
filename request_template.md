### **依頼: Rust 製コードジェネレーター `superfastgen` の出力修正**

**【最終目的】**

Rust で開発中のコードジェネレーター `superfastgen` が、Dart の標準的なコードジェネレーターである `build_runner` と完全に同一の `freezed` コードを生成できるように、`superfastgen` の Rust コードを修正する。

**【プロジェクト構成】**

- **Rust プロジェクト (本体):** `/superfastgen`
  - `superfastgen` のソースコード (`src/`) が格納されている。
  - `cargo` コマンドは、必ずこのディレクトリで実行する。
- **Flutter プロジェクト (実験場):** `/superfastgen/test_flutter_app/aminomi`
  - コード生成の対象となる Dart のモデルファイル (`lib/models/`) が置かれている。
  - `fvm`, `dart`, `flutter` コマンドは、必ずこのディレクトリで実行する。

**【比較と修正のためのワークフロー】**

`superfastgen` と `build_runner` は、どちらも `lib/models/` ディレクトリに生成物を書き込むため、比較には以下の手順を踏む必要があります。

1.  **`superfastgen` でコードを生成する**

    - 場所: `/superfastgen` (Rust プロジェクトルート)
    - コマンド:
      ```bash
      cargo run -- generate --type freezed --delete-conflicting-outputs test_flutter_app/aminomi/lib/models
      ```
    - 結果: `test_flutter_app/aminomi/lib/models/` 内の `*.freezed.dart` ファイルが `superfastgen` 製の内容で更新される。

2.  **`superfastgen` の生成物を比較用にコピーする**

    - 場所: `/superfastgen` (Rust プロジェクトルート)
    - コマンド例 (`event.dart` の場合):
      ```bash
      cp test_flutter_app/aminomi/lib/models/event.freezed.dart test_flutter_app/aminomi/lib/models/event.superfastgen.freezed.dart
      ```
    - 結果: `superfastgen` が生成したコードが、比較用の別名ファイルとして保存される。

3.  **`build_runner` で「正解」のコードを生成する**

    - 場所: `test_flutter_app/aminomi` (Flutter プロジェクトルート)
    - コマンド:
      ```bash
      cd test_flutter_app/aminomi
      fvm dart run build_runner build --delete-conflicting-outputs
      cd ../..
      ```
    - 結果: `test_flutter_app/aminomi/lib/models/` 内の `*.freezed.dart` ファイルが、`build_runner` 製の「正解」コードで上書きされる。

4.  **`diff` で差分を確認する**
    - 場所: `/superfastgen` (Rust プロジェクトルート)
    - コマンド例 (`event.dart` の場合):
      ```bash
      diff test_flutter_app/aminomi/lib/models/event.freezed.dart test_flutter_app/aminomi/lib/models/event.superfastgen.freezed.dart
      ```
    - 結果: 両者のコードの差分が出力される。差分がなければ、そのモデルは修正不要。

**【現在の状況 (2024/07/24)】**

- `payment.dart`: 差分なし。対応完了。
- `event.dart`: **差分あり。** 主な原因は、`List` 型プロパティの扱い方 (`superfastgen` が不変性を保証する `EqualUnmodifiableListView` を生成していない)。
- **次に行うべきこと:** `event.dart` の差分を解消するため、`superfastgen` の Rust コード (`src/commands/freezed_gen.rs` 等) を修正する。
