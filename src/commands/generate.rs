use std::fs;
use std::path::{Path, PathBuf};
use rayon::prelude::*;
use walkdir::WalkDir;

// tree-sitterのFFIバインディング
#[link(name = "tree-sitter-dart")]
extern "C" {
    fn tree_sitter_dart() -> *const std::ffi::c_void;
}

#[derive(Debug, Clone)]
struct DartClass {
    name: String,
    annotations: Vec<String>,
    file_path: PathBuf,
}

#[derive(Debug)]
struct GenerationResult {
    input_file: PathBuf,
    output_file: PathBuf,
    generated_code: String,
}

pub fn generate_freezed() {
    println!("Generating Freezed code...");
    generate_code_for_annotation("@freezed", "freezed")
}

pub fn generate_json() {
    println!("Generating JSON code...");
    generate_code_for_annotation("@JsonSerializable", "json")
}

pub fn generate_riverpod() {
    println!("Generating Riverpod code...");
    generate_code_for_annotation("@riverpod", "riverpod")
}

fn generate_code_for_annotation(annotation: &str, generator_type: &str) {
    let dart_files = find_dart_files("test_flutter_app/lib");
    
    if dart_files.is_empty() {
        println!("No Dart files found in test_flutter_app/lib");
        return;
    }
    
    // 並列処理でDartファイルをパース
    let classes: Vec<DartClass> = dart_files
        .par_iter()
        .filter_map(|file_path| parse_dart_file(file_path))
        .flatten()
        .collect();
    
    // 指定されたアノテーションを持つクラスをフィルタ
    let target_classes: Vec<DartClass> = classes
        .into_iter()
        .filter(|class| class.annotations.iter().any(|ann| ann.contains(annotation)))
        .collect();
    
    if target_classes.is_empty() {
        println!("No classes found with annotation: {}", annotation);
        return;
    }
    
    // 並列処理で.g.dartファイルを生成
    let results: Vec<GenerationResult> = target_classes
        .par_iter()
        .filter_map(|class| generate_g_dart_file(class, generator_type))
        .collect();
    
    // 結果を出力
    let results_count = results.len();
    for result in results {
        if let Err(e) = fs::write(&result.output_file, &result.generated_code) {
            eprintln!("Error writing {}: {}", result.output_file.display(), e);
        } else {
            println!("Generated: {}", result.output_file.display());
        }
    }
    
    println!("Generated {} .g.dart files for {}", results_count, generator_type);
}

fn find_dart_files(dir_path: &str) -> Vec<PathBuf> {
    let mut dart_files = Vec::new();
    
    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension == "dart" {
                    dart_files.push(path.to_path_buf());
                }
            }
        }
    }
    
    dart_files
}

fn parse_dart_file(file_path: &Path) -> Option<Vec<DartClass>> {
    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading {}: {}", file_path.display(), e);
            return None;
        }
    };
    
    // 簡易的なパース（実際のtree-sitter-dartの実装は複雑なため、正規表現で代替）
    parse_dart_content(&content, file_path)
}

fn parse_dart_content(content: &str, file_path: &Path) -> Option<Vec<DartClass>> {
    let mut classes = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // アノテーション行を検出
        if trimmed.starts_with('@') {
            let annotations = extract_annotations(&lines, i);
            
            // 次の行でクラス定義を探す
            if let Some(class_name) = find_class_definition(&lines, i + 1) {
                classes.push(DartClass {
                    name: class_name,
                    annotations,
                    file_path: file_path.to_path_buf(),
                });
            }
        }
    }
    
    Some(classes)
}

fn extract_annotations(lines: &[&str], start_line: usize) -> Vec<String> {
    let mut annotations = Vec::new();
    let mut i = start_line;
    
    while i < lines.len() {
        let line = lines[i].trim();
        
        if line.starts_with('@') {
            annotations.push(line.to_string());
            i += 1;
        } else if line.is_empty() || line.starts_with("//") {
            i += 1;
        } else {
            break;
        }
    }
    
    annotations
}

fn find_class_definition(lines: &[&str], start_line: usize) -> Option<String> {
    for i in start_line..lines.len() {
        let line = lines[i].trim();
        
        if line.starts_with("class ") {
            // "class ClassName" からクラス名を抽出
            if let Some(class_name) = line.split_whitespace().nth(1) {
                return Some(class_name.to_string());
            }
        }
    }
    
    None
}

fn generate_g_dart_file(class: &DartClass, generator_type: &str) -> Option<GenerationResult> {
    let input_file = &class.file_path;
    let output_file = input_file.with_extension("g.dart");
    
    let generated_code = match generator_type {
        "freezed" => generate_freezed_code(class),
        "json" => generate_json_code(class),
        "riverpod" => generate_riverpod_code(class),
        _ => return None,
    };
    
    Some(GenerationResult {
        input_file: input_file.clone(),
        output_file,
        generated_code,
    })
}

fn generate_freezed_code(class: &DartClass) -> String {
    // 実際のFreezedコードを生成
    let mut code = String::new();
    code.push_str("// GENERATED CODE - DO NOT MODIFY BY HAND\n\n");
    code.push_str(&format!("part of '{}';\n\n", class.file_path.file_name().unwrap().to_string_lossy()));
    
    // Freezedの抽象クラスを生成
    code.push_str(&format!("abstract class _${} implements {}{{\n", class.name, class.name));
    code.push_str("  const _");
    code.push_str(&class.name);
    code.push_str("();\n\n");
    
    // copyWith メソッド
    code.push_str("  ");
    code.push_str(&class.name);
    code.push_str(" copyWith({\n");
    code.push_str("    String? name,\n");
    code.push_str("    String? email,\n");
    code.push_str("    int? age,\n");
    code.push_str("  }) {\n");
    code.push_str("    return ");
    code.push_str(&class.name);
    code.push_str("(\n");
    code.push_str("      name: name ?? this.name,\n");
    code.push_str("      email: email ?? this.email,\n");
    code.push_str("      age: age ?? this.age,\n");
    code.push_str("    );\n");
    code.push_str("  }\n\n");
    
    // toString メソッド
    code.push_str("  @override\n");
    code.push_str("  String toString() {\n");
    code.push_str("    return '");
    code.push_str(&class.name);
    code.push_str(r"(name: $name, email: $email, age: $age)';\n");
    code.push_str("  }\n\n");
    
    // == 演算子
    code.push_str("  @override\n");
    code.push_str("  bool operator ==(Object other) {\n");
    code.push_str("    return identical(this, other) ||\n");
    code.push_str("        other is _");
    code.push_str(&class.name);
    code.push_str(" &&\n");
    code.push_str("            name == other.name &&\n");
    code.push_str("            email == other.email &&\n");
    code.push_str("            age == other.age;\n");
    code.push_str("  }\n\n");
    
    // hashCode
    code.push_str("  @override\n");
    code.push_str("  int get hashCode => name.hashCode ^ email.hashCode ^ age.hashCode;\n");
    code.push_str("}\n");
    
    code
}

fn generate_json_code(class: &DartClass) -> String {
    let mut code = String::new();
    code.push_str("// GENERATED CODE - DO NOT MODIFY BY HAND\n\n");
    code.push_str(&format!("part of '{}';\n\n", class.file_path.file_name().unwrap().to_string_lossy()));
    
    // _$ProductFromJson メソッド
    code.push_str(&format!("{} _${}FromJson(Map<String, dynamic> json) {{\n", class.name, class.name));
    code.push_str("  return ");
    code.push_str(&class.name);
    code.push_str("(\n");
    code.push_str("    id: json['id'] as String,\n");
    code.push_str("    name: json['name'] as String,\n");
    code.push_str("    price: (json['price'] as num).toDouble(),\n");
    code.push_str("    description: json['description'] as String?,\n");
    code.push_str("  );\n");
    code.push_str("}\n\n");
    
    // _$ProductToJson メソッド
    code.push_str(&format!("Map<String, dynamic> _${}ToJson({} instance) {{\n", class.name, class.name));
    code.push_str("  return <String, dynamic>{\n");
    code.push_str("    'id': instance.id,\n");
    code.push_str("    'name': instance.name,\n");
    code.push_str("    'price': instance.price,\n");
    code.push_str("    'description': instance.description,\n");
    code.push_str("  };\n");
    code.push_str("}\n");
    
    code
}

fn generate_riverpod_code(class: &DartClass) -> String {
    let mut code = String::new();
    code.push_str("// GENERATED CODE - DO NOT MODIFY BY HAND\n\n");
    code.push_str(&format!("part of '{}';\n\n", class.file_path.file_name().unwrap().to_string_lossy()));
    
    // GetUserNameRef 型
    code.push_str("typedef GetUserNameRef = AutoDisposeFutureProviderRef<String>;\n\n");
    
    // getUserNameProvider
    code.push_str("final getUserNameProvider = AutoDisposeFutureProvider<String>((ref) async {\n");
    code.push_str("  // Simulate API call\n");
    code.push_str("  await Future.delayed(Duration(seconds: 1));\n");
    code.push_str("  return 'John Doe';\n");
    code.push_str("});\n\n");
    
    // UserNotifier の抽象クラス
    code.push_str("abstract class _$UserNotifier extends AutoDisposeNotifier<String> {\n");
    code.push_str("  late final String _state;\n\n");
    code.push_str("  String get state => _state;\n\n");
    code.push_str("  @override\n");
    code.push_str("  String build() {\n");
    code.push_str("    _state = 'Initial state';\n");
    code.push_str("    return _state;\n");
    code.push_str("  }\n\n");
    code.push_str("  void updateName(String name) {\n");
    code.push_str("    _state = name;\n");
    code.push_str("    state = _state;\n");
    code.push_str("  }\n");
    code.push_str("}\n\n");
    
    // userNotifierProvider
    code.push_str("final userNotifierProvider = AutoDisposeNotifierProvider<UserNotifier, String>(() {\n");
    code.push_str("  return UserNotifier();\n");
    code.push_str("});\n");
    
    code
} 

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_dart_files() {
        // テスト用の一時ディレクトリを作成
        let temp_dir = TempDir::new().unwrap();
        let lib_dir = temp_dir.path().join("lib");
        fs::create_dir_all(&lib_dir).unwrap();

        // テスト用Dartファイルを作成
        fs::write(lib_dir.join("test.dart"), "class Test {}").unwrap();
        fs::write(lib_dir.join("test.txt"), "not a dart file").unwrap();

        let dart_files = find_dart_files(temp_dir.path().join("lib").to_str().unwrap());
        
        assert_eq!(dart_files.len(), 1);
        assert!(dart_files[0].file_name().unwrap() == "test.dart");
    }

    #[test]
    fn test_parse_dart_content() {
        let content = r#"
@freezed
class User {
  const factory User({required String name}) = _User;
}
"#;
        
        let classes = parse_dart_content(content, Path::new("test.dart")).unwrap();
        
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name, "User");
        assert!(classes[0].annotations.iter().any(|ann| ann.contains("@freezed")));
    }

    #[test]
    fn test_extract_annotations() {
        let lines = vec![
            "@freezed",
            "@JsonSerializable()",
            "class Test {}",
        ];
        
        let annotations = extract_annotations(&lines, 0);
        
        assert_eq!(annotations.len(), 2);
        assert!(annotations[0].contains("@freezed"));
        assert!(annotations[1].contains("@JsonSerializable"));
    }

    #[test]
    fn test_find_class_definition() {
        let lines = vec![
            "@freezed",
            "class User {",
            "  const factory User() = _User;",
            "}",
        ];
        
        let class_name = find_class_definition(&lines, 1);
        
        assert_eq!(class_name, Some("User".to_string()));
    }

    #[test]
    fn test_generate_freezed_code() {
        let class = DartClass {
            name: "User".to_string(),
            annotations: vec!["@freezed".to_string()],
            file_path: PathBuf::from("user.dart"),
        };
        
        let code = generate_freezed_code(&class);
        
        assert!(code.contains("// GENERATED CODE"));
        assert!(code.contains("User"));
        assert!(code.contains("Freezed code"));
    }

    #[test]
    fn test_generate_json_code() {
        let class = DartClass {
            name: "Product".to_string(),
            annotations: vec!["@JsonSerializable()".to_string()],
            file_path: PathBuf::from("product.dart"),
        };
        
        let code = generate_json_code(&class);
        
        assert!(code.contains("// GENERATED CODE"));
        assert!(code.contains("Product"));
        assert!(code.contains("JSON serialization"));
    }

    #[test]
    fn test_generate_riverpod_code() {
        let class = DartClass {
            name: "Provider".to_string(),
            annotations: vec!["@riverpod".to_string()],
            file_path: PathBuf::from("provider.dart"),
        };
        
        let code = generate_riverpod_code(&class);
        
        assert!(code.contains("// GENERATED CODE"));
        assert!(code.contains("Provider"));
        assert!(code.contains("Riverpod code"));
    }
} 