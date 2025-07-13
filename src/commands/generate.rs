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
    pub is_named: bool, // 追加
}

#[derive(Debug, Clone)]
pub struct DartFunction {
    pub name: String,
    pub return_type: String,
    pub parameters: Vec<DartField>,
    pub annotations: Vec<String>,
    pub file_path: PathBuf,
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

fn generate_code_for_annotation(annotation: &str, generator_type: &str) {
    // Flutterプロジェクトのルートを自動検出
    if let Some(project_root) = find_flutter_project_root() {
        let lib_path = project_root.join("lib");
        let lib_path_str = lib_path.to_string_lossy();
        
        info!("Using Flutter project: {}", project_root.display());
        info!("Lib directory: {}", lib_path_str);
        
        // 出力先はlibディレクトリと同じ場所（.g.dartファイルは元のファイルと同じディレクトリに）
        generate_code_for_annotation_with_paths(annotation, generator_type, &lib_path_str, &lib_path_str)
    } else {
        error!("No Flutter project found. Make sure you're in a directory with pubspec.yaml and lib/");
        std::process::exit(1);
    }
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
        .filter(|class| {
            let has_ann = class.annotations.iter().any(|ann| ann.contains(annotation));
            if has_ann {
                debug!("Class {} in {} has annotation {}", class.name, class.file_path.display(), annotation);
            } else {
                debug!("Class {} in {} does NOT have annotation {} (found: {:?})", class.name, class.file_path.display(), annotation, class.annotations);
            }
            has_ann
        })
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

fn find_flutter_project_root() -> Option<PathBuf> {
    let mut current_dir = std::env::current_dir().ok()?;
    
    // 上位ディレクトリを探索してpubspec.yamlを探す
    loop {
        let pubspec_path = current_dir.join("pubspec.yaml");
        let lib_path = current_dir.join("lib");
        
        if pubspec_path.exists() && lib_path.exists() {
            debug!("Found Flutter project root: {}", current_dir.display());
            return Some(current_dir);
        }
        
        // 親ディレクトリに移動
        if !current_dir.pop() {
            break;
        }
    }
    
    None
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
    let mut pending_annotations: Vec<String> = Vec::new();
    
    for line in lines.iter() {
        let trimmed = line.trim();
        if trimmed.starts_with('@') {
            pending_annotations.push(trimmed.to_string());
        } else if trimmed.starts_with("class ") || trimmed.starts_with("abstract class ") {
            // "class ClassName" or "abstract class ClassName" からクラス名を抽出
            let class_name = if trimmed.starts_with("abstract class ") {
                trimmed.split_whitespace().nth(2)
            } else {
                trimmed.split_whitespace().nth(1)
            };
            if let Some(class_name) = class_name {
                classes.push(DartClass {
                    name: class_name.to_string(),
                    annotations: pending_annotations.clone(),
                    file_path: file_path.to_path_buf(),
                });
            }
            pending_annotations.clear();
        } else if !trimmed.is_empty() && !trimmed.starts_with("//") {
            // Any non-empty, non-comment, non-annotation, non-class line clears pending annotations
            pending_annotations.clear();
        }
    }
    Some(classes)
}

fn extract_fields_from_declaration(declaration: tree_sitter::Node, source: &str, fields: &mut Vec<DartField>, tree: &tree_sitter::Tree) {
    debug!("extract_fields_from_declaration called with kind: {}", declaration.kind());
    
    // Handle normal Dart class field declarations
    let mut field_type: Option<String> = None;
    let mut field_names = Vec::new();
    
    // Get the full text of the declaration for nullable type detection
    let declaration_text = declaration.utf8_text(source.as_bytes()).unwrap();
    debug!("Declaration text: '{}'", declaration_text);
    
    for child in declaration.children(&mut tree.walk()) {
        debug!("Declaration child: {}", child.kind());
        if child.kind() == "type_identifier" {
            field_type = Some(child.utf8_text(source.as_bytes()).unwrap().to_string());
            debug!("Found type: {}", field_type.as_ref().unwrap());
        } else if child.kind() == "initialized_identifier_list" {
            // Extract field names from initialized_identifier_list
            for identifier in child.children(&mut tree.walk()) {
                if identifier.kind() == "initialized_identifier" {
                    for id_child in identifier.children(&mut tree.walk()) {
                        if id_child.kind() == "identifier" {
                            let name = id_child.utf8_text(source.as_bytes()).unwrap().to_string();
                            field_names.push(name.clone());
                            debug!("Found name: {}", name);
                        }
                    }
                }
            }
        }
    }
    
    // Create field pairs
    if let Some(ty) = field_type {
        debug!("Processing {} field names with type {}", field_names.len(), ty);
        for name in field_names {
            if !fields.iter().any(|f| f.name == name) {
                // Check for nullable type in the declaration text
                let final_type = if declaration_text.contains('?') && !ty.ends_with('?') {
                    format!("{}?", ty)
                } else {
                    ty.clone()
                };
                fields.push(DartField { name: name.clone(), ty: final_type.clone(), is_named: false });
                debug!("Added field: {} {}", final_type, name);
            }
        }
    } else {
        debug!("No field type found in declaration");
    }
    
    // Handle redirecting_factory_constructor_signature (existing logic)
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
                            // Freezed factory fields go here
                            for opt_param in param.children(&mut tree.walk()) {
                                if opt_param.kind() == "formal_parameter" {
                                    extract_field_from_formal_parameter(opt_param, source, fields, tree);
                                }
                            }
                        }
                    }
                }
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
        fields.push(DartField { name, ty, is_named: false });
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
        fields.push(DartField { name, ty, is_named: false });
    }
}

fn extract_field_from_formal_parameter(param: tree_sitter::Node, source: &str, fields: &mut Vec<DartField>, _tree: &tree_sitter::Tree) {
    debug!("extract_field_from_formal_parameter called with kind: {}", param.kind());
    
    // Get the full text of the parameter
    let param_text = param.utf8_text(source.as_bytes()).unwrap();
    debug!("Parameter text: '{}'", param_text);
    
    // Function to extract type and name
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
        // If the parameter text contains ?, add ? to the type name
        let final_type = if param_text.contains('?') && !ty.ends_with('?') {
            format!("{}?", ty.clone())
        } else if param_text.contains('?') && ty.ends_with('?') {
            ty.clone() // If it already has ?, keep it
        } else {
            ty.clone()
        };
        
        debug!("Extracted field: {} {} (final: {})", ty, name, final_type);
        if !fields.iter().any(|f| f.name == name) {
            fields.push(DartField { name, ty: final_type, is_named: false });
            debug!("Added field to list");
        }
    }
}

fn generate_g_dart_file_with_output_path(class: &DartClass, generator_type: &str, _output_path: &str) -> Option<GenerationResult> {
    let input_file = &class.file_path;
    
    // Set output path based on generator type
    let output_file = match generator_type {
        "freezed" => input_file.with_extension("freezed.dart"),
        _ => input_file.with_extension("g.dart"),
    };
    
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
    code.push_str(&format!("part of '{}';\n\n", class.file_path.file_name().unwrap().to_string_lossy()));
    
    // Get class name
    let class_name = &class.name;
    
    // Extract fields from source file
    let source_content = std::fs::read_to_string(&class.file_path).unwrap_or_default();
    let fields = extract_fields_from_dart_class(&source_content);
    
    // --- mixin ---
    code.push_str(&format!("mixin _${} {{\n", class_name));
    // copyWith signature only (no implementation)
    code.push_str(&format!("  {} copyWith({{", class_name));
    for (i, field) in fields.iter().enumerate() {
        if i > 0 { code.push_str(", "); }
        // Handle nullable types correctly
        let param_type = if field.ty.ends_with('?') {
            field.ty.clone()
        } else {
            format!("{}?", field.ty)
        };
        code.push_str(&format!("{} {}", param_type, field.name));
    }
    code.push_str("});\n");
    code.push_str("}\n\n");
    
    // --- class ---
    code.push_str(&format!("class _{} with _${} implements {} {{\n", class_name, class_name, class_name));
    // Field declarations
    for field in &fields {
        code.push_str(&format!("  final {} {};\n", field.ty, field.name));
    }
    code.push_str("\n");
    // const constructor
    code.push_str(&format!("  const _{}({{", class_name));
    for (i, field) in fields.iter().enumerate() {
        if i > 0 { code.push_str(", "); }
        // Only add required for non-nullable fields
        if field.ty.ends_with('?') {
            code.push_str(&format!("this.{}", field.name));
        } else {
            code.push_str(&format!("required this.{}", field.name));
        }
    }
    code.push_str("});\n\n");
    // copyWith implementation
    code.push_str(&format!("  @override\n  {} copyWith({{", class_name));
    for (i, field) in fields.iter().enumerate() {
        if i > 0 { code.push_str(", "); }
        // Handle nullable types correctly
        let param_type = if field.ty.ends_with('?') {
            field.ty.clone()
        } else {
            format!("{}?", field.ty)
        };
        code.push_str(&format!("{} {}", param_type, field.name));
    }
    code.push_str(&format!("}}) {{\n    return _{}(", class_name));
    for (i, field) in fields.iter().enumerate() {
        if i > 0 { code.push_str(", "); }
        code.push_str(&format!("{}: {} ?? this.{}", field.name, field.name, field.name));
    }
    code.push_str(");\n  }\n\n");
    // toString method
    code.push_str(&generate_to_string(class_name, &fields));
    code.push_str("\n");
    // == operator
    code.push_str(&generate_eq(class_name, &fields));
    code.push_str("\n");
    // hashCode
    code.push_str(&generate_hash_code(&fields));
    code.push_str("\n");
    // toJson method
    code.push_str(&generate_to_json(class_name, &fields));
    code.push_str("\n");
    code.push_str("}\n\n");
    
    // Note: JSON functions are generated in user.g.dart, not here
    
    code
}

fn generate_copy_with(class_name: &str, fields: &[DartField]) -> String {
    let mut code = String::new();
    code.push_str(&format!("  {} copyWith({{\n", class_name));
    
    // Generate optional parameters
    for field in fields {
        // Remove duplicate ? from nullable types
        let param_type = if field.ty.ends_with('?') {
            field.ty.clone()
        } else {
            format!("{}?", field.ty)
        };
        code.push_str(&format!("    {} {},\n", param_type, field.name));
    }
    code.push_str("  }) {\n");
    code.push_str(&format!("    return _{}(", class_name));
    
    // Generate field assignments
    for (i, field) in fields.iter().enumerate() {
        if i > 0 { code.push_str(", "); }
        code.push_str(&format!("{}: {} ?? this.{}", field.name, field.name, field.name));
    }
    code.push_str(");\n");
    code.push_str("  }\n");
    
    code
}

fn generate_to_string(class_name: &str, fields: &[DartField]) -> String {
    let mut code = String::new();
    code.push_str("  @override\n");
    code.push_str(&format!("  String toString() {{\n"));
    code.push_str(&format!("    return '{}(", class_name));
    
    // Generate string representation of fields
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
    
    // Generate field comparisons
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
    
    // Generate hashCode for fields
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
    code.push_str(&format!("part of '{}';\n\n", class.file_path.file_name().unwrap().to_string_lossy()));
    
    let class_name = &class.name;
    
    // ソースファイルからフィールドを抽出
    let source_content = std::fs::read_to_string(&class.file_path).unwrap_or_default();
    let fields = extract_fields_from_dart_class(&source_content);
    
    debug!("Extracted {} fields for class {}", fields.len(), class_name);
    for field in &fields {
        debug!("Field: {} {}", field.ty, field.name);
    }
    
    // _$ClassNameFromJson関数
    code.push_str(&format!("{} _${}FromJson(Map<String, dynamic> json) {{\n", class_name, class_name));
    code.push_str(&format!("  return {}(", class_name));
    
    // フィールドのJSON解析を生成
    for (i, field) in fields.iter().enumerate() {
        if i > 0 { code.push_str(", "); }
        code.push_str(&format!("{}: json['{}'] as {}", field.name, field.name, field.ty));
    }
    code.push_str(");\n");
    code.push_str("}\n\n");
    
    // _$ClassNameToJson関数
    code.push_str(&format!("Map<String, dynamic> _${}ToJson({} instance) {{\n", class_name, class_name));
    code.push_str("  return {\n");
    
    // フィールドのJSON変換を生成
    for (i, field) in fields.iter().enumerate() {
        if i > 0 { code.push_str(",\n"); }
        code.push_str(&format!("    '{}': (instance as dynamic).{}", field.name, field.name));
    }
    code.push_str("\n  };\n");
    code.push_str("}\n");
    
    code
}

fn generate_riverpod_code(class: &DartClass) -> String {
    let mut code = String::new();
    code.push_str("// GENERATED CODE - DO NOT MODIFY BY HAND\n\n");
    
    // Extract function and class information from source file
    let source_content = std::fs::read_to_string(&class.file_path).unwrap_or_default();
    let functions = extract_functions_from_dart_source(&source_content, &class.file_path);
    
    debug!("Found {} functions in {}", functions.len(), class.file_path.display());
    for function in &functions {
        debug!("Function: {} with annotations: {:?}", function.name, function.annotations);
    }
    
    code.push_str(&format!("part of '{}';\n\n", class.file_path.file_name().unwrap().to_string_lossy()));
    
    // Generate providers from functions with @riverpod annotation
    for function in &functions {
        if function.annotations.iter().any(|ann| ann.contains("@riverpod")) {
            debug!("Generating provider for function: {}", function.name);
            code.push_str(&generate_function_provider(function));
            code.push_str("\n");
        }
    }
    
    // Generate NotifierProvider for class types
    if class.annotations.iter().any(|ann| ann.contains("@riverpod")) {
        debug!("Generating NotifierProvider for class: {}", class.name);
        code.push_str(&generate_notifier_provider(class));
    }
    
    code
}

fn generate_function_provider(function: &DartFunction) -> String {
    let mut code = String::new();
    
    // Generate provider name from function name
    let provider_name = format!("{}Provider", function.name);
    
    debug!("Generating provider for function: {} with return_type: '{}'", function.name, function.return_type);
    
    // Determine appropriate provider type and extract the actual return type
    let (provider_type, actual_return_type) = if function.return_type.starts_with("Future<") {
        ("AutoDisposeFutureProvider", function.return_type.trim_start_matches("Future<").trim_end_matches(">").to_string())
    } else if function.return_type.starts_with("Stream<") {
        ("AutoDisposeStreamProvider", function.return_type.trim_start_matches("Stream<").trim_end_matches(">").to_string())
    } else {
        ("AutoDisposeProvider", function.return_type.clone())
    };
    
    // Check for family support (whether parameters exist beyond Ref)
    let has_family_parameters = function.parameters.iter()
        .any(|p| p.name != "ref" && p.ty != "Ref");
    
    debug!("Function {}: has_family_parameters = {}", function.name, has_family_parameters);
    
    if has_family_parameters {
        debug!("Generating family provider for function: {}", function.name);
        let family_params: Vec<_> = function.parameters.iter()
            .filter(|p| p.name != "ref" && p.ty != "Ref")
            .collect();
        
        debug!("Family params: {:?}", family_params);
        
        // For Future providers, we need to use the inner type for the provider
        let return_type = if function.return_type.starts_with("Future<") {
            let inner = &function.return_type[7..function.return_type.len()-1];
            inner.to_string()
        } else {
            function.return_type.clone()
        };
        
        // Use the correct provider type for family providers too
        let family_provider_type = if function.return_type.starts_with("Future<") {
            "AutoDisposeFutureProvider"
        } else if function.return_type.starts_with("Stream<") {
            "AutoDisposeStreamProvider"
        } else {
            "AutoDisposeProvider"
        };
        
        debug!("Function {}: original return_type = '{}', family_provider_type = '{}'", function.name, function.return_type, family_provider_type);
        
        // Generate parameter type - avoid tuples for multiple parameters
        let param_types: Vec<_> = family_params.iter().map(|p| p.ty.clone()).collect();
        let param_type = if param_types.len() == 1 {
            param_types[0].clone()
        } else {
            // For multiple parameters, use Map<String, dynamic>
            "Map<String, dynamic>".to_string()
        };
        
        debug!("Family generation - return_type: '{}', param_type: '{}'", return_type, param_type);
        
        // Debug: Print the exact format string being generated
        let format_str = format!("final {} = {}.family<{}, {}>((ref, params) {{\n", 
            provider_name, family_provider_type, return_type, param_type);
        debug!("Generated format string: '{}'", format_str);
        
        code.push_str(&format!("final {} = {}.family<{}, {}>((ref, params) {{\n", 
            provider_name, family_provider_type, return_type, param_type
        ));
        code.push_str(&format!("  return {}(ref", function.name));
        
        // Argument passing for family providers
        let mut positional_i = 0;
        let positional_count = family_params.iter().filter(|p| !p.is_named).count();
        for param in family_params {
            if param.is_named {
                code.push_str(&format!(", {}: params['{}']", param.name, param.name));
            } else {
                if positional_count == 1 {
                    code.push_str(", params");
                    break; // Only one positional param, so break after adding
                } else {
                    code.push_str(&format!(", params[{}]", positional_i));
                    positional_i += 1;
                }
            }
        }
        code.push_str(");\n");
        code.push_str("});\n");
    } else {
        // Regular provider
        code.push_str(&format!("final {} = {}<{}>((ref) {{\n", 
            provider_name, provider_type, actual_return_type
        ));
        code.push_str(&format!("  return {}(ref);\n", function.name));
        code.push_str("});\n");
    }
    
    code
}

fn generate_notifier_provider(class: &DartClass) -> String {
    let mut code = String::new();
    
    // Generate NotifierProvider
    let provider_name = format!("{}Provider", to_lower_camel_case(&class.name));
    code.push_str(&format!("final {} = NotifierProvider<{}, String>(() {{\n", 
        provider_name, class.name
    ));
    code.push_str(&format!("  return {}();\n", class.name));
    code.push_str("});\n");
    
    code
} 

fn to_lower_camel_case(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
    }
}

fn collect_used_types(functions: &[DartFunction]) -> std::collections::HashSet<String> {
    let mut types = std::collections::HashSet::new();
    
    for function in functions {
        // Add return type
        if function.return_type.contains("List<") {
            types.insert("List".to_string());
        }
        if function.return_type.contains("Map<") {
            types.insert("Map".to_string());
        }
        if function.return_type.contains("Set<") {
            types.insert("Set".to_string());
        }
        
        // Add parameter types
        for param in &function.parameters {
            if param.ty.contains("List<") {
                types.insert("List".to_string());
            }
            if param.ty.contains("Map<") {
                types.insert("Map".to_string());
            }
            if param.ty.contains("Set<") {
                types.insert("Set".to_string());
            }
        }
    }
    
    types
}

/// Output AST to file for debugging
fn write_ast_to_file(node: tree_sitter::Node, source: &str, depth: usize, file: &mut std::fs::File) {
    let indent = "  ".repeat(depth);
    let node_text = node.utf8_text(source.as_bytes()).unwrap_or_default();
    writeln!(file, "{}Node: {} = '{}'", indent, node.kind(), node_text).unwrap();
    
    for child in node.children(&mut node.walk()) {
        write_ast_to_file(child, source, depth + 1, file);
    }
}

/// Extract Dart function information using tree-sitter-dart
pub fn extract_functions_from_dart_source(source: &str, file_path: &Path) -> Vec<DartFunction> {
    debug!("Processing source with {} characters", source.len());
    if source.len() > 100 {
        debug!("Source preview: {}", &source[..100]);
    }
    
    let mut parser = Parser::new();
    parser.set_language(unsafe { std::mem::transmute(tree_sitter_dart()) }).unwrap();
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();
    let mut functions = Vec::new();

    // Output AST to file for debugging
    let mut file = OpenOptions::new().create(true).write(true).append(true).open("debug_ast.txt").unwrap();
    writeln!(file, "\n=== Complete AST for {} ===", file_path.display()).unwrap();
    write_ast_to_file(root, source, 0, &mut file);
    writeln!(file, "=== End AST ===").unwrap();

    // Recursively visit all nodes to find function signatures
    fn visit_functions_recursive(node: tree_sitter::Node, source: &str, file_path: &Path, functions: &mut Vec<DartFunction>) {
        if node.kind() == "function_signature" {
            // Extract function name
            let function_name = node.children(&mut node.walk()).find(|n| n.kind() == "identifier")
                .map(|n| n.utf8_text(source.as_bytes()).unwrap_or("").to_string())
                .unwrap_or_default();
            debug!("Found function: {}", function_name);

            // Collect annotations
            let mut annotations = Vec::new();
            let mut current_node = node;
            while let Some(prev_sibling) = current_node.prev_sibling() {
                if prev_sibling.kind() == "annotation" {
                    let annotation_text = prev_sibling.utf8_text(source.as_bytes()).unwrap_or("");
                    annotations.push(annotation_text.to_string());
                    debug!("Found annotation: {}", annotation_text);
                } else if !prev_sibling.kind().contains("comment") && !prev_sibling.kind().contains("whitespace") {
                    break;
                }
                current_node = prev_sibling;
            }

            // Extract return type
            let mut return_type = "dynamic".to_string();
            let mut base_type = "dynamic".to_string();
            let mut type_arguments = Vec::new();
            
            for child in node.children(&mut node.walk()) {
                if child.kind() == "type_identifier" {
                    base_type = child.utf8_text(source.as_bytes()).unwrap_or("dynamic").to_string();
                    debug!("Found base type: {}", base_type);
                } else if child.kind() == "type_arguments" {
                    // Extract type arguments like <String> from Future<String>
                    let args_text = child.utf8_text(source.as_bytes()).unwrap_or("");
                    debug!("Found type arguments: {}", args_text);
                    type_arguments.push(args_text.to_string());
                } else if child.kind() == "function_type" {
                    // Handle complex types like Future<String>
                    return_type = child.utf8_text(source.as_bytes()).unwrap_or("dynamic").to_string();
                    debug!("Found function type: {}", return_type);
                    break;
                }
            }
            
            // Construct the full type if we have base type and arguments
            if base_type != "dynamic" && !type_arguments.is_empty() {
                return_type = format!("{}{}", base_type, type_arguments.join(""));
                debug!("Constructed complex type: {}", return_type);
            } else if base_type != "dynamic" {
                return_type = base_type;
            }
            
            // Fallback: If we found a simple type, check if it's part of a complex type
            if return_type == "dynamic" {
                // Look for the full function signature to extract the complete return type
                let function_text = node.utf8_text(source.as_bytes()).unwrap_or("");
                debug!("Function text: '{}'", function_text);
                if function_text.contains("Future<") {
                    // Extract the full Future<Type> pattern
                    if let Some(start) = function_text.find("Future<") {
                        if let Some(end) = function_text[start..].find('>') {
                            let full_type = &function_text[start..start + end + 1];
                            return_type = full_type.to_string();
                            debug!("Extracted complex type: {}", return_type);
                        }
                    }
                } else if function_text.contains("List<") {
                    // Extract the full List<Type> pattern
                    if let Some(start) = function_text.find("List<") {
                        if let Some(end) = function_text[start..].find('>') {
                            let full_type = &function_text[start..start + end + 1];
                            return_type = full_type.to_string();
                            debug!("Extracted List type: {}", return_type);
                        }
                    }
                }
            }
            
            debug!("Final return type: {}", return_type);

            // Extract the clean return type for provider generation
            // If the return type is Future<T> or Stream<T>, extract T
            let clean_return_type = if return_type.starts_with("Future<") || return_type.starts_with("Stream<") {
                if let Some(start) = return_type.find('<') {
                    if let Some(end) = return_type.rfind('>') {
                        return_type[start + 1..end].trim().to_string()
                    } else {
                        return_type.clone()
                    }
                } else {
                    return_type.clone()
                }
            } else if return_type.starts_with("List<") {
                if let Some(end) = return_type.find('>') {
                    return_type[..=end].trim().to_string()
                } else {
                    return_type.clone()
                }
            } else if let Some(comma_pos) = return_type.find(", (") {
                return_type[..comma_pos].trim().to_string()
            } else {
                return_type.clone()
            };
            debug!("Original return_type: '{}', clean_return_type: '{}'", return_type, clean_return_type);
            // --- ここまで修正 ---

            // Extract parameters
            let mut parameters = Vec::new();
            for child in node.children(&mut node.walk()) {
                if child.kind() == "formal_parameter_list" {
                    for param in child.children(&mut node.walk()) {
                        debug!("Parameter node: {} | text: {}", param.kind(), param.utf8_text(source.as_bytes()).unwrap_or(""));
                        
                        if param.kind() == "formal_parameter" {
                            let param_text = param.utf8_text(source.as_bytes()).unwrap_or("");
                            debug!("Formal parameter: {}", param_text);
                            let mut param_type = "dynamic".to_string();
                            let mut param_name = "param".to_string();
                            let mut is_named = false;
                            for param_child in param.children(&mut node.walk()) {
                                if param_child.kind() == "type_identifier" {
                                    param_type = param_child.utf8_text(source.as_bytes()).unwrap_or("dynamic").to_string();
                                } else if param_child.kind() == "identifier" {
                                    param_name = param_child.utf8_text(source.as_bytes()).unwrap_or("param").to_string();
                                }
                            }
                            // Detect nullable types
                            if param_text.contains('?') && !param_type.ends_with('?') {
                                param_type.push('?');
                            }
                            parameters.push(DartField {
                                name: param_name,
                                ty: param_type,
                                is_named,
                            });
                        } else if param.kind() == "optional_formal_parameters" {
                            debug!("Found optional formal parameters");
                            // Handle named parameters like {required int page, int limit = 10}
                            for opt_param in param.children(&mut node.walk()) {
                                if opt_param.kind() == "formal_parameter" {
                                    let opt_param_text = opt_param.utf8_text(source.as_bytes()).unwrap_or("");
                                    debug!("Optional parameter: {}", opt_param_text);
                                    let mut param_type = "dynamic".to_string();
                                    let mut param_name = "param".to_string();
                                    let mut is_named = true;
                                    for opt_param_child in opt_param.children(&mut node.walk()) {
                                        match opt_param_child.kind() {
                                            "type_identifier" => {
                                                param_type = opt_param_child.utf8_text(source.as_bytes()).unwrap_or("dynamic").to_string();
                                            },
                                            "identifier" => {
                                                param_name = opt_param_child.utf8_text(source.as_bytes()).unwrap_or("param").to_string();
                                            },
                                            _ => {}
                                        }
                                    }
                                    if opt_param_text.contains('?') && !param_type.ends_with('?') {
                                        param_type.push('?');
                                    }
                                    parameters.push(DartField {
                                        name: param_name,
                                        ty: param_type,
                                        is_named,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            functions.push(DartFunction {
                name: function_name,
                return_type: return_type, // Use the original type for provider generation
                parameters,
                annotations,
                file_path: file_path.to_path_buf(),
            });
        }
        // Recurse into children
        for child in node.children(&mut node.walk()) {
            visit_functions_recursive(child, source, file_path, functions);
        }
    }

    visit_functions_recursive(root, source, file_path, &mut functions);
    functions
}

/// Extract Dart class field information using tree-sitter-dart
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

    // Output AST to file for debugging
    let mut file = OpenOptions::new().create(true).write(true).append(true).open("debug_ast.txt").unwrap();
    writeln!(file, "\n=== Complete AST for source ===").unwrap();
    write_ast_to_file(root, source, 0, &mut file);
    writeln!(file, "=== End AST ===").unwrap();

    // Search for class nodes
    for class_node in root.children(&mut tree.walk()) {
        if class_node.kind() == "class_definition" {
            // Extract class name
            let class_name = class_node.children(&mut tree.walk()).find(|n| n.kind() == "identifier").map(|n| n.utf8_text(source.as_bytes()).unwrap_or("")).unwrap_or("");
            writeln!(file, "\n=== Processing class: {} ===", class_name).unwrap();
            debug!("Processing class: {}", class_name);
            
            // Search class body
            for member in class_node.children(&mut tree.walk()) {
                if member.kind() == "class_body" {
                    debug!("Found class_body for {}", class_name);
                    for body_item in member.children(&mut tree.walk()) {
                        debug!("Body item: {}", body_item.kind());
                        // Extract fields from declaration (normal Dart class fields)
                        if body_item.kind() == "declaration" {
                            debug!("Found declaration, extracting fields...");
                            extract_fields_from_declaration(body_item, source, &mut fields, &tree);
                        }
                        // Handle normal field declarations
                        else if body_item.kind() == "field_declaration" {
                            debug!("Found field_declaration, extracting fields...");
                            extract_fields_from_field_declaration(body_item, source, &mut fields, &tree);
                        }
                        // Extract constructor parameters
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
                        // Extract normal Dart class constructor parameters
                        else if body_item.kind() == "constructor_declaration" {
                            writeln!(file, "Found constructor_declaration").unwrap();
                            for constructor_child in body_item.children(&mut tree.walk()) {
                                writeln!(file, "  constructor_child: {}", constructor_child.kind()).unwrap();
                                if constructor_child.kind() == "constructor_signature" {
                                    for param_list in constructor_child.children(&mut tree.walk()) {
                                        writeln!(file, "    param_list: {}", param_list.kind()).unwrap();
                                        if param_list.kind() == "formal_parameter_list" {
                                            for param in param_list.children(&mut tree.walk()) {
                                                writeln!(file, "      param: {} | text: {}", param.kind(), param.utf8_text(source.as_bytes()).unwrap_or("<err>")).unwrap();
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
                        // Extract Freezed const factory constructor parameters
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

fn extract_fields_from_field_declaration(field_decl: tree_sitter::Node, source: &str, fields: &mut Vec<DartField>, tree: &tree_sitter::Tree) {
    debug!("extract_fields_from_field_declaration called with kind: {}", field_decl.kind());

    // Debug: print all child node kinds and their text
    for child in field_decl.children(&mut tree.walk()) {
        let kind = child.kind();
        let text = child.utf8_text(source.as_bytes()).unwrap_or("");
        println!("[DEBUG] field_decl child kind: {} | text: {}", kind, text);
    }

    // Robustly extract all type/name pairs from field_declaration for normal Dart classes
    let mut field_type: Option<String> = None;
    let mut field_names = Vec::new();
    
    // Get the full text of the field declaration
    let field_text = field_decl.utf8_text(source.as_bytes()).unwrap();
    debug!("Field declaration text: '{}'", field_text);
    
    // Find type information
    for child in field_decl.children(&mut tree.walk()) {
        debug!("Child kind: {}", child.kind());
        if child.kind() == "type_identifier" {
            field_type = Some(child.utf8_text(source.as_bytes()).unwrap().to_string());
            debug!("Found type: {}", field_type.as_ref().unwrap());
        }
    }
    
    // Find variable names (there may be multiple variables declared)
    for child in field_decl.children(&mut tree.walk()) {
        if child.kind() == "identifier" {
            let name = child.utf8_text(source.as_bytes()).unwrap().to_string();
            field_names.push(name.clone());
            debug!("Found field name: {}", name);
        }
    }
    
    // Create type/name pairs
    if let Some(ty) = field_type {
        for name in field_names {
            if !fields.iter().any(|f| f.name == name) {
                // Check for nullable type
                let final_type = if field_text.contains('?') && !ty.ends_with('?') {
                    format!("{}?", ty)
                } else {
                    ty.clone()
                };
                fields.push(DartField { name: name.clone(), ty: final_type.clone(), is_named: false });
                debug!("Added field: {} {}", final_type, name);
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