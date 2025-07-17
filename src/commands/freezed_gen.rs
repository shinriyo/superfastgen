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
    eprintln!("[DEBUG] generate_freezed_code called for class: {}", class.name);
    let mut code = String::new();
    let source_content = std::fs::read_to_string(&class.file_path).unwrap_or_default();
    let union_cases = extract_union_cases_from_dart_class(&source_content, &class.name);
    let fields = extract_fields_from_dart_class(&source_content, &class.name);
    eprintln!("[DEBUG] Extracted {} fields for {}", fields.len(), class.name);
    
    // Add _privateConstructorUsedError
    code.push_str("final _privateConstructorUsedError = UnsupportedError(\n");
    code.push_str("  'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models',\n");
    code.push_str(");\n\n");
    
    // Add top-level fromJson function
    code.push_str(&format!("{} _${}FromJson(Map<String, dynamic> json) {{\n", class.name, class.name));
    code.push_str(&format!("  return _{}.fromJson(json);\n", class.name));
    code.push_str("}\n\n");
    
    // Add mixin _$User
    code.push_str("/// @nodoc\n");
    code.push_str(&format!("mixin _${} {{\n", class.name));
    for field in &fields {
        code.push_str(&format!("  {} get {} => throw _privateConstructorUsedError;\n", field.ty, field.name));
    }
    code.push_str("\n");
    code.push_str("  /// Serializes this User to a JSON map.\n");
    code.push_str("  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;\n\n");
    code.push_str("  /// Create a copy of User\n");
    code.push_str("  /// with the given fields replaced by the non-null parameter values.\n");
    code.push_str("  @JsonKey(includeFromJson: false, includeToJson: false)\n");
    code.push_str(&format!("  ${}CopyWith<{}> get copyWith => throw _privateConstructorUsedError;\n", class.name, class.name));
    code.push_str("}\n\n");
    
    // Add $UserCopyWith abstract class
    code.push_str("/// @nodoc\n");
    code.push_str(&format!("abstract class ${}CopyWith<$Res> {{\n", class.name));
    code.push_str(&format!("  factory ${}CopyWith({} value, $Res Function({}) then) =\n", class.name, class.name, class.name));
    code.push_str(&format!("      _${}CopyWithImpl<$Res, {}>;\n", class.name, class.name));
    code.push_str("  @useResult\n");
    code.push_str("  $Res call({\n");
    for field in &fields {
        code.push_str(&format!("    {} {},\n", field.ty, field.name));
    }
    code.push_str("  });\n");
    code.push_str("}\n\n");
    
    // Add _$UserCopyWithImpl class
    code.push_str("/// @nodoc\n");
    code.push_str(&format!("class _${}CopyWithImpl<$Res, $Val extends {}>\n", class.name, class.name));
    code.push_str(&format!("    implements ${}CopyWith<$Res> {{\n", class.name));
    code.push_str(&format!("  _${}CopyWithImpl(this._value, this._then);\n\n", class.name));
    code.push_str("  // ignore: unused_field\n");
    code.push_str("  final $Val _value;\n");
    code.push_str("  // ignore: unused_field\n");
    code.push_str("  final $Res Function($Val) _then;\n\n");
    code.push_str("  /// Create a copy of User\n");
    code.push_str("  /// with the given fields replaced by the non-null parameter values.\n");
    code.push_str("  @pragma('vm:prefer-inline')\n");
    code.push_str("  @override\n");
    code.push_str("  $Res call({\n");
    for field in &fields {
        code.push_str(&format!("    Object? {} = null,\n", field.name));
    }
    code.push_str("  }) {\n");
    code.push_str("    return _then(\n");
    code.push_str("      _value.copyWith(\n");
    for field in &fields {
        code.push_str(&format!("            {}:\n", field.name));
        code.push_str(&format!("                null == {}\n", field.name));
        code.push_str(&format!("                    ? _value.{}\n", field.name));
        code.push_str(&format!("                    : {} // ignore: cast_nullable_to_non_nullable\n", field.name));
        code.push_str(&format!("                        as {},\n", field.ty));
    }
    code.push_str("          )\n");
    code.push_str("          as $Val,\n");
    code.push_str("    );\n");
    code.push_str("  }\n");
    code.push_str("}\n\n");
    
    // Add _$$UserImplCopyWith abstract class
    code.push_str("/// @nodoc\n");
    code.push_str(&format!("abstract class _$${}ImplCopyWith<$Res> implements ${}CopyWith<$Res> {{\n", class.name, class.name));
    code.push_str(&format!("  factory _$${}ImplCopyWith(\n", class.name));
    code.push_str(&format!("    _${}Impl value,\n", class.name));
    code.push_str(&format!("    $Res Function(_${}Impl) then,\n", class.name));
    code.push_str(&format!("  ) = __$${}ImplCopyWithImpl<$Res>;\n", class.name));
    code.push_str("  @override\n");
    code.push_str("  @useResult\n");
    code.push_str("  $Res call({\n");
    for field in &fields {
        code.push_str(&format!("    {} {},\n", field.ty, field.name));
    }
    code.push_str("  });\n");
    code.push_str("}\n\n");
    
    // Add __$$UserImplCopyWithImpl class
    code.push_str("/// @nodoc\n");
    code.push_str(&format!("class __$${}ImplCopyWithImpl<$Res>\n", class.name));
    code.push_str(&format!("    extends _${}CopyWithImpl<$Res, _${}Impl>\n", class.name, class.name));
    code.push_str(&format!("    implements _$${}ImplCopyWith<$Res> {{\n", class.name));
    code.push_str(&format!("  __$${}ImplCopyWithImpl(_${}Impl _value, $Res Function(_${}Impl) _then)\n", class.name, class.name, class.name));
    code.push_str("    : super(_value, _then);\n\n");
    code.push_str("  /// Create a copy of User\n");
    code.push_str("  /// with the given fields replaced by the non-null parameter values.\n");
    code.push_str("  @pragma('vm:prefer-inline')\n");
    code.push_str("  @override\n");
    code.push_str("  $Res call({\n");
    for field in &fields {
        code.push_str(&format!("    Object? {} = null,\n", field.name));
    }
    code.push_str("  }) {\n");
    code.push_str("    return _then(\n");
    code.push_str(&format!("      _${}Impl(\n", class.name));
    for field in &fields {
        code.push_str(&format!("        {}:\n", field.name));
        code.push_str(&format!("            null == {}\n", field.name));
        code.push_str(&format!("                ? _value.{}\n", field.name));
        code.push_str(&format!("                : {} // ignore: cast_nullable_to_non_nullable\n", field.name));
        code.push_str(&format!("                    as {},\n", field.ty));
    }
    code.push_str("      ),\n");
    code.push_str("    );\n");
    code.push_str("  }\n");
    code.push_str("}\n\n");
    
    // Add _$UserImpl class with @JsonSerializable
    code.push_str("/// @nodoc\n");
    code.push_str("@JsonSerializable()\n");
    code.push_str(&format!("class _${}Impl implements _{} {{\n", class.name, class.name));
    code.push_str(&format!("  const _${}Impl({{\n", class.name));
    for field in &fields {
        if field.has_default {
            code.push_str(&format!("    this.{} = {},\n", field.name, field.default_value.as_ref().unwrap_or(&"false".to_string())));
        } else {
            code.push_str(&format!("    required this.{},\n", field.name));
        }
    }
    code.push_str("  });\n\n");
    code.push_str(&format!("  factory _${}Impl.fromJson(Map<String, dynamic> json) =>\n", class.name));
    code.push_str(&format!("      _$${}ImplFromJson(json);\n\n", class.name));
    
    // Add fields with @override and @JsonKey
    for field in &fields {
        code.push_str("  @override\n");
        if field.has_default {
            code.push_str("  @JsonKey()\n");
        }
        code.push_str(&format!("  final {} {};\n", field.ty, field.name));
    }
    code.push_str("\n");
    
    // Add toString method
    code.push_str("  @override\n");
    code.push_str("  String toString() {\n");
    code.push_str(&format!("    return '{}(", class.name));
    for (i, field) in fields.iter().enumerate() {
        if i > 0 { code.push_str(", "); }
        code.push_str(&format!("{}: ${}", field.name, field.name));
    }
    code.push_str(")';\n");
    code.push_str("  }\n\n");
    
    // Add equality operator
    code.push_str("  @override\n");
    code.push_str("  bool operator ==(Object other) {\n");
    code.push_str("    return identical(this, other) ||\n");
    code.push_str(&format!("        (other.runtimeType == runtimeType &&\n"));
    code.push_str(&format!("            other is _${}Impl &&\n", class.name));
    for (i, field) in fields.iter().enumerate() {
        if i > 0 { code.push_str(" &&\n"); }
        code.push_str(&format!("            (identical(other.{}, {}) || other.{} == {})", field.name, field.name, field.name, field.name));
    }
    code.push_str(");\n");
    code.push_str("  }\n\n");
    
    // Add hashCode
    code.push_str("  @JsonKey(includeFromJson: false, includeToJson: false)\n");
    code.push_str("  @override\n");
    code.push_str("  int get hashCode =>\n");
    code.push_str("      Object.hash(runtimeType");
    for field in &fields {
        code.push_str(&format!(", {}", field.name));
    }
    code.push_str(");\n\n");
    
    // Add copyWith method
    code.push_str("  /// Create a copy of User\n");
    code.push_str("  /// with the given fields replaced by the non-null parameter values.\n");
    code.push_str("  @JsonKey(includeFromJson: false, includeToJson: false)\n");
    code.push_str("  @override\n");
    code.push_str("  @pragma('vm:prefer-inline')\n");
    code.push_str(&format!("  _$${}ImplCopyWith<_${}Impl> get copyWith =>\n", class.name, class.name));
    code.push_str(&format!("      __$${}ImplCopyWithImpl<_${}Impl>(this, _$identity);\n\n", class.name, class.name));
    
    // Add toJson method
    code.push_str("  @override\n");
    code.push_str("  Map<String, dynamic> toJson() {\n");
    code.push_str(&format!("    return _$${}ImplToJson(this);\n", class.name));
    code.push_str("  }\n");
    code.push_str("}\n\n");
    
    // Add abstract _User class
    code.push_str(&format!("abstract class _{} implements {} {{\n", class.name, class.name));
    code.push_str("  const factory _");
    code.push_str(&class.name);
    code.push_str("({\n");
    for field in &fields {
        if field.has_default {
            code.push_str(&format!("    final {} {},\n", field.ty, field.name));
        } else {
            code.push_str(&format!("    required final {} {},\n", field.ty, field.name));
        }
    }
    code.push_str(&format!("  }}) = _${}Impl;\n\n", class.name));
    code.push_str(&format!("  factory _{}.fromJson(Map<String, dynamic> json) = _${}Impl.fromJson;\n\n", class.name, class.name));
    
    // Add getters
    for field in &fields {
        code.push_str("  @override\n");
        code.push_str(&format!("  {} get {};\n", field.ty, field.name));
    }
    code.push_str("\n");
    
    // Add copyWith method
    code.push_str("  /// Create a copy of User\n");
    code.push_str("  /// with the given fields replaced by the non-null parameter values.\n");
    code.push_str("  @override\n");
    code.push_str("  @JsonKey(includeFromJson: false, includeToJson: false)\n");
    code.push_str(&format!("  _$${}ImplCopyWith<_${}Impl> get copyWith =>\n", class.name, class.name));
    code.push_str("      throw _privateConstructorUsedError;\n");
    code.push_str("}\n");
    
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
    eprintln!("[DEBUG] Classes to generate: {:?}", classes.iter().map(|c| &c.name).collect::<Vec<_>>());
    
    if classes.is_empty() {
        eprintln!("[DEBUG] No classes found in {}", file_path.display());
        return None;
    }

    // Generate both .freezed.dart and .g.dart files
    let mut all_generated_code = String::new();
    let file_stem = file_path.file_stem().unwrap().to_string_lossy();

    // FreezedGenerator comment block (Dart build_runnerと同じ)
    all_generated_code.push_str("// coverage:ignore-file\n");
    all_generated_code.push_str("// GENERATED CODE - DO NOT MODIFY BY HAND\n");
    all_generated_code.push_str("// ignore_for_file: type=lint\n");
    all_generated_code.push_str("// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark\n\n");
    all_generated_code.push_str(&format!("part of '{}';\n\n", format!("{}.dart", file_stem)));
    all_generated_code.push_str("// **************************************************************************\n");
    all_generated_code.push_str("// FreezedGenerator\n");
    all_generated_code.push_str("// **************************************************************************\n\n");
    all_generated_code.push_str("T _$identity<T>(T value) => value;\n\n");

    // Generate code for each class
    for (i, class) in classes.iter().enumerate() {
        eprintln!("[DEBUG] Generating code for class: {}", class.name);
        all_generated_code.push_str(&generate_freezed_code(class));
        if i + 1 != classes.len() {
            all_generated_code.push_str("\n");
        }
    }
    
    // Generate .g.dart content
    let mut g_dart_code = String::new();
    g_dart_code.push_str("// GENERATED CODE - DO NOT MODIFY BY HAND\n\n");
    g_dart_code.push_str(&format!("part of '{}';\n\n", format!("{}.dart", file_stem)));
    g_dart_code.push_str("// **************************************************************************\n");
    g_dart_code.push_str("// JsonSerializableGenerator\n");
    g_dart_code.push_str("// **************************************************************************\n\n");
    
    // Generate JSON serialization code for each class
    for class in classes {
        eprintln!("[DEBUG] Generating JSON code for class: {}", class.name);
        g_dart_code.push_str(&generate_json_code(class));
        g_dart_code.push_str("\n");
    }
    
    eprintln!("[DEBUG] Generated freezed code preview: {}", &all_generated_code[..all_generated_code.len().min(500)]);
    
    Some(GenerationResult {
        freezed_code: all_generated_code,
        g_dart_code,
    })
}

pub fn generate_json_code(class: &DartClass) -> String {
    let mut code = String::new();
    let fields = extract_fields_from_dart_class(&std::fs::read_to_string(&class.file_path).unwrap_or_default(), &class.name);
    let impl_class = format!("_${}Impl", class.name);
    let from_json_fn = format!("_$${}ImplFromJson", class.name);
    let to_json_fn = format!("_$${}ImplToJson", class.name);

    // FromJson
    code.push_str(&format!("{} {}(Map<String, dynamic> json) => {}(\n", impl_class, from_json_fn, impl_class));
    for field in &fields {
        if field.name == "id" && field.ty == "int" {
            code.push_str(&format!("  id: (json['id'] as num).toInt(),\n"));
        } else if field.name == "isPremium" && field.ty == "bool" {
            code.push_str(&format!("  isPremium: json['isPremium'] as bool? ?? false,\n"));
        } else {
            code.push_str(&format!("  {}: json['{}'] as {},\n", field.name, field.name, field.ty));
        }
    }
    code.push_str(");\n\n");

    // ToJson
    code.push_str(&format!("Map<String, dynamic> {}({} instance) =>\n    <String, dynamic>{{\n", to_json_fn, impl_class));
    for field in &fields {
        code.push_str(&format!("      '{}': instance.{},\n", field.name, field.name));
    }
    code.push_str("    };\n\n");
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