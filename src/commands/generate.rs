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
    format!(
        "// GENERATED CODE - DO NOT MODIFY BY HAND\n\n\
         part of '{}';\n\n\
         // Freezed code for {}\n\
         // TODO: Implement actual Freezed generation logic\n",
        class.file_path.file_name().unwrap().to_string_lossy(),
        class.name
    )
}

fn generate_json_code(class: &DartClass) -> String {
    format!(
        "// GENERATED CODE - DO NOT MODIFY BY HAND\n\n\
         part of '{}';\n\n\
         // JSON serialization code for {}\n\
         // TODO: Implement actual JSON generation logic\n",
        class.file_path.file_name().unwrap().to_string_lossy(),
        class.name
    )
}

fn generate_riverpod_code(class: &DartClass) -> String {
    format!(
        "// GENERATED CODE - DO NOT MODIFY BY HAND\n\n\
         part of '{}';\n\n\
         // Riverpod code for {}\n\
         // TODO: Implement actual Riverpod generation logic\n",
        class.file_path.file_name().unwrap().to_string_lossy(),
        class.name
    )
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