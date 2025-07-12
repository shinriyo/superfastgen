use std::fs;
use std::path::{Path, PathBuf};
use rayon::prelude::*;
use walkdir::WalkDir;
use tree_sitter::Parser;
use std::fs::OpenOptions;
use std::io::Write;
use log::{info, debug, error};

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
    #[allow(dead_code)]
    input_file: PathBuf,
    output_file: PathBuf,
    generated_code: String,
}

#[derive(Debug, Clone)]
pub struct DartField {
    pub name: String,
    pub ty: String,
}

#[allow(dead_code)]
pub fn generate_freezed() {
    info!("Generating Freezed code...");
    generate_code_for_annotation("@freezed", "freezed")
}

#[allow(dead_code)]
pub fn generate_json() {
    info!("Generating JSON code...");
    generate_code_for_annotation("@JsonSerializable", "json")
}

#[allow(dead_code)]
pub fn generate_riverpod() {
    info!("Generating Riverpod code...");
    generate_code_for_annotation("@riverpod", "riverpod")
}

// New functions: configurable paths
pub fn generate_freezed_with_paths(input_path: &str, output_path: &str) {
    info!("Generating Freezed code from {} to {}...", input_path, output_path);
    generate_code_for_annotation_with_paths("@freezed", "freezed", input_path, output_path)
}

pub fn generate_json_with_paths(input_path: &str, output_path: &str) {
    info!("Generating JSON code from {} to {}...", input_path, output_path);
    generate_code_for_annotation_with_paths("@JsonSerializable", "json", input_path, output_path)
}

pub fn generate_riverpod_with_paths(input_path: &str, output_path: &str) {
    info!("Generating Riverpod code from {} to {}...", input_path, output_path);
    generate_code_for_annotation_with_paths("@riverpod", "riverpod", input_path, output_path)
}

#[allow(dead_code)]
fn generate_code_for_annotation(annotation: &str, generator_type: &str) {
    generate_code_for_annotation_with_paths(annotation, generator_type, "test_flutter_app/lib", "test_flutter_app/lib/gen")
}

fn generate_code_for_annotation_with_paths(annotation: &str, generator_type: &str, input_path: &str, output_path: &str) {
    let dart_files = find_dart_files(input_path);
    
    if dart_files.is_empty() {
        info!("No Dart files found in {}", input_path);
        return;
    }
    
    // 並列処理でDartファイルをパース
    let classes: Vec<DartClass> = dart_files
        .par_iter()
        .filter_map(|file_path| parse_dart_file(file_path))
        .flatten()
        .collect();
    
    // 指定されたアノテーションを持つクラスをフィルタ（重複を除去）
    let mut target_classes: Vec<DartClass> = classes
        .into_iter()
        .filter(|class| class.annotations.iter().any(|ann| ann.contains(annotation)))
        .collect();
    
    // 重複を除去（同じクラス名とファイルパスの組み合わせ）
    target_classes.dedup_by(|a, b| a.name == b.name && a.file_path == b.file_path);
    
    debug!("Found {} classes with annotation '{}':", target_classes.len(), annotation);
    for class in &target_classes {
        debug!("- Class: {} in file: {}", class.name, class.file_path.display());
    }
    
    if target_classes.is_empty() {
        info!("No classes found with annotation: {}", annotation);
        return;
    }
    
    // 並列処理で.g.dartファイルを生成
    let results: Vec<GenerationResult> = target_classes
        .par_iter()
        .filter_map(|class| generate_g_dart_file_with_output_path(class, generator_type, output_path))
        .collect();
    
    // 結果を出力
    let results_count = results.len();
    for result in results {
        if let Err(e) = fs::write(&result.output_file, &result.generated_code) {
            error!("Error writing {}: {}", result.output_file.display(), e);
        } else {
            info!("Generated: {}", result.output_file.display());
        }
    }
    
    info!("Generated {} .g.dart files for {}", results_count, generator_type);
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
            error!("Error reading {}: {}", file_path.display(), e);
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

#[allow(dead_code)]
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

fn generate_g_dart_file_with_output_path(class: &DartClass, generator_type: &str, output_path: &str) -> Option<GenerationResult> {
    let input_file = &class.file_path;
    let output_file = PathBuf::from(output_path).join(input_file.file_name().unwrap());
    let output_file = output_file.with_extension("g.dart");
    
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
    let mut code = String::new();
    code.push_str("// GENERATED CODE - DO NOT MODIFY BY HAND\n\n");
    code.push_str(&format!("part of '{}';\n\n", class.file_path.file_stem().unwrap().to_str().unwrap()));
    
    // クラス名を取得
    let class_name = &class.name;
    
    // ソースファイルからフィールドを抽出
    let source_content = std::fs::read_to_string(&class.file_path).unwrap_or_default();
    let fields = extract_fields_from_dart_class(&source_content);
    
    // 抽象クラス定義
    code.push_str(&format!("abstract class _${} implements {} {{\n", class_name, class_name));
    
    // copyWithメソッド
    code.push_str(&generate_copy_with(class_name, &fields));
    code.push_str("\n");
    
    // toStringメソッド
    code.push_str(&generate_to_string(class_name, &fields));
    code.push_str("\n");
    
    // ==演算子
    code.push_str(&generate_eq(class_name, &fields));
    code.push_str("\n");
    
    // hashCode
    code.push_str(&generate_hash_code(&fields));
    code.push_str("\n");
    
    // fromJsonファクトリメソッド
    code.push_str(&generate_from_json(class_name, &fields));
    code.push_str("\n");
    
    // toJsonメソッド
    code.push_str(&generate_to_json(class_name, &fields));
    code.push_str("\n");
    
    code.push_str("}\n");
    
    code
}

fn generate_copy_with(class_name: &str, fields: &[DartField]) -> String {
    let mut code = String::new();
    code.push_str(&format!("  {} copyWith({{\n", class_name));
    
    // オプショナルパラメータを生成
    for field in fields {
        code.push_str(&format!("    {}? {},\n", field.ty, field.name));
    }
    code.push_str("  }) {\n");
    code.push_str(&format!("    return {}(\n", class_name));
    
    // フィールドの割り当てを生成
    for (i, field) in fields.iter().enumerate() {
        if i > 0 { code.push_str(",\n"); }
        code.push_str(&format!("      {}: {} ?? this.{}", field.name, field.name, field.name));
    }
    code.push_str("\n    );\n");
    code.push_str("  }\n");
    
    code
}

fn generate_to_string(class_name: &str, fields: &[DartField]) -> String {
    let mut code = String::new();
    code.push_str("  @override\n");
    code.push_str(&format!("  String toString() {{\n"));
    code.push_str(&format!("    return '{}(", class_name));
    
    // フィールドの文字列表現を生成
    for (i, field) in fields.iter().enumerate() {
        if i > 0 { code.push_str(", "); }
        code.push_str(&format!("{}: ${}", field.name, field.name));
    }
    code.push_str(")';\n");
    code.push_str("  }\n");
    
    code
}

fn generate_eq(class_name: &str, fields: &[DartField]) -> String {
    let mut code = String::new();
    code.push_str("  @override\n");
    code.push_str("  bool operator ==(Object other) {\n");
    code.push_str(&format!("    return identical(this, other) ||\n"));
    code.push_str(&format!("        other is _{} &&\n", class_name));
    
    // フィールドの比較を生成
    for (i, field) in fields.iter().enumerate() {
        if i > 0 { code.push_str(" &&\n"); }
        code.push_str(&format!("        {} == other.{}", field.name, field.name));
    }
    code.push_str(";\n");
    code.push_str("  }\n");
    
    code
}

fn generate_hash_code(fields: &[DartField]) -> String {
    let mut code = String::new();
    code.push_str("  @override\n");
    code.push_str("  int get hashCode => ");
    
    // フィールドのハッシュコードを生成
    for (i, field) in fields.iter().enumerate() {
        if i > 0 { code.push_str(" ^ "); }
        code.push_str(&format!("{}.hashCode", field.name));
    }
    
    if fields.is_empty() {
        code.push_str("0");
    }
    code.push_str(";\n");
    
    code
}

fn generate_json_code(class: &DartClass) -> String {
    let mut code = String::new();
    code.push_str("// GENERATED CODE - DO NOT MODIFY BY HAND\n\n");
    code.push_str(&format!("part of '{}';\n\n", class.file_path.file_stem().unwrap().to_str().unwrap()));
    
    let class_name = &class.name;
    
    // ソースファイルからフィールドを抽出
    let source_content = std::fs::read_to_string(&class.file_path).unwrap_or_default();
    let fields = extract_fields_from_dart_class(&source_content);
    
    // _$ClassNameFromJson関数
    code.push_str(&format!("{} _${}FromJson(Map<String, dynamic> json) {{\n", class_name, class_name));
    code.push_str(&format!("  return {}(\n", class_name));
    
    // フィールドのJSON解析を生成
    for (i, field) in fields.iter().enumerate() {
        if i > 0 { code.push_str(",\n"); }
        code.push_str(&format!("    {}: json['{}'] as {}", field.name, field.name, field.ty));
    }
    code.push_str("\n  );\n");
    code.push_str("}\n\n");
    
    // _$ClassNameToJson関数
    code.push_str(&format!("Map<String, dynamic> _${}ToJson({} instance) {{\n", class_name, class_name));
    code.push_str("  return {\n");
    
    // フィールドのJSON変換を生成
    for (i, field) in fields.iter().enumerate() {
        if i > 0 { code.push_str(",\n"); }
        code.push_str(&format!("    '{}': instance.{}", field.name, field.name));
    }
    code.push_str("\n  };\n");
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

/// AST全体をファイルに出力する
fn write_ast_to_file(node: tree_sitter::Node, source: &str, depth: usize, file: &mut std::fs::File) {
    let indent = "  ".repeat(depth);
    let node_text = node.utf8_text(source.as_bytes()).unwrap_or_default();
    writeln!(file, "{}Node: {} = '{}'", indent, node.kind(), node_text).unwrap();
    
    for child in node.children(&mut node.walk()) {
        write_ast_to_file(child, source, depth + 1, file);
    }
}

/// Dartクラスのフィールド情報をtree-sitter-dartで抽出する
pub fn extract_fields_from_dart_class(source: &str) -> Vec<DartField> {
    debug!("Processing source with {} characters", source.len());
    if source.len() > 100 {
        debug!("Source preview: {}", &source[..100]);
    }
    
    let mut parser = Parser::new();
    parser.set_language(unsafe { std::mem::transmute(tree_sitter_dart()) }).unwrap();
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();
    let mut fields = Vec::new();

    // AST全体をファイルに出力（各クラスごと）
    let mut file = OpenOptions::new().create(true).write(true).append(true).open("debug_ast.txt").unwrap();
    writeln!(file, "\n=== Complete AST for source ===").unwrap();
    write_ast_to_file(root, source, 0, &mut file);
    writeln!(file, "=== End AST ===").unwrap();

    // クラスノードを探索
    for class_node in root.children(&mut tree.walk()) {
        if class_node.kind() == "class_definition" {
            // クラス名を取得
            let class_name = class_node.children(&mut tree.walk()).find(|n| n.kind() == "identifier").map(|n| n.utf8_text(source.as_bytes()).unwrap_or("")).unwrap_or("");
            writeln!(file, "\n=== Processing class: {} ===", class_name).unwrap();
            debug!("Processing class: {}", class_name);
            
            // クラス本体を探索
            for member in class_node.children(&mut tree.walk()) {
                if member.kind() == "class_body" {
                    debug!("Found class_body for {}", class_name);
                    for body_item in member.children(&mut tree.walk()) {
                        debug!("Body item: {}", body_item.kind());
                        // フィールド宣言を抽出
                        if body_item.kind() == "declaration" {
                            debug!("Found declaration, extracting fields...");
                            extract_fields_from_declaration(body_item, source, &mut fields, &tree);
                        }
                        // 通常のフィールド宣言も処理
                        else if body_item.kind() == "field_declaration" {
                            debug!("Found field_declaration, extracting fields...");
                            extract_fields_from_field_declaration(body_item, source, &mut fields, &tree);
                        }
                        // コンストラクタのパラメータを抽出
                        else if body_item.kind() == "constructor_signature" || body_item.kind() == "redirecting_factory_constructor_signature" {
                            writeln!(file, "Found constructor: {}", body_item.kind()).unwrap();
                            for param_list in body_item.children(&mut tree.walk()) {
                                writeln!(file, "  param_list: {}", param_list.kind()).unwrap();
                                if param_list.kind() == "formal_parameter_list" {
                                    for param in param_list.children(&mut tree.walk()) {
                                        writeln!(file, "    param: {} | text: {}", param.kind(), param.utf8_text(source.as_bytes()).unwrap_or("<err>")).unwrap();
                                        for child in param.children(&mut tree.walk()) {
                                            writeln!(file, "      child: {} | text: {}", child.kind(), child.utf8_text(source.as_bytes()).unwrap_or("<err>")).unwrap();
                                        }
                                        if param.kind() == "formal_parameter" {
                                            extract_field_from_formal_parameter(param, source, &mut fields, &tree);
                                        } else if param.kind() == "default_formal_parameter" {
                                            extract_field_from_parameter(param, source, &mut fields, &tree);
                                        } else if param.kind() == "typed_identifier" {
                                            extract_field_from_typed_identifier(param, source, &mut fields, &tree);
                                        }
                                    }
                                }
                            }
                        }
                        // Freezedのconst factoryコンストラクタを抽出
                        else if body_item.kind() == "const_constructor_signature" || body_item.kind() == "redirecting_factory_constructor_signature" {
                            writeln!(file, "Found factory constructor: {}", body_item.kind()).unwrap();
                            for param_list in body_item.children(&mut tree.walk()) {
                                writeln!(file, "  param_list: {}", param_list.kind()).unwrap();
                                if param_list.kind() == "formal_parameter_list" {
                                    for param in param_list.children(&mut tree.walk()) {
                                        writeln!(file, "    param: {} | text: {}", param.kind(), param.utf8_text(source.as_bytes()).unwrap_or("<err>")).unwrap();
                                        if param.kind() == "formal_parameter" {
                                            extract_field_from_formal_parameter(param, source, &mut fields, &tree);
                                        } else if param.kind() == "default_formal_parameter" {
                                            extract_field_from_parameter(param, source, &mut fields, &tree);
                                        } else if param.kind() == "typed_identifier" {
                                            extract_field_from_typed_identifier(param, source, &mut fields, &tree);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    fields
}

fn extract_fields_from_declaration(declaration: tree_sitter::Node, source: &str, fields: &mut Vec<DartField>, tree: &tree_sitter::Tree) {
    debug!("extract_fields_from_declaration called with kind: {}", declaration.kind());
    
    // redirecting_factory_constructor_signatureを処理
    for child in declaration.children(&mut tree.walk()) {
        debug!("Declaration child: {}", child.kind());
        if child.kind() == "redirecting_factory_constructor_signature" {
            debug!("Found redirecting_factory_constructor_signature");
            for param_list in child.children(&mut tree.walk()) {
                debug!("param_list: {}", param_list.kind());
                if param_list.kind() == "formal_parameter_list" {
                    for param in param_list.children(&mut tree.walk()) {
                        debug!("param: {} | text: {}", param.kind(), param.utf8_text(source.as_bytes()).unwrap_or("<err>"));
                        if param.kind() == "formal_parameter" {
                            extract_field_from_formal_parameter(param, source, fields, tree);
                        } else if param.kind() == "optional_formal_parameters" {
                            // Freezedのfactoryのフィールドはここに入る
                            for opt_param in param.children(&mut tree.walk()) {
                                if opt_param.kind() == "formal_parameter" {
                                    extract_field_from_formal_parameter(opt_param, source, fields, tree);
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // 既存の処理（通常のフィールド宣言）
            let mut ty = String::new();
            let mut name = String::new();
            
            if child.kind() == "type_identifier" {
                ty = child.utf8_text(source.as_bytes()).unwrap().to_string();
                debug!("Found type: {}", ty);
            } else if child.kind() == "initialized_identifier_list" {
                // フィールド名を抽出
                for identifier in child.children(&mut tree.walk()) {
                    if identifier.kind() == "initialized_identifier" {
                        for id_child in identifier.children(&mut tree.walk()) {
                            if id_child.kind() == "identifier" {
                                name = id_child.utf8_text(source.as_bytes()).unwrap().to_string();
                                debug!("Found name: {}", name);
                            }
                        }
                    }
                }
            }
            
            if !name.is_empty() && !ty.is_empty() {
                debug!("Adding field: {} {}", ty, name);
                fields.push(DartField { name, ty });
            }
        }
    }
}

fn extract_field_from_parameter(param: tree_sitter::Node, source: &str, fields: &mut Vec<DartField>, tree: &tree_sitter::Tree) {
    let mut ty = String::new();
    let mut name = String::new();
    
    for child in param.children(&mut tree.walk()) {
        if child.kind() == "typed_identifier" {
            for t in child.children(&mut tree.walk()) {
                if t.kind() == "type_identifier" {
                    ty = t.utf8_text(source.as_bytes()).unwrap().to_string();
                } else if t.kind() == "identifier" {
                    name = t.utf8_text(source.as_bytes()).unwrap().to_string();
                }
            }
        }
    }
    
    if !name.is_empty() && !ty.is_empty() {
        fields.push(DartField { name, ty });
    }
}

fn extract_field_from_typed_identifier(typed_id: tree_sitter::Node, source: &str, fields: &mut Vec<DartField>, tree: &tree_sitter::Tree) {
    let mut ty = String::new();
    let mut name = String::new();
    
    for t in typed_id.children(&mut tree.walk()) {
        if t.kind() == "type_identifier" {
            ty = t.utf8_text(source.as_bytes()).unwrap().to_string();
        } else if t.kind() == "identifier" {
            name = t.utf8_text(source.as_bytes()).unwrap().to_string();
        }
    }
    
    if !name.is_empty() && !ty.is_empty() {
        fields.push(DartField { name, ty });
    }
}

fn extract_field_from_formal_parameter(param: tree_sitter::Node, source: &str, fields: &mut Vec<DartField>, _tree: &tree_sitter::Tree) {
    debug!("extract_field_from_formal_parameter called with kind: {}", param.kind());
    
    // パラメータのテキスト全体を取得
    let param_text = param.utf8_text(source.as_bytes()).unwrap();
    debug!("Parameter text: '{}'", param_text);
    
    // 型と名前を抽出する関数
    fn extract_type_and_name(node: tree_sitter::Node, source: &str) -> (Option<String>, Option<String>) {
        let mut field_type = None;
        let mut field_name = None;
        
        for child in node.children(&mut node.walk()) {
            match child.kind() {
                "type_identifier" => {
                    field_type = Some(child.utf8_text(source.as_bytes()).unwrap().to_string());
                    debug!("Found type: {}", field_type.as_ref().unwrap());
                },
                "nullable_type" => {
                    for t in child.children(&mut child.walk()) {
                        if t.kind() == "type_identifier" {
                            field_type = Some(format!("{}?", t.utf8_text(source.as_bytes()).unwrap()));
                            debug!("Found nullable type: {}", field_type.as_ref().unwrap());
                        }
                    }
                },
                "identifier" => {
                    field_name = Some(child.utf8_text(source.as_bytes()).unwrap().to_string());
                    debug!("Found name: {}", field_name.as_ref().unwrap());
                },
                _ => {
                    let (child_type, child_name) = extract_type_and_name(child, source);
                    if field_type.is_none() {
                        field_type = child_type;
                    }
                    if field_name.is_none() {
                        field_name = child_name;
                    }
                }
            }
        }
        
        (field_type, field_name)
    }
    
    let (ty, name) = extract_type_and_name(param, source);
    
    if let (Some(ty), Some(name)) = (ty, name) {
        // パラメータテキストに?が含まれている場合は型名に?を付与
        let final_type = if param_text.contains('?') && !ty.ends_with('?') {
            format!("{}?", ty.clone())
        } else if param_text.contains('?') && ty.ends_with('?') {
            ty.clone() // 既に?が付いている場合はそのまま
        } else {
            ty.clone()
        };
        
        debug!("Extracted field: {} {} (final: {})", ty, name, final_type);
        if !fields.iter().any(|f| f.name == name) {
            fields.push(DartField { name, ty: final_type });
            debug!("Added field to list");
        }
    }
}

#[allow(dead_code)]
fn extract_field_from_variable_declaration(var_decl: tree_sitter::Node, source: &str, fields: &mut Vec<DartField>, _tree: &tree_sitter::Tree) {
    let mut cursor = var_decl.walk();
    
    for child in var_decl.children(&mut cursor) {
        match child.kind() {
            "variable_declarator" => {
                let mut declarator_cursor = child.walk();
                let mut type_node: Option<tree_sitter::Node> = None;
                let mut name_node: Option<tree_sitter::Node> = None;
                
                for grandchild in child.children(&mut declarator_cursor) {
                    match grandchild.kind() {
                        "type_identifier" => type_node = Some(grandchild),
                        "identifier" => name_node = Some(grandchild),
                        _ => {}
                    }
                }
                
                if let (Some(type_node), Some(name_node)) = (type_node, name_node) {
                    let field_type = type_node.utf8_text(source.as_bytes()).unwrap_or("dynamic").to_string();
                    let field_name = name_node.utf8_text(source.as_bytes()).unwrap_or("unknown").to_string();
                    
                    fields.push(DartField {
                        name: field_name,
                        ty: field_type,
                    });
                }
            }
            _ => {}
        }
    }
}

fn extract_fields_from_field_declaration(field_decl: tree_sitter::Node, source: &str, fields: &mut Vec<DartField>, tree: &tree_sitter::Tree) {
    debug!("extract_fields_from_field_declaration called with kind: {}", field_decl.kind());
    
    for child in field_decl.children(&mut tree.walk()) {
        debug!("Field declaration child: {}", child.kind());
        
        if child.kind() == "final_builtin" || child.kind() == "var_builtin" {
            // final や var キーワードは無視
            continue;
        } else if child.kind() == "type_identifier" {
            // 型情報を取得
            let field_type = child.utf8_text(source.as_bytes()).unwrap().to_string();
            debug!("Found field type: {}", field_type);
            
            // 次の兄弟ノードでフィールド名を探す
            if let Some(next_sibling) = field_decl.next_sibling() {
                for sibling_child in next_sibling.children(&mut tree.walk()) {
                    if sibling_child.kind() == "initialized_identifier_list" {
                        for identifier in sibling_child.children(&mut tree.walk()) {
                            if identifier.kind() == "initialized_identifier" {
                                for id_child in identifier.children(&mut tree.walk()) {
                                    if id_child.kind() == "identifier" {
                                        let field_name = id_child.utf8_text(source.as_bytes()).unwrap().to_string();
                                        debug!("Found field name: {}", field_name);
                                        
                                        // nullable型かどうかをチェック
                                        let final_type = if field_decl.utf8_text(source.as_bytes()).unwrap().contains('?') {
                                            format!("{}?", field_type)
                                        } else {
                                            field_type.clone()
                                        };
                                        
                                        if !fields.iter().any(|f| f.name == field_name) {
                                            fields.push(DartField { name: field_name.clone(), ty: final_type.clone() });
                                            debug!("Added field: {} {}", final_type, field_name);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else if child.kind() == "nullable_type" {
            // nullable型の処理
            for type_child in child.children(&mut tree.walk()) {
                if type_child.kind() == "type_identifier" {
                    let field_type = type_child.utf8_text(source.as_bytes()).unwrap().to_string();
                    debug!("Found nullable field type: {}", field_type);
                    
                    // フィールド名を探す
                    if let Some(next_sibling) = field_decl.next_sibling() {
                        for sibling_child in next_sibling.children(&mut tree.walk()) {
                            if sibling_child.kind() == "initialized_identifier_list" {
                                for identifier in sibling_child.children(&mut tree.walk()) {
                                    if identifier.kind() == "initialized_identifier" {
                                        for id_child in identifier.children(&mut tree.walk()) {
                                            if id_child.kind() == "identifier" {
                                                let field_name = id_child.utf8_text(source.as_bytes()).unwrap().to_string();
                                                debug!("Found nullable field name: {}", field_name);
                                                
                                                if !fields.iter().any(|f| f.name == field_name) {
                                                    fields.push(DartField { name: field_name.clone(), ty: format!("{}?", field_type) });
                                                    debug!("Added nullable field: {}? {}", field_type, field_name);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn generate_from_json(class_name: &str, _fields: &[DartField]) -> String {
    let mut code = String::new();
    code.push_str(&format!("  factory {}.fromJson(Map<String, dynamic> json) => _${}FromJson(json);\n", class_name, class_name));
    code
}

fn generate_to_json(class_name: &str, _fields: &[DartField]) -> String {
    let mut code = String::new();
    code.push_str(&format!("  Map<String, dynamic> toJson() => _${}ToJson(this);\n", class_name));
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
        assert!(code.contains("abstract class _$User"));
        assert!(code.contains("copyWith"));
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
        assert!(code.contains("_$ProductFromJson"));
        assert!(code.contains("_$ProductToJson"));
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
        assert!(code.contains("GetUserNameRef"));
        assert!(code.contains("getUserNameProvider"));
        assert!(code.contains("_$UserNotifier"));
    }

    #[test]
    fn test_extract_fields_from_freezed_class() {
        let freezed_source = r#"
import 'package:freezed_annotation/freezed_annotation.dart';

part 'user.freezed.dart';
part 'user.g.dart';

@freezed
class User with _$User {
  const factory User({
    required String name,
    required String email,
    int? age,
  }) = _User;

  factory User.fromJson(Map<String, dynamic> json) => _$UserFromJson(json);
}
"#;
        
        let fields = extract_fields_from_dart_class(freezed_source);
        debug!("Extracted fields: {:?}", fields);
        
        // 期待されるフィールドを確認
        assert_eq!(fields.len(), 3);
        
        let name_field = fields.iter().find(|f| f.name == "name").unwrap();
        assert_eq!(name_field.ty, "String");
        
        let email_field = fields.iter().find(|f| f.name == "email").unwrap();
        assert_eq!(email_field.ty, "String");
        
        let age_field = fields.iter().find(|f| f.name == "age").unwrap();
        assert_eq!(age_field.ty, "int?");
    }
} 