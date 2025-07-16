// Freezed/JsonSerializable code generation logic

use std::path::{Path, PathBuf};
use std::fs;
use regex;

#[derive(Clone, Debug)]
pub struct DartClass {
    pub name: String,
    pub annotations: Vec<String>,
    pub file_path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct DartField {
    pub name: String,
    pub ty: String,
    pub is_named: bool, // Added
    pub has_default: bool, // Added for @Default annotation
    pub default_value: Option<String>, // Added for @Default annotation value
}

#[derive(Clone, Debug)]
pub struct DartFunction {
    pub name: String,
    pub return_type: String,
    pub parameters: Vec<DartField>,
    pub annotations: Vec<String>,
    pub file_path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct CaseInfo {
    pub case_name: String,
    pub fields: Vec<DartField>,
}

pub struct GenerationResult {
    pub freezed_code: String,
    pub g_dart_code: String,
}

// --- Freezed/JsonSerializable code generation functions ---

pub fn generate_freezed_code(class: &DartClass) -> String {
    eprintln!("[DEBUG] generate_freezed_code called for {}", class.name);
    let mut code = String::new();
    
    let source_content = std::fs::read_to_string(&class.file_path).unwrap_or_default();
    let union_cases = extract_union_cases_from_dart_class(&source_content, &class.name);
    let fields = extract_fields_from_dart_class(&source_content, &class.name);

    if union_cases.len() > 1 {
        // Union type (sealed class): Generate mixin first
        code.push_str(&format!("mixin _${} {{\n", class.name));
        code.push_str("}\n\n");
        
        // Generate each case as independent class
        for case in &union_cases {
            let case_class_name = format!("{}{}", class.name, to_pascal_case(&case.case_name));
            code.push_str(&format!("class {} with _${} implements {} {{\n", case_class_name, class.name, class.name));
            
            // Generate fields
            for field in &case.fields {
                code.push_str(&format!("  final {} {};\n", field.ty, field.name));
            }
            code.push_str("\n");
            
            // Constructor - use positional parameters to match factory constructor
            code.push_str(&format!("  const {}(\n", case_class_name));
            for field in &case.fields {
                code.push_str(&format!("    this.{},\n", field.name));
            }
            code.push_str("  );\n\n");
            
            // toString
            code.push_str("  @override\n  String toString() {\n    return '");
            code.push_str(&format!("{}(", case_class_name));
            code.push_str(&case.fields.iter().map(|f| format!("{}: ${}", f.name, f.name)).collect::<Vec<_>>().join(", "));
            code.push_str(")';\n  }\n\n");
            
            // operator ==
            code.push_str("  @override\n  bool operator ==(Object other) {\n    return identical(this, other) ||\n        other is ");
            code.push_str(&format!("{} &&\n", case_class_name));
            for field in &case.fields {
                code.push_str(&format!("        {} == other.{} &&\n", field.name, field.name));
            }
            code.push_str("        true;\n  }\n\n");
            
            // hashCode
            code.push_str("  @override\n  int get hashCode => ");
            for (i, field) in case.fields.iter().enumerate() {
                if i > 0 { code.push_str(" ^ "); }
                code.push_str(&format!("{}.hashCode", field.name));
            }
            if case.fields.is_empty() {
                code.push_str("0");
            }
            code.push_str(";\n\n");
            
            // toJson
            code.push_str(&format!("  Map<String, dynamic> toJson() => _${}ToJson(this);\n", case_class_name));
            code.push_str("}\n\n");
        }
    } else {
        // Regular class - generate implementation class directly
        code.push_str(&format!("class _{} implements {} {{\n", class.name, class.name));
        
        // Generate fields
        for field in &fields {
            code.push_str(&format!("  final {} {};\n", field.ty, field.name));
        }
        code.push_str("\n");
        
        // Constructor - use named parameters to match the factory constructor
        code.push_str(&format!("  const _{}(", class.name));
        code.push_str("{\n");
        for field in fields.iter() {
            if field.ty.ends_with('?') {
                // Nullable fields don't need required
                code.push_str(&format!("    this.{},\n", field.name));
            } else if field.has_default {
                // Fields with @Default don't need required and have default value
                if let Some(ref default_val) = field.default_value {
                    // 型に応じてDartリテラルを出力
                    let dart_default = if field.ty == "String" {
                        let val = default_val.trim();
                        if val.starts_with("'") && val.ends_with("'") || val.starts_with('"') && val.ends_with('"') {
                            val.to_string()
                        } else {
                            format!("'{}'", val.trim_matches('"').trim_matches('\''))
                        }
                    } else if default_val.trim().starts_with("const ") {
                        default_val.trim().to_string()
                    } else {
                        default_val.trim().to_string()
                    };
                    code.push_str(&format!("    this.{} = {},\n", field.name, dart_default));
                } else {
                    code.push_str(&format!("    this.{},\n", field.name));
                }
            } else {
                code.push_str(&format!("    required this.{},\n", field.name));
            }
        }
        code.push_str("  });\n\n");
        
        // copyWith - use correct syntax
        code.push_str(&format!("  @override\n  {} copyWith({{", class.name));
        for (i, field) in fields.iter().enumerate() {
            if i > 0 { code.push_str(", "); }
            let param_type = if field.ty.ends_with('?') {
                field.ty.clone()
            } else {
                format!("{}?", field.ty)
            };
            code.push_str(&format!("{} {}", param_type, field.name));
        }
        code.push_str("}) {\n");
        code.push_str(&format!("    return _{}(", class.name));
        for (i, field) in fields.iter().enumerate() {
            if i > 0 { code.push_str(", "); }
            code.push_str(&format!("{}: {} ?? this.{}", field.name, field.name, field.name));
        }
        code.push_str(");\n");
        code.push_str("  }\n");
        code.push_str("  @override\n");
        code.push_str("  String toString() {\n");
        code.push_str(&format!("    return '{}(", class.name));
        for (i, field) in fields.iter().enumerate() {
            if i > 0 { code.push_str(", "); }
            code.push_str(&format!("{}: ${}", field.name, field.name));
        }
        code.push_str(")';\n");
        code.push_str("  }\n");
        code.push_str("  @override\n");
        code.push_str("  bool operator ==(Object other) {\n");
        code.push_str(&format!("    return identical(this, other) ||\n"));
        code.push_str(&format!("        other is _{} &&\n", class.name));
        for (i, field) in fields.iter().enumerate() {
            if i > 0 { code.push_str(" &&\n"); }
            code.push_str(&format!("        {} == other.{}", field.name, field.name));
        }
        code.push_str(";\n");
        code.push_str("  }\n");
        code.push_str("  @override\n");
        code.push_str("  int get hashCode => ");
        for (i, field) in fields.iter().enumerate() {
            if i > 0 { code.push_str(" ^ "); }
            code.push_str(&format!("{}.hashCode", field.name));
        }
        if fields.is_empty() {
            code.push_str("0");
        }
        code.push_str(";\n\n");
        code.push_str(&format!("  @override\n  Map<String, dynamic> toJson() => _${}ToJson(this);\n", class.name));
        code.push_str("}\n\n");
    }
    
    code
}

pub fn generate_copy_with(class_name: &str, fields: &[DartField]) -> String {
    let mut code = String::new();
    code.push_str(&format!("  {} copyWith({{\n", class_name));
    for field in fields {
        let param_type = if field.ty.ends_with('?') {
            field.ty.clone()
        } else {
            format!("{}?", field.ty)
        };
        code.push_str(&format!("    {} {},\n", param_type, field.name));
    }
    code.push_str("  }) {\n");
    code.push_str(&format!("    return _{}({{", class_name));
    for (i, field) in fields.iter().enumerate() {
        if i > 0 { code.push_str(", "); }
        code.push_str(&format!("{}: {} ?? this.{}", field.name, field.name, field.name));
    }
    code.push_str(");\n");
    code.push_str("  }\n");
    code
}

pub fn generate_to_string(class_name: &str, fields: &[DartField]) -> String {
    let mut code = String::new();
    code.push_str("  @override\n");
    code.push_str(&format!("  String toString() {{\n"));
    code.push_str(&format!("    return '{}(", class_name));
    for (i, field) in fields.iter().enumerate() {
        if i > 0 { code.push_str(", "); }
        code.push_str(&format!("{}: ${}", field.name, field.name));
    }
    code.push_str(")';\n");
    code.push_str("  }\n");
    code
}

pub fn generate_eq(class_name: &str, fields: &[DartField]) -> String {
    let mut code = String::new();
    code.push_str("  @override\n");
    code.push_str("  bool operator ==(Object other) {\n");
    code.push_str(&format!("    return identical(this, other) ||\n"));
    code.push_str(&format!("        other is _{} &&\n", class_name));
    for (i, field) in fields.iter().enumerate() {
        if i > 0 { code.push_str(" &&\n"); }
        code.push_str(&format!("        {} == other.{}", field.name, field.name));
    }
    code.push_str(";\n");
    code.push_str("  }\n");
    code
}

pub fn generate_hash_code(fields: &[DartField]) -> String {
    let mut code = String::new();
    code.push_str("  @override\n");
    code.push_str("  int get hashCode => ");
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

pub fn generate_freezed_file(file_path: &Path, classes: &[DartClass]) -> Option<GenerationResult> {
    eprintln!("[DEBUG] generate_freezed_file called for {}", file_path.display());
    
    if classes.is_empty() {
        eprintln!("[DEBUG] No classes found in {}", file_path.display());
        return None;
    }

    // Generate both .freezed.dart and .g.dart files
    let mut all_generated_code = String::new();
    let file_stem = file_path.file_stem().unwrap().to_string_lossy();
    
    // Generate .freezed.dart content
    all_generated_code.push_str("// GENERATED CODE - DO NOT MODIFY BY HAND\n");
    all_generated_code.push_str("// **************************************************************************\n");
    all_generated_code.push_str("// FreezedGenerator\n");
    all_generated_code.push_str("// **************************************************************************\n\n");
    
    // Fix part directive to include .dart extension
    all_generated_code.push_str(&format!("part of '{}';\n\n", format!("{}.dart", file_stem)));
    
    // Generate code for each class
    for class in classes {
        all_generated_code.push_str(&generate_freezed_code(class));
        all_generated_code.push_str("\n");
    }
    
    // Generate .g.dart content
    let mut g_dart_code = String::new();
    g_dart_code.push_str("// GENERATED CODE - DO NOT MODIFY BY HAND\n");
    g_dart_code.push_str("// **************************************************************************\n");
    g_dart_code.push_str("// JsonSerializableGenerator\n");
    g_dart_code.push_str("// **************************************************************************\n\n");
    
    // Fix part directive to include .dart extension
    g_dart_code.push_str(&format!("part of '{}';\n\n", format!("{}.dart", file_stem)));
    
    // Generate JSON serialization code for each class
    for class in classes {
        g_dart_code.push_str(&generate_json_code(class));
        g_dart_code.push_str("\n");
    }
    
    Some(GenerationResult {
        freezed_code: all_generated_code,
        g_dart_code,
    })
}

pub fn generate_json_code(class: &DartClass) -> String {
    eprintln!("[DEBUG] generate_json_code called for {}", class.name);
    let mut code = String::new();
    
    let source_content = std::fs::read_to_string(&class.file_path).unwrap_or_default();
    let union_cases = extract_union_cases_from_dart_class(&source_content, &class.name);
    let fields = extract_fields_from_dart_class(&source_content, &class.name);

    if union_cases.len() > 1 {
        // Union type: Generate JSON functions for each case
        for case in &union_cases {
            let case_class_name = format!("{}{}", class.name, to_pascal_case(&case.case_name));
            code.push_str(&format!("Map<String, dynamic> _${}ToJson({} instance) {{\n", case_class_name, case_class_name));
            code.push_str("  return {\n");
            code.push_str(&format!("    'runtimeType': '{}',\n", case.case_name));
            for field in &case.fields {
                code.push_str(&format!("    '{}': instance.{},\n", field.name, field.name));
            }
            code.push_str("  };\n");
            code.push_str("}\n\n");
            
            code.push_str(&format!("{} _${}FromJson(Map<String, dynamic> json) {{\n", case_class_name, case_class_name));
            code.push_str(&format!("  return {}(\n", case_class_name));
            for field in &case.fields {
                code.push_str(&format!("    json['{}'] as {},\n", field.name, field.ty));
            }
            code.push_str("  );\n");
            code.push_str("}\n\n");
        }
        
        // Generate the main fromJson function for the union type
        code.push_str(&format!("{} _${}FromJson(Map<String, dynamic> json) {{\n", class.name, class.name));
        code.push_str("  switch (json['runtimeType'] as String) {\n");
        for case in &union_cases {
            let case_class_name = format!("{}{}", class.name, to_pascal_case(&case.case_name));
            code.push_str(&format!("    case '{}':\n", case.case_name));
            code.push_str(&format!("      return _${}FromJson(json);\n", case_class_name));
        }
        code.push_str(&format!("    default:\n"));
        code.push_str(&format!("      throw CheckedFromJsonException(json, '{}', '{}', 'Invalid union type');\n", class.name, class.name));
        code.push_str("  }\n");
        code.push_str("}\n\n");
        
        // Don't generate main toJson function for union types since they don't have a direct toJson method
        // Each union case has its own toJson method
    } else {
        // Regular class: Generate JSON functions
        code.push_str(&format!("Map<String, dynamic> _${}ToJson(_{} instance) {{\n", class.name, class.name));
        code.push_str("  return {\n");
        for field in &fields {
            code.push_str(&format!("    '{}': instance.{},\n", field.name, field.name));
        }
        code.push_str("  };\n");
        code.push_str("}\n\n");
        
        code.push_str(&format!("{} _${}FromJson(Map<String, dynamic> json) {{\n", class.name, class.name));
        code.push_str(&format!("  return _{}(", class.name));
        for (i, field) in fields.iter().enumerate() {
            if i > 0 { code.push_str(", "); }
            code.push_str(&format!("{}: json['{}'] as {}", field.name, field.name, field.ty));
        }
        code.push_str(");\n");
        code.push_str("}\n\n");
    }
    
    code
}

pub fn extract_fields_from_dart_class(source_content: &str, class_name: &str) -> Vec<DartField> {
    eprintln!("[DEBUG] extract_fields_from_dart_class called for {}", class_name);
    let mut fields = Vec::new();
    // Find the main constructor for this class
    let constructor_pattern = format!("const factory {}({{", class_name);
    if let Some(constructor_start) = source_content.find(&constructor_pattern) {
        eprintln!("[DEBUG] Found constructor at position {}", constructor_start);
        // Find the closing brace of the constructor parameters
        let mut brace_count = 0;
        let mut in_constructor = false;
        let mut constructor_content = String::new();
        for (i, ch) in source_content[constructor_start..].chars().enumerate() {
            if ch == '{' {
                brace_count += 1;
                in_constructor = true;
            } else if ch == '}' {
                brace_count -= 1;
                if in_constructor && brace_count == 0 {
                    // Found the end of constructor parameters
                    constructor_content = source_content[constructor_start..constructor_start + i + 1].to_string();
                    break;
                }
            }
        }
        eprintln!("[DEBUG] Constructor content: {}", constructor_content);
        // Extract parameters from the constructor content
        if let Some(start_brace) = constructor_content.find('{') {
            if let Some(end_brace) = constructor_content.rfind('}') {
                let params_content = &constructor_content[start_brace + 1..end_brace];
                eprintln!("[DEBUG] Parameters content: {}", params_content);
                // Split parameters by comma, but be careful with nested braces and comments
                let mut params = Vec::new();
                let mut current_param = String::new();
                let mut brace_count = 0;
                let mut paren_count = 0;
                let mut in_comment = false;
                let mut comment_type = None; // '//' or '/*'
                for ch in params_content.chars() {
                    match ch {
                        '{' => {
                            if !in_comment {
                                brace_count += 1;
                            }
                            current_param.push(ch);
                        }
                        '}' => {
                            if !in_comment {
                                brace_count -= 1;
                            }
                            current_param.push(ch);
                        }
                        '(' => {
                            if !in_comment {
                                paren_count += 1;
                            }
                            current_param.push(ch);
                        }
                        ')' => {
                            if !in_comment {
                                paren_count -= 1;
                            }
                            current_param.push(ch);
                        }
                        '/' => {
                            current_param.push(ch);
                            // Check for comment start
                            if !in_comment {
                                if current_param.ends_with("//") {
                                    in_comment = true;
                                    comment_type = Some("//");
                                } else if current_param.ends_with("/*") {
                                    in_comment = true;
                                    comment_type = Some("/*");
                                }
                            }
                        }
                        '*' => {
                            current_param.push(ch);
                            // Check for comment end
                            if in_comment && comment_type == Some("/*") && current_param.ends_with("*/") {
                                in_comment = false;
                                comment_type = None;
                            }
                        }
                        '\n' => {
                            if in_comment && comment_type == Some("//") {
                                in_comment = false;
                                comment_type = None;
                            }
                            current_param.push(ch);
                        }
                        ',' => {
                            if brace_count == 0 && paren_count == 0 && !in_comment {
                                let trimmed = current_param.trim();
                                if !trimmed.is_empty() {
                                    params.push(trimmed.to_string());
                                }
                                current_param.clear();
                            } else {
                                current_param.push(ch);
                            }
                        }
                        _ => current_param.push(ch),
                    }
                }
                // Add the last parameter if it exists
                let trimmed = current_param.trim();
                if !trimmed.is_empty() {
                    params.push(trimmed.to_string());
                }
                // Post-process parameters to handle multi-line parameters
                let mut processed_params = Vec::new();
                for param in params {
                    let lines: Vec<&str> = param.lines().collect();
                    let mut processed_param = String::new();
                    for line in lines {
                        let trimmed_line = line.trim();
                        // Skip comment-only lines
                        if trimmed_line.starts_with("//") || trimmed_line.starts_with("/*") {
                            continue;
                        }
                        // Skip empty lines
                        if trimmed_line.is_empty() {
                            continue;
                        }
                        // Skip standalone comment words
                        let comment_words = ["draft", "published", "cancelled", "completed", "pending", "succeeded", "failed"];
                        if comment_words.iter().any(|&word| trimmed_line == word) {
                            continue;
                        }
                        if !processed_param.is_empty() {
                            processed_param.push(' ');
                        }
                        processed_param.push_str(trimmed_line);
                    }
                    if !processed_param.is_empty() {
                        processed_params.push(processed_param);
                    }
                }
                params = processed_params;
                eprintln!("[DEBUG] Extracted {} parameters", params.len());
                // Process each parameter
                for param in params {
                    eprintln!("[DEBUG] Processing parameter: {}", param);
                    if let Some(field) = parse_dart_parameter(&param) {
                        let field_clone = field.clone();
                        fields.push(field);
                        eprintln!("[DEBUG] Added field: {} {}", field_clone.ty, field_clone.name);
                    }
                }
            }
        }
    }
    eprintln!("[DEBUG] Extracted {} fields for {}", fields.len(), class_name);
    for field in &fields {
        eprintln!("  {} {}", field.ty, field.name);
    }
    fields
}

fn parse_dart_parameter(param: &str) -> Option<DartField> {
    let param = param.trim();
    // Skip comments and empty parameters
    if param.starts_with("//") || param.starts_with("/*") || param.is_empty() {
        return None;
    }
    // Skip lines that are just comments or comment fragments
    if param.chars().all(|c| c.is_whitespace() || c == '/') {
        return None;
    }
    // Skip standalone comment words like "draft", "published", "cancelled", "completed"
    let comment_words = ["draft", "published", "cancelled", "completed", "pending", "succeeded", "failed"];
    if comment_words.iter().any(|&word| param == word) {
        return None;
    }
    // Skip if the parameter contains comment fragments at the beginning
    if comment_words.iter().any(|&word| param.starts_with(word)) {
        return None;
    }
    // Parse type, name, and default value
    let mut ty = String::new();
    let mut name = String::new();
    let is_named = false;
    let mut has_default = false;
    let mut default_value = None;
    let mut param = param.to_string();
    // Remove @Default annotation
    if let Some(default_start) = param.find("@Default(") {
        if let Some(default_end) = param[default_start..].find(')') {
            let default_val = &param[default_start + 9..default_start + default_end];
            has_default = true;
            default_value = Some(default_val.trim().to_string());
            // Remove the @Default(...) part
            let before = &param[..default_start];
            let after = &param[default_start + default_end + 1..];
            param = format!("{}{}", before, after).trim().to_string();
        }
    }
    // Remove required keyword
    let param = param.trim_start_matches("required ").trim();
    // Parse type and name
    let mut parts = param.split_whitespace();
    if let Some(first) = parts.next() {
        ty = first.to_string();
        if let Some(second) = parts.next() {
            name = second.trim_end_matches(',').to_string();
        }
    }
    if ty.is_empty() || name.is_empty() {
        return None;
    }
    Some(DartField {
        name,
        ty,
        is_named,
        has_default,
        default_value,
    })
}

pub fn extract_union_cases_from_dart_class(source_content: &str, class_name: &str) -> Vec<CaseInfo> {
    eprintln!("[DEBUG] extract_union_cases_from_dart_class called for {}", class_name);
    let mut cases = Vec::new();
    let lines: Vec<&str> = source_content.lines().collect();
    let mut in_class = false;
    let mut brace_count = 0;
    let mut in_factory = false;
    let mut factory_lines = Vec::new();
    for line in lines.iter() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("class {}", class_name)) {
            in_class = true;
            brace_count = 0;
            brace_count += trimmed.chars().filter(|&c| c == '{').count();
            brace_count -= trimmed.chars().filter(|&c| c == '}').count();
            continue;
        }
        if in_class {
            brace_count += trimmed.chars().filter(|&c| c == '{').count();
            brace_count -= trimmed.chars().filter(|&c| c == '}').count();
            if brace_count <= 0 {
                break;
            }
            if trimmed.contains("const factory") {
                in_factory = true;
                factory_lines.clear();
            }
            if in_factory {
                factory_lines.push(trimmed);
                if trimmed.contains(")") || trimmed.contains(";" ) {
                    let factory_decl = factory_lines.join(" ");
                    in_factory = false;
                    if let Some(dot_pos) = factory_decl.find(&format!("{}.", class_name)) {
                        let after_dot = &factory_decl[dot_pos + class_name.len() + 1..];
                        if let Some(paren_pos) = after_dot.find('(') {
                            let case_name = &after_dot[..paren_pos].trim();
                            eprintln!("[DEBUG] Case name: {}", case_name);
                            let mut params_content = String::new();
                            let mut paren_level = 0;
                            let mut found_start = false;
                            for ch in after_dot[paren_pos..].chars() {
                                if ch == '(' {
                                    paren_level += 1;
                                    found_start = true;
                                    if paren_level == 1 { continue; }
                                }
                                if ch == ')' {
                                    paren_level -= 1;
                                    if paren_level == 0 { break; }
                                }
                                if found_start && paren_level >= 1 {
                                    params_content.push(ch);
                                }
                            }
                            let mut case_fields = Vec::new();
                            if !params_content.trim().is_empty() {
                                let mut params = Vec::new();
                                let mut current_param = String::new();
                                let mut brace_count = 0;
                                let mut paren_count = 0;
                                for ch in params_content.chars() {
                                    match ch {
                                        '{' => { brace_count += 1; current_param.push(ch); }
                                        '}' => { brace_count -= 1; current_param.push(ch); }
                                        '(' => { paren_count += 1; current_param.push(ch); }
                                        ')' => { paren_count -= 1; current_param.push(ch); }
                                        ',' => {
                                            if brace_count == 0 && paren_count == 0 {
                                                if !current_param.trim().is_empty() {
                                                    params.push(current_param.trim().to_string());
                                                }
                                                current_param.clear();
                                            } else {
                                                current_param.push(ch);
                                            }
                                        }
                                        _ => current_param.push(ch),
                                    }
                                }
                                if !current_param.trim().is_empty() {
                                    params.push(current_param.trim().to_string());
                                }
                                for param in params {
                                    let param_trimmed = param.trim();
                                    if param_trimmed.is_empty() || param_trimmed.starts_with("//") {
                                        continue;
                                    }
                                    eprintln!("[DEBUG] Processing union case parameter: {}", param_trimmed);
                                    if let Some(field) = parse_dart_parameter(param_trimmed) {
                                        if !case_fields.iter().any(|f: &DartField| f.name == field.name) {
                                            let field_clone = field.clone();
                                            case_fields.push(field);
                                            eprintln!("[DEBUG] Added union case field: {} {}", field_clone.ty, field_clone.name);
                                        }
                                    }
                                }
                            }
                            cases.push(CaseInfo {
                                case_name: case_name.to_string(),
                                fields: case_fields,
                            });
                        } else {
                            cases.push(CaseInfo {
                                case_name: after_dot.trim().to_string(),
                                fields: Vec::new(),
                            });
                        }
                    }
                }
            }
        }
    }
    eprintln!("[DEBUG] union cases for {}:", class_name);
    for case in &cases {
        eprintln!("  case: {}", case.case_name);
        for field in &case.fields {
            eprintln!("    field: {} {}", field.ty, field.name);
        }
    }
    cases
}

pub fn get_safe_output_paths(file_path: &Path) -> (PathBuf, PathBuf) {
    let file_stem = file_path.file_stem().unwrap().to_string_lossy();
    let base_name = if file_stem.ends_with(".freezed") {
        &file_stem[..file_stem.len() - 8]
    } else if file_stem.ends_with(".g") {
        &file_stem[..file_stem.len() - 2]
    } else {
        &file_stem
    };
    let mut freezed_output_path = file_path.parent().unwrap().to_path_buf();
    freezed_output_path.push(format!("{}.freezed.dart", base_name));
    let mut g_dart_output_path = file_path.parent().unwrap().to_path_buf();
    g_dart_output_path.push(format!("{}.g.dart", base_name));
    (freezed_output_path, g_dart_output_path)
}

// Helper function for converting to PascalCase
fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    
    for ch in s.chars() {
        if ch.is_alphanumeric() {
            if capitalize_next {
                result.push(ch.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                result.push(ch.to_ascii_lowercase());
            }
        } else {
            capitalize_next = true;
        }
    }
    
    result
} 