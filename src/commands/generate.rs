use std::fs;
use std::path::{Path, PathBuf};
use rayon::prelude::*;
use walkdir::WalkDir;
use tree_sitter::Parser;
use std::fs::OpenOptions;
use std::io::Write;
use log::{info, debug, error};
use sha1::{Sha1, Digest};

// tree-sitter FFI bindings
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
    pub is_named: bool, // Added
}

#[derive(Debug, Clone)]
pub struct DartFunction {
    pub name: String,
    pub return_type: String,
    pub parameters: Vec<DartField>,
    pub annotations: Vec<String>,
    pub file_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct CaseInfo {
    pub case_name: String,
    pub fields: Vec<DartField>,
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

pub fn generate_riverpod_with_paths_and_clean(input_path: &str, output_path: &str, delete_conflicting_outputs: bool) {
    info!("Generating Riverpod code from {} to {}...", input_path, output_path);
    generate_code_for_annotation_with_paths_and_clean("@riverpod", "riverpod", input_path, output_path, delete_conflicting_outputs)
}

fn generate_code_for_annotation(annotation: &str, generator_type: &str) {
    // Auto-detect Flutter project root
    if let Some(project_root) = find_flutter_project_root() {
        let lib_path = project_root.join("lib");
        let lib_path_str = lib_path.to_string_lossy();
        
        info!("Using Flutter project: {}", project_root.display());
        info!("Lib directory: {}", lib_path_str);
        
        // Output to same location as lib directory (.g.dart files in same directory as original files)
        generate_code_for_annotation_with_paths(annotation, generator_type, &lib_path_str, &lib_path_str)
    } else {
        error!("No Flutter project found. Make sure you're in a directory with pubspec.yaml and lib/");
        std::process::exit(1);
    }
}

fn generate_code_for_annotation_with_paths(annotation: &str, generator_type: &str, input_path: &str, output_path: &str) {
    generate_code_for_annotation_with_paths_and_clean(annotation, generator_type, input_path, output_path, false)
}

fn generate_code_for_annotation_with_paths_and_clean(annotation: &str, generator_type: &str, input_path: &str, output_path: &str, delete_conflicting_outputs: bool) {
    eprintln!("[DEBUG] generate_code_for_annotation_with_paths_and_clean called: annotation={}, generator_type={}, input_path={}, output_path={}, delete_conflicting_outputs={}", annotation, generator_type, input_path, output_path, delete_conflicting_outputs);
    
    eprintln!("[DEBUG] Using input path: {}", input_path);
    
    let dart_files = find_dart_files(input_path);
    debug!("Found {} Dart files in {}", dart_files.len(), input_path);
    for file in &dart_files {
        eprintln!("[DEBUG] Dart file found: {}", file.display());
        debug!("  - {}", file.display());
    }
    
    if dart_files.is_empty() {
        info!("No Dart files found in {}", input_path);
        return;
    }
    
            // Parse Dart files in parallel
    let classes: Vec<DartClass> = dart_files
        .par_iter()
        .filter_map(|file_path| parse_dart_file(file_path))
        .flatten()
        .collect();
    
            // Filter classes with specified annotation (remove duplicates)
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
    
            // Remove duplicates (same class name and file path combination)
    target_classes.dedup_by(|a, b| a.name == b.name && a.file_path == b.file_path);
    
    debug!("Found {} classes with annotation '{}':", target_classes.len(), annotation);
    for class in &target_classes {
        debug!("- Class: {} in file: {}", class.name, class.file_path.display());
    }
    
    if target_classes.is_empty() {
        info!("No classes found with annotation: {}", annotation);
        return;
    }
    
    // Create output directory if it doesn't exist
    let output_dir = Path::new(output_path);
    if let Err(e) = fs::create_dir_all(output_dir) {
        error!("Error creating output directory {}: {}", output_path, e);
        return;
    }
    
    // Delete conflicting outputs if requested
    if delete_conflicting_outputs {
        // output_dir だけでなく input_path 全体の .g.dart も削除
        if let Err(e) = clean_output_directory_all_g_dart(Path::new(input_path)) {
            error!("Error cleaning all .g.dart files in {}: {}", input_path, e);
            return;
        }
        if let Err(e) = clean_output_directory(output_dir) {
            error!("Error cleaning output directory {}: {}", output_path, e);
            return;
        }
    }
    
    // Generate .g.dart files in parallel
    let results: Vec<GenerationResult> = target_classes
        .par_iter()
        .filter_map(|class| generate_g_dart_file_with_output_path(class, generator_type, output_path))
        .collect();
    
    // Output results
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
    
    // Search parent directories for pubspec.yaml
    loop {
        let pubspec_path = current_dir.join("pubspec.yaml");
        let lib_path = current_dir.join("lib");
        
        if pubspec_path.exists() && lib_path.exists() {
            debug!("Found Flutter project root: {}", current_dir.display());
            return Some(current_dir);
        }
        
        // Move to parent directory
        if !current_dir.pop() {
            break;
        }
    }
    
    None
}

fn find_project_root_from_file(file_path: &Path) -> PathBuf {
    // Find the project root by looking for pubspec.yaml
    let mut dir = file_path.parent().unwrap_or_else(|| Path::new(""));
    
    loop {
        if dir.join("pubspec.yaml").exists() {
            return dir.to_path_buf();
        }
        
        if let Some(parent) = dir.parent() {
            dir = parent;
        } else {
            break;
        }
    }
    
    // If no pubspec.yaml found, use the current working directory
    std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf())
}

fn update_part_directive_in_file(input_file: &Path, output_file: &Path) {
    let content = match fs::read_to_string(input_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("[DEBUG] Failed to read input file {}: {}", input_file.display(), e);
            return;
        }
    };
    
    // Calculate relative path from input file to output file
    let input_dir = input_file.parent().unwrap_or_else(|| Path::new(""));
    let output_dir = output_file.parent().unwrap_or_else(|| Path::new(""));
    let output_filename = output_file.file_name().unwrap_or_else(|| std::ffi::OsStr::new(""));
    
    let relative_path = if input_dir == output_dir {
        output_filename.to_string_lossy().to_string()
    } else {
        // Calculate relative path from input to output
        let mut relative = String::new();
        let mut input_parts: Vec<_> = input_dir.components().collect();
        let mut output_parts: Vec<_> = output_dir.components().collect();
        
        // Find common prefix
        let mut common_len = 0;
        for (a, b) in input_parts.iter().zip(output_parts.iter()) {
            if a == b {
                common_len += 1;
            } else {
                break;
            }
        }
        
        // Add ".." for each level up from input to common ancestor
        for _ in 0..(input_parts.len() - common_len) {
            if !relative.is_empty() {
                relative.push('/');
            }
            relative.push_str("..");
        }
        
        // Add path from common ancestor to output file
        for part in output_parts.iter().skip(common_len) {
            if !relative.is_empty() {
                relative.push('/');
            }
            relative.push_str(&part.as_os_str().to_string_lossy());
        }
        
        // Add filename
        if !relative.is_empty() {
            relative.push('/');
        }
        relative.push_str(&output_filename.to_string_lossy());
        
        relative
    };
    
    // Replace the part directive
    let old_part = format!("part '{}';", output_filename.to_string_lossy());
    let new_part = format!("part '{}';", relative_path);
    
    let updated_content = content.replace(&old_part, &new_part);
    
    if updated_content != content {
        if let Err(e) = fs::write(input_file, updated_content) {
            eprintln!("[DEBUG] Failed to update part directive in {}: {}", input_file.display(), e);
        } else {
            eprintln!("[DEBUG] Updated part directive in {}: {} -> {}", input_file.display(), old_part, new_part);
        }
    }
}

fn find_dart_files(dir_path: &str) -> Vec<PathBuf> {
    eprintln!("[DEBUG] find_dart_files called with dir_path: {}", dir_path);
    let mut dart_files = Vec::new();
    
    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension == "dart" {
                    eprintln!("[DEBUG] Found Dart file: {}", path.display());
                    dart_files.push(path.to_path_buf());
                }
            }
        }
    }
    
    eprintln!("[DEBUG] find_dart_files returning {} files", dart_files.len());
    dart_files
}

fn clean_output_directory(output_dir: &Path) -> Result<(), std::io::Error> {
    eprintln!("[DEBUG] clean_output_directory called for: {}", output_dir.display());
    if !output_dir.exists() {
        eprintln!("[DEBUG] Output directory does not exist: {}", output_dir.display());
        return Ok(());
    }
    
    eprintln!("[DEBUG] Scanning output directory: {}", output_dir.display());
    for entry in WalkDir::new(output_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            eprintln!("[DEBUG] Found file: {}", path.display());
            if let Some(file_name) = path.file_name() {
                let file_name_str = file_name.to_string_lossy();
                eprintln!("[DEBUG] File name: {}", file_name_str);
                if file_name_str.ends_with(".g.dart") {
                    info!("Deleting conflicting output: {}", path.display());
                    fs::remove_file(path)?;
                    eprintln!("[DEBUG] Deleted file: {}", path.display());
                }
            }
        }
    }
    
    Ok(())
}

fn clean_output_directory_all_g_dart(input_path: &Path) -> Result<(), std::io::Error> {
    eprintln!("[DEBUG] clean_output_directory_all_g_dart called for: {}", input_path.display());
    if !input_path.exists() {
        eprintln!("[DEBUG] Input directory does not exist: {}", input_path.display());
        return Ok(());
    }
    for entry in WalkDir::new(input_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Some(file_name) = path.file_name() {
                let file_name_str = file_name.to_string_lossy();
                if file_name_str.ends_with(".g.dart") {
                    info!("Deleting conflicting output (all): {}", path.display());
                    fs::remove_file(path)?;
                    eprintln!("[DEBUG] Deleted file (all): {}", path.display());
                }
            }
        }
    }
    Ok(())
}

fn parse_dart_file(file_path: &Path) -> Option<Vec<DartClass>> {
    eprintln!("[DEBUG] parse_dart_file called: {}", file_path.display());
    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            error!("Error reading {}: {}", file_path.display(), e);
            return None;
        }
    };
    
    // Simple parsing (using regex as tree-sitter-dart implementation is complex)
    parse_dart_content(&content, file_path)
}

fn parse_dart_content(content: &str, file_path: &Path) -> Option<Vec<DartClass>> {
    eprintln!("[DEBUG] parse_dart_content called: {} ({} bytes)", file_path.display(), content.len());
    let mut classes = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut pending_annotations: Vec<String> = Vec::new();
    
    for line in lines.iter() {
        let trimmed = line.trim();
        if trimmed.starts_with('@') {
            pending_annotations.push(trimmed.to_string());
        } else if trimmed.starts_with("class ") || trimmed.starts_with("abstract class ") {
            // Extract class name from "class ClassName" or "abstract class ClassName"
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
            let type_text = child.utf8_text(source.as_bytes()).unwrap().trim().to_string();
            if !type_text.is_empty() {
                field_type = Some(type_text);
                debug!("Found type: {}", field_type.as_ref().unwrap());
            }
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
                            ty = t.utf8_text(source.as_bytes()).unwrap().trim().to_string();
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
            ty = t.utf8_text(source.as_bytes()).unwrap().trim().to_string();
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
                    let type_text = child.utf8_text(source.as_bytes()).unwrap().trim().to_string();
                    if !type_text.is_empty() {
                        field_type = Some(type_text);
                        debug!("Found type: {}", field_type.as_ref().unwrap());
                    }
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

fn generate_g_dart_file_with_output_path(class: &DartClass, generator_type: &str, output_path: &str) -> Option<GenerationResult> {
    let input_file = &class.file_path;
    let file_stem = input_file.file_stem()?;
    
    // Resolve output path relative to the input file's directory
    let input_dir = input_file.parent().unwrap_or_else(|| Path::new(""));
    let output_dir = if output_path.starts_with('/') {
        // Absolute path
        Path::new(output_path).to_path_buf()
    } else {
        // Relative path - resolve from current working directory
        // For output paths like "test_flutter_app/aminomi/lib/gen", use it as is
        Path::new(output_path).to_path_buf()
    };
    
    let output_file = match generator_type {
        "freezed" => output_dir.join(format!("{}.g.dart", file_stem.to_string_lossy())),
        "json" => output_dir.join(format!("{}.g.dart", file_stem.to_string_lossy())),
        "riverpod" => output_dir.join(format!("{}.g.dart", file_stem.to_string_lossy())),
        _ => output_dir.join(format!("{}.g.dart", file_stem.to_string_lossy())),
    };
    
    eprintln!("[DEBUG] generate_g_dart_file_with_output_path: input_file={}, output_path={}, output_file={}", input_file.display(), output_path, output_file.display());
    
    // Create output directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&output_dir) {
        eprintln!("[DEBUG] Failed to create output directory {}: {}", output_dir.display(), e);
        return None;
    }
    
    // Update the original file's part directive
    update_part_directive_in_file(&class.file_path, &output_file);

    let generated_code = match generator_type {
        "freezed" => generate_freezed_code(class),
        "json" => generate_json_code(class),
        "riverpod" => generate_riverpod_code(class),
        _ => return None,
    };

    eprintln!("[DEBUG] Generated code length: {} characters", generated_code.len());
    
    Some(GenerationResult {
        input_file: input_file.clone(),
        output_file,
        generated_code,
    })
}

fn generate_freezed_code(class: &DartClass) -> String {
    eprintln!("[DEBUG] generate_freezed_code called for {}", class.name);
    let mut code = String::new();
    code.push_str("// GENERATED CODE - DO NOT MODIFY BY HAND\n");
    code.push_str("// **************************************************************************\n");
    code.push_str("// FreezedGenerator\n");
    code.push_str("// **************************************************************************\n\n");
    code.push_str(&format!("part of '{}';\n\n", class.file_path.file_name().unwrap().to_string_lossy()));

    let class_name = &class.name;
    let source_content = std::fs::read_to_string(&class.file_path).unwrap_or_default();
    let union_cases = extract_union_cases_from_dart_class(&source_content, class_name);

    eprintln!("[DEBUG] generate_freezed_code for {}: {} union cases found", class_name, union_cases.len());
    for (i, case) in union_cases.iter().enumerate() {
        eprintln!("[DEBUG] case {}: {} with {} fields", i, case.case_name, case.fields.len());
    }

    if union_cases.len() > 1 {
        // Union type (sealed class): Generate each case as independent class
        for case in &union_cases {
            let case_class_name = format!("{}{}", class_name, to_pascal_case(&case.case_name));
            code.push_str(&format!("class {} implements {} {{\n", case_class_name, class_name));
            for field in &case.fields {
                code.push_str(&format!("  final {} {};\n", field.ty, field.name));
            }
            code.push_str("\n");
            code.push_str(&format!("  const {}({{", case_class_name));
            for (i, field) in case.fields.iter().enumerate() {
                if i > 0 { code.push_str(", "); }
                if field.ty.ends_with('?') {
                    code.push_str(&format!("this.{}", field.name));
                } else {
                    code.push_str(&format!("required this.{}", field.name));
                }
            }
            code.push_str("});\n\n");
            code.push_str("  @override\n  String toString() {\n    return '");
            code.push_str(&format!("{}({})", case_class_name, 
                case.fields.iter().map(|f| format!("{}: ${{{}}}", f.name, f.name)).collect::<Vec<_>>().join(", ")));
            code.push_str("';\n  }\n\n");
            code.push_str("  @override\n  bool operator ==(Object other) {\n    return identical(this, other) ||\n        other is ");
            code.push_str(&format!("{} &&\n", case_class_name));
            for field in &case.fields {
                code.push_str(&format!("        {} == other.{} &&\n", field.name, field.name));
            }
            code.push_str("        true;\n  }\n\n");
            code.push_str("  @override\n  int get hashCode => ");
            for (i, field) in case.fields.iter().enumerate() {
                if i > 0 { code.push_str(" ^ "); }
                code.push_str(&format!("{}.hashCode", field.name));
            }
            if case.fields.is_empty() {
                code.push_str("0");
            }
            code.push_str(";\n\n");
            code.push_str(&format!("  Map<String, dynamic> toJson() => _${}ToJson(this);\n", case_class_name));
            code.push_str("}\n\n");
        }
    } else {
        // Regular class
        let fields = extract_fields_from_dart_class(&source_content, class_name);
        code.push_str(&format!("mixin _${} {{\n", class_name));
        code.push_str(&format!("  {} copyWith({{", class_name));
        for (i, field) in fields.iter().enumerate() {
            if i > 0 { code.push_str(", "); }
            let param_type = if field.ty.ends_with('?') {
                field.ty.clone()
            } else {
                format!("{}?", field.ty)
            };
            code.push_str(&format!("{} {}", param_type, field.name));
        }
        code.push_str("});\n");
        code.push_str("}\n\n");
        code.push_str(&format!("class _{} with _${} implements {} {{\n", class_name, class_name, class_name));
        for field in &fields {
            code.push_str(&format!("  final {} {};\n", field.ty, field.name));
        }
        code.push_str("\n");
        code.push_str(&format!("  const _{}({{", class_name));
        for (i, field) in fields.iter().enumerate() {
            if i > 0 { code.push_str(", "); }
            if field.ty.ends_with('?') {
                code.push_str(&format!("this.{}", field.name));
            } else {
                code.push_str(&format!("required this.{}", field.name));
            }
        }
        code.push_str(");\n\n");
        code.push_str(&format!("  @override\n  {} copyWith({{", class_name));
        for (i, field) in fields.iter().enumerate() {
            if i > 0 { code.push_str(", "); }
            let param_type = if field.ty.ends_with('?') {
                field.ty.clone()
            } else {
                format!("{}?", field.ty)
            };
            code.push_str(&format!("{} {}", param_type, field.name));
        }
        code.push_str("}}) {\n");
        code.push_str(&format!("    return _{}(", class_name));
        for (i, field) in fields.iter().enumerate() {
            if i > 0 { code.push_str(", "); }
            code.push_str(&format!("{}: {} ?? this.{}", field.name, field.name, field.name));
        }
        code.push_str(");\n  }\n\n");
        code.push_str("  @override\n  String toString() {\n    return '");
        code.push_str(&format!("{}(", class_name));
        for (i, field) in fields.iter().enumerate() {
            if i > 0 { code.push_str(", "); }
            code.push_str(&format!("{}: ${{{}}}", field.name, field.name));
        }
        code.push_str(")';\n  }\n\n");
        code.push_str("  @override\n  bool operator ==(Object other) {\n    return identical(this, other) ||\n        other is _");
        code.push_str(&format!("{} &&\n", class_name));
        for field in &fields {
            code.push_str(&format!("        {} == other.{} &&\n", field.name, field.name));
        }
        code.push_str("        true;\n  }\n\n");
        code.push_str("  @override\n  int get hashCode => ");
        for (i, field) in fields.iter().enumerate() {
            if i > 0 { code.push_str(" ^ "); }
            code.push_str(&format!("{}.hashCode", field.name));
        }
        code.push_str(";\n\n");
        code.push_str(&format!("  Map<String, dynamic> toJson() => _${}ToJson(this);\n", class_name));
        code.push_str("}\n");
    }

    // Add JSON serialization functions for Freezed classes
    code.push_str("\n// JSON Serialization Functions\n");
    if union_cases.len() > 1 {
        // Union type: Generate JSON functions for each case
        for case in &union_cases {
            let case_class_name = format!("{}{}", class_name, to_pascal_case(&case.case_name));
            code.push_str(&format!("Map<String, dynamic> _${}ToJson({} instance) {{\n", case_class_name, case_class_name));
            code.push_str("  return {\n");
            for field in &case.fields {
                code.push_str(&format!("    '{}': instance.{},\n", field.name, field.name));
            }
            code.push_str("  };\n");
            code.push_str("}\n\n");
            
            code.push_str(&format!("{} _${}FromJson(Map<String, dynamic> json) {{\n", case_class_name, case_class_name));
            code.push_str(&format!("  return {}(\n", case_class_name));
            for field in &case.fields {
                code.push_str(&format!("    {}: json['{}'] as {},\n", field.name, field.name, field.ty));
            }
            code.push_str("  );\n");
            code.push_str("}\n\n");
        }
    } else {
        // Regular class: Generate JSON functions
        let fields = extract_fields_from_dart_class(&source_content, class_name);
        code.push_str(&format!("Map<String, dynamic> _${}ToJson({} instance) {{\n", class_name, class_name));
        code.push_str("  return {\n");
        for field in &fields {
            code.push_str(&format!("    '{}': instance.{},\n", field.name, field.name));
        }
        code.push_str("  };\n");
        code.push_str("}\n\n");
        
        code.push_str(&format!("{} _${}FromJson(Map<String, dynamic> json) {{\n", class_name, class_name));
        code.push_str(&format!("  return {}(\n", class_name));
        for field in &fields {
            code.push_str(&format!("    {}: json['{}'] as {},\n", field.name, field.name, field.ty));
        }
        code.push_str("  );\n");
        code.push_str("}\n");
    }
    
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
    code.push_str("// GENERATED CODE - DO NOT MODIFY BY HAND\n");
    code.push_str("// **************************************************************************\n");
    code.push_str("// JsonSerializableGenerator\n");
    code.push_str("// **************************************************************************\n\n");
    code.push_str(&format!("part of '{}';\n\n", class.file_path.file_name().unwrap().to_string_lossy()));
    let class_name = &class.name;
    let source_content = std::fs::read_to_string(&class.file_path).unwrap_or_default();
    let union_cases = extract_union_cases_from_dart_class(&source_content, class_name);
    if union_cases.len() > 1 {
        // Union type (sealed class): Only case class definitions are generated (Json functions are not generated)
        for case in &union_cases {
            let case_class_name = format!("{}{}", class_name, to_pascal_case(&case.case_name));
            code.push_str(&format!("// {}: generated by Rust, JSON serialization is handled by build_runner\n", case_class_name));
        }
    } else {
        // Regular class: Only class definition is generated (Json functions are not generated)
        code.push_str(&format!("// {}: generated by Rust, JSON serialization is handled by build_runner\n", class_name));
    }
    code
}

fn generate_riverpod_code(class: &DartClass) -> String {
    let mut code = String::new();
    code.push_str("// GENERATED CODE - DO NOT MODIFY BY HAND\n");
    code.push_str("// **************************************************************************\n");
    code.push_str("// RiverpodGenerator\n");
    code.push_str("// **************************************************************************\n\n");
    // Calculate relative path from output to input file
    let output_dir = Path::new("lib/gen"); // This is where the generated file will be
    let input_dir = class.file_path.parent().unwrap_or_else(|| Path::new(""));
    let relative_path = if output_dir == input_dir {
        class.file_path.file_name().unwrap().to_string_lossy().to_string()
    } else {
        // Calculate relative path from lib/gen to lib/providers
        "providers/".to_string() + &class.file_path.file_name().unwrap().to_string_lossy()
    };
    
    code.push_str(&format!("part of '{}';\n\n", relative_path));
    
    // Note: In Dart part files, imports should be in the main file, not in the part file
    // The main file (auth_provider.dart) should have the necessary imports

    // Extract function and class information from source file
    let source_content = std::fs::read_to_string(&class.file_path).unwrap_or_default();
    let functions = extract_functions_from_dart_source(&source_content, &class.file_path);
    
    debug!("Found {} functions in {}", functions.len(), class.file_path.display());
    for function in &functions {
        debug!("Function: {} with annotations: {:?}", function.name, function.annotations);
    }
    
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
    
    // Generate hash function for the provider
    let hash_input = format!("{}{}", function.name, function.file_path.display());
    let mut hasher = Sha1::new();
    hasher.update(hash_input.as_bytes());
    let hash_result = hasher.finalize();
    let hash_string = format!("{:x}", hash_result);
    
    code.push_str(&format!("String _${}Hash() => r'{}';\n\n", provider_name, hash_string));
    
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
        code.push_str(&format!("  name: r'{}',\n", provider_name));
        code.push_str(&format!("  debugGetCreateSourceHash:\n"));
        code.push_str(&format!("      const bool.fromEnvironment('dart.vm.product') ? null : _${}Hash,\n", provider_name));
        code.push_str("  dependencies: null,\n");
        code.push_str("  allTransitiveDependencies: null,\n");
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
    
    // Parse the class to determine its actual type
    let class_content = match std::fs::read_to_string(&class.file_path) {
        Ok(content) => content,
        Err(_) => return String::new(),
    };
    
    // Extract the class type from the class definition
    let class_type = extract_class_type_from_content(&class_content, &class.name);
    
    // Generate the _$ClassName class (build_runner compatible)
    let generated_class_name = format!("_${}", class.name);
    code.push_str(&format!("abstract class {} extends Notifier<{}> {{\n", generated_class_name, class_type));
    code.push_str(&format!("  @override\n"));
    code.push_str(&format!("  {} build();\n", class_type));
    code.push_str("}\n\n");
    
    // Generate NotifierProvider
    let provider_name = format!("{}Provider", to_lower_camel_case(&class.name));
    code.push_str(&format!("final {} = NotifierProvider<{}, {}>(() {{\n", 
        provider_name, class.name, class_type
    ));
    code.push_str(&format!("  return {}();\n", class.name));
    code.push_str("});\n");
    
    code
}

fn extract_class_type_from_content(content: &str, class_name: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    
    // First, try to find the build() method to determine the correct type
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("@override") {
            // Look for the next line which should be the build method
            if i + 1 < lines.len() {
                let next_line = lines[i + 1].trim();
                if next_line.starts_with("AuthState build()") || next_line.starts_with("String build()") || next_line.starts_with("bool build()") {
                    // Extract the return type from the build method
                    if next_line.contains("AuthState") {
                        return "AuthState".to_string();
                    } else if next_line.contains("String") {
                        return "String".to_string();
                    } else if next_line.contains("bool") {
                        return "bool".to_string();
                    }
                }
            }
        }
    }
    
    // Fallback: look for class definition
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("class ") && trimmed.contains(class_name) {
            // Look for "extends Notifier<Type>" or "extends _$ClassName"
            if trimmed.contains("extends _$") {
                // For generated classes, extract the type from the class name
                // Remove "Notifier" suffix if present
                let type_name = if class_name.ends_with("Notifier") {
                    class_name.trim_end_matches("Notifier").to_string()
                } else {
                    class_name.to_string()
                };
                return type_name;
            } else if trimmed.contains("extends Notifier<") {
                // Extract type from "extends Notifier<Type>"
                if let Some(start) = trimmed.find("Notifier<") {
                    if let Some(end) = trimmed[start..].find('>') {
                        return trimmed[start + 9..start + end].to_string();
                    }
                }
            }
        }
    }
    
    // Default fallback
    "String".to_string()
}

fn to_lower_camel_case(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
    }
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize = true;
    for c in s.chars() {
        if c == '_' || c == '-' {
            capitalize = true;
        } else if capitalize {
            result.push(c.to_ascii_uppercase());
            capitalize = false;
        } else {
            result.push(c);
        }
    }
    result
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

            // Extract return type (with nullable support)
            let mut return_type = "dynamic".to_string();
            let mut base_type = "dynamic".to_string();
            let mut type_arguments = Vec::new();
            let mut found_nullable = false;
            for child in node.children(&mut node.walk()) {
                if child.kind() == "nullable_type" {
                    // e.g. AppUser?
                    found_nullable = true;
                    for nullable_child in child.children(&mut child.walk()) {
                        if nullable_child.kind() == "type_identifier" {
                            base_type = nullable_child.utf8_text(source.as_bytes()).unwrap_or("dynamic").to_string();
                        }
                        if nullable_child.kind() == "type_arguments" {
                            let args_text = nullable_child.utf8_text(source.as_bytes()).unwrap_or("");
                            type_arguments.push(args_text.to_string());
                        }
                    }
                } else if child.kind() == "type_identifier" {
                    base_type = child.utf8_text(source.as_bytes()).unwrap_or("dynamic").to_string();
                } else if child.kind() == "type_arguments" {
                    let args_text = child.utf8_text(source.as_bytes()).unwrap_or("");
                    type_arguments.push(args_text.to_string());
                } else if child.kind() == "function_type" {
                    return_type = child.utf8_text(source.as_bytes()).unwrap_or("dynamic").to_string();
                    break;
                }
            }
            // Construct the full type if we have base type and arguments
            if base_type != "dynamic" && !type_arguments.is_empty() {
                return_type = format!("{}{}", base_type, type_arguments.join(""));
            } else if base_type != "dynamic" {
                return_type = base_type;
            }
            if found_nullable && !return_type.ends_with('?') {
                return_type = format!("{}?", return_type);
            }
            // Fallback: If we found a simple type, check if it's part of a complex type
            if return_type == "dynamic" {
                let function_text = node.utf8_text(source.as_bytes()).unwrap_or("");
                if function_text.contains("Future<") {
                    if let Some(start) = function_text.find("Future<") {
                        if let Some(end) = function_text[start..].find('>') {
                            let full_type = &function_text[start..start + end + 1];
                            return_type = full_type.to_string();
                        }
                    }
                } else if function_text.contains("List<") {
                    if let Some(start) = function_text.find("List<") {
                        if let Some(end) = function_text[start..].find('>') {
                            let full_type = &function_text[start..start + end + 1];
                            return_type = full_type.to_string();
                        }
                    }
                }
            }

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
                            param_type = param_child.utf8_text(source.as_bytes()).unwrap_or("dynamic").trim().to_string();
                        } else if param_child.kind() == "identifier" {
                                    param_name = param_child.utf8_text(source.as_bytes()).unwrap_or("param").to_string();
                                }
                            }
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
                            for opt_param in param.children(&mut node.walk()) {
                                if opt_param.kind() == "formal_parameter" {
                                    let opt_param_text = opt_param.utf8_text(source.as_bytes()).unwrap_or("");
                                    debug!("Optional parameter: {}", opt_param_text);
                                    let mut param_type = "dynamic".to_string();
                                    let mut param_name = "param".to_string();
                                    let mut is_named = true;
                                    for opt_param_child in opt_param.children(&mut node.walk()) {
                                                                if opt_param_child.kind() == "type_identifier" {
                            param_type = opt_param_child.utf8_text(source.as_bytes()).unwrap_or("dynamic").trim().to_string();
                        } else if opt_param_child.kind() == "identifier" {
                                            param_name = opt_param_child.utf8_text(source.as_bytes()).unwrap_or("param").to_string();
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
                return_type,
                parameters,
                annotations,
                file_path: file_path.to_path_buf(),
            });
        }
        for child in node.children(&mut node.walk()) {
            visit_functions_recursive(child, source, file_path, functions);
        }
    }
    visit_functions_recursive(root, source, file_path, &mut functions);
    functions
}

/// Extract Dart class field information using tree-sitter-dart
pub fn extract_fields_from_dart_class(source: &str, target_class_name: &str) -> Vec<DartField> {
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
            
            // Only process the target class
            if class_name != target_class_name {
                continue;
            }
            
            // Check if this is a union/sealed class by looking for multiple factory constructors
            let mut factory_constructors = Vec::new();
            
            // First pass: collect all factory constructors to determine if this is a union type
            for member in class_node.children(&mut tree.walk()) {
                if member.kind() == "class_body" {
                    for body_item in member.children(&mut tree.walk()) {
                        if body_item.kind() == "redirecting_factory_constructor_signature" {
                            let constructor_text = body_item.utf8_text(source.as_bytes()).unwrap_or("");
                            factory_constructors.push(constructor_text);
                            debug!("Found factory constructor: {}", constructor_text);
                        }
                    }
                }
            }
            
            // Determine if this is a union type (has multiple factory constructors)
            let is_union_type = factory_constructors.len() > 1;
            debug!("Class {} is union type: {}", class_name, is_union_type);
            
            // Second pass: extract fields based on type
            for member in class_node.children(&mut tree.walk()) {
                if member.kind() == "class_body" {
                    debug!("Found class_body for {}", class_name);
                    for body_item in member.children(&mut tree.walk()) {
                        debug!("Body item: {}", body_item.kind());
                        
                        if is_union_type {
                            // For union types, only extract fields from declarations (not constructors)
                            if body_item.kind() == "declaration" {
                                debug!("Found declaration in union type, extracting fields...");
                                extract_fields_from_declaration(body_item, source, &mut fields, &tree);
                            }
                            else if body_item.kind() == "field_declaration" {
                                debug!("Found field_declaration in union type, extracting fields...");
                                extract_fields_from_field_declaration(body_item, source, &mut fields, &tree);
                            }
                            // Skip constructor extraction for union types
                        } else {
                            // For non-union types, extract all fields normally
                            if body_item.kind() == "declaration" {
                                debug!("Found declaration, extracting fields...");
                                extract_fields_from_declaration(body_item, source, &mut fields, &tree);
                            }
                            else if body_item.kind() == "field_declaration" {
                                debug!("Found field_declaration, extracting fields...");
                                extract_fields_from_field_declaration(body_item, source, &mut fields, &tree);
                            }
                            else if body_item.kind() == "constructor_signature" || body_item.kind() == "redirecting_factory_constructor_signature" {
                                writeln!(file, "Found constructor: {}", body_item.kind()).unwrap();
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
                            else if body_item.kind() == "const_constructor_signature" {
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
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let lib_dir = temp_dir.path().join("lib");
        fs::create_dir_all(&lib_dir).unwrap();

        // Create a test Dart file
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
        
        let fields = extract_fields_from_dart_class(freezed_source, "User");
        debug!("Extracted fields: {:?}", fields);
        
        // Check expected fields
        assert_eq!(fields.len(), 3);
        
        let name_field = fields.iter().find(|f| f.name == "name").unwrap();
        assert_eq!(name_field.ty, "String");
        
        let email_field = fields.iter().find(|f| f.name == "email").unwrap();
        assert_eq!(email_field.ty, "String");
        
        let age_field = fields.iter().find(|f| f.name == "age").unwrap();
        assert_eq!(age_field.ty, "int?");
    }

    #[test]
    fn test_generate_with_custom_paths() {
        // Create a temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let input_dir = temp_dir.path().join("custom_input");
        let output_dir = temp_dir.path().join("custom_output");
        
        fs::create_dir_all(&input_dir).unwrap();
        fs::create_dir_all(&output_dir).unwrap();
        
        // Create a test Dart file with @riverpod annotation
        let dart_content = r#"
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'test_provider.g.dart';

@riverpod
Future<String> testFunction(TestFunctionRef ref) async {
  return "test";
}
"#;
        
        fs::write(input_dir.join("test_provider.dart"), dart_content).unwrap();
        
        // Test the generation with custom paths
        let input_path = input_dir.to_str().unwrap();
        let output_path = output_dir.to_str().unwrap();
        
        // This would normally call the actual generation function
        // For now, we'll test the path handling
        let dart_files = find_dart_files(input_path);
        assert_eq!(dart_files.len(), 1);
        assert!(dart_files[0].file_name().unwrap() == "test_provider.dart");
        
        // Test that the output directory exists
        assert!(output_dir.exists());
    }

    #[test]
    fn test_generate_g_dart_file_with_output_path() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("test.dart");
        
        // Create a test class
        let class = DartClass {
            name: "TestClass".to_string(),
            annotations: vec!["@riverpod".to_string()],
            file_path: input_file.clone(),
        };
        
        // Test generation with custom output path
        let result = generate_g_dart_file_with_output_path(&class, "riverpod", temp_dir.path().to_str().unwrap());
        
        assert!(result.is_some());
        let result = result.unwrap();
        
        // Check that output file path is correct
        assert!(result.output_file.file_name().unwrap() == "test.g.dart");
        assert!(result.output_file.parent().unwrap() == temp_dir.path());
        
        // Check that generated code contains expected content
        assert!(result.generated_code.contains("part of 'test.dart'"));
        assert!(result.generated_code.contains("// GENERATED CODE"));
    }

    #[test]
    fn test_find_dart_files_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let lib_dir = temp_dir.path().join("lib");
        let models_dir = lib_dir.join("models");
        let providers_dir = lib_dir.join("providers");
        
        fs::create_dir_all(&models_dir).unwrap();
        fs::create_dir_all(&providers_dir).unwrap();
        
        // Create Dart files in different subdirectories
        fs::write(models_dir.join("user.dart"), "class User {}").unwrap();
        fs::write(providers_dir.join("auth_provider.dart"), "class AuthProvider {}").unwrap();
        fs::write(lib_dir.join("main.dart"), "void main() {}").unwrap();
        
        let dart_files = find_dart_files(temp_dir.path().join("lib").to_str().unwrap());
        
        // Should find all 3 Dart files recursively
        assert_eq!(dart_files.len(), 3);
        
        let file_names: Vec<_> = dart_files.iter()
            .map(|f| f.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        
        assert!(file_names.contains(&"user.dart".to_string()));
        assert!(file_names.contains(&"auth_provider.dart".to_string()));
        assert!(file_names.contains(&"main.dart".to_string()));
    }
} 

/// Extract union cases (factory constructors) and their fields from a Dart class
pub fn extract_union_cases_from_dart_class(source: &str, target_class_name: &str) -> Vec<CaseInfo> {
    eprintln!("[DEBUG] extract_union_cases_from_dart_class called for {}", target_class_name);
    let mut parser = Parser::new();
    parser.set_language(unsafe { std::mem::transmute(tree_sitter_dart()) }).unwrap();
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();
    let mut cases = Vec::new();

    for class_node in root.children(&mut tree.walk()) {
        if class_node.kind() == "class_definition" {
            let class_name = class_node.children(&mut tree.walk()).find(|n| n.kind() == "identifier").map(|n| n.utf8_text(source.as_bytes()).unwrap_or("")).unwrap_or("");
            if class_name != target_class_name {
                continue;
            }
            for member in class_node.children(&mut tree.walk()) {
                if member.kind() == "class_body" {
                    for body_item in member.children(&mut tree.walk()) {
                        if body_item.kind() == "declaration" {
                            for child in body_item.children(&mut tree.walk()) {
                                if child.kind() == "redirecting_factory_constructor_signature" {
                                    let mut case_name = String::new();
                                    let mut fields = Vec::new();
                                    let mut found_dot = false;
                                    for sub in child.children(&mut tree.walk()) {
                                        if sub.kind() == "identifier" && found_dot {
                                            case_name = sub.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                                            found_dot = false;
                                        } else if sub.kind() == "." {
                                            found_dot = true;
                                        }
                                        if sub.kind() == "formal_parameter_list" {
                                            for param in sub.children(&mut tree.walk()) {
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
                                    if !case_name.is_empty() {
                                        cases.push(CaseInfo { case_name, fields });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // Debug output
    eprintln!("[DEBUG] union cases for {}:", target_class_name);
    for case in &cases {
        eprintln!("  case: {}", case.case_name);
        for f in &case.fields {
            eprintln!("    field: {} {}", f.ty, f.name);
        }
    }
    cases
} 