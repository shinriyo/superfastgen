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

/// Writes the _privateConstructorUsedError only once per file.
pub fn generate_freezed_file(file_path: &Path, classes: &[DartClass]) -> Option<GenerationResult> {
    let mut freezed_code = String::new();
    let mut g_dart_code = String::new();

    // デバッグ: クラス一覧を出力
    eprintln!("[DEBUG] classes to generate: {:?}", classes.iter().map(|c| &c.name).collect::<Vec<_>>());

    // Add Dart official header comments
    freezed_code.push_str("// coverage:ignore-file\n");
    freezed_code.push_str("// GENERATED CODE - DO NOT MODIFY BY HAND\n");
    freezed_code.push_str("// ignore_for_file: type=lint\n");
    freezed_code.push_str("// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark\n\n");
    
    // Add part of directive
    let file_stem = file_path.file_stem().unwrap().to_string_lossy();
    freezed_code.push_str(&format!("part of '{}';\n\n", format!("{}.dart", file_stem)));
    
    // Note: imports are not allowed in part files
    
    // Add FreezedGenerator comment block
    freezed_code.push_str("// **************************************************************************\n");
    freezed_code.push_str("// FreezedGenerator\n");
    freezed_code.push_str("// **************************************************************************\n\n");
    
    // Add T _$identity function
    freezed_code.push_str("T _$identity<T>(T value) => value;\n\n");
    
    // Add _privateConstructorUsedError only once at the top
    freezed_code.push_str("final _privateConstructorUsedError = UnsupportedError(\n");
    freezed_code.push_str("    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');\n\n");

    // クラスごとにfreezed_codeとg_dart_codeを分離してpush
    for class in classes {
        let class_code = generate_freezed_code(class);
        eprintln!("[DEBUG] Generated {} bytes for class: {}", class_code.len(), class.name);
        eprintln!("[DEBUG] Class code preview: {}", &class_code[..class_code.len().min(200)]);
        // freezed_codeにはクラス定義・mixin・copyWith・抽象クラスのみ
        freezed_code.push_str(&class_code);
    }
    
    // Generate .g.dart content
    let file_stem = file_path.file_stem().unwrap().to_string_lossy();
    g_dart_code.push_str("// GENERATED CODE - DO NOT MODIFY BY HAND\n\n");
    g_dart_code.push_str(&format!("part of '{}';\n\n", format!("{}.dart", file_stem)));
    g_dart_code.push_str("// **************************************************************************\n");
    g_dart_code.push_str("// JsonSerializableGenerator\n");
    g_dart_code.push_str("// **************************************************************************\n\n");
    
    // Generate JSON serialization code for each class
    let mut processed_classes = std::collections::HashSet::new();
    for class in classes {
        if !processed_classes.contains(&class.name) {
            eprintln!("[DEBUG] Generating JSON code for class: {}", class.name);
            g_dart_code.push_str(&generate_json_code(class));
            processed_classes.insert(class.name.clone());
        }
    }
    // 末尾の空行を1つだけにする
    while g_dart_code.ends_with("\n\n") {
        g_dart_code.pop();
    }
    if !g_dart_code.ends_with("\n") {
        g_dart_code.push('\n');
    }
    
    eprintln!("[DEBUG] Generated freezed code preview: {}", &freezed_code[..freezed_code.len().min(500)]);
    eprintln!("[DEBUG] Total freezed code length: {} bytes", freezed_code.len());
    
    Some(GenerationResult {
        freezed_code,
        g_dart_code,
    })
}

pub fn generate_freezed_code(class: &DartClass) -> String {
    eprintln!("[DEBUG] generate_freezed_code called for class: {}", class.name);
    let mut code = String::new();
    let source_content = std::fs::read_to_string(&class.file_path).unwrap_or_default();
    let union_cases = extract_union_cases_from_dart_class(&source_content, &class.name);
    let fields = extract_fields_from_dart_class(&source_content, &class.name);
    eprintln!("[DEBUG] Extracted {} fields for {}", fields.len(), class.name);
    eprintln!("[DEBUG] Extracted {} union cases for {}", union_cases.len(), class.name);
    

    
    // Check if this is a union type (sealed class)
    if !union_cases.is_empty() {
        // Generate union type code
        generate_union_type_code(&mut code, class, &union_cases, &fields);
    } else {
        // Generate regular class code
        generate_regular_class_code(class, &fields, &union_cases, &mut code);
    }
    
    code
}

fn generate_regular_class_code(class: &DartClass, fields: &[DartField], _union_cases: &[CaseInfo], code: &mut String) {

    // Add top-level fromJson function
    code.push_str(&format!("{} _${}FromJson(Map<String, dynamic> json) {{\n", class.name, class.name));
    code.push_str(&format!("  return _${}Impl.fromJson(json);\n", class.name));
    code.push_str("}\n\n");
    
    // Add mixin _$Event
    code.push_str("/// @nodoc\n");
    code.push_str(&format!("mixin _${} {{\n", class.name));
    
    // Add getters for all fields
    for field in fields {
        code.push_str(&format!("  {} get {} => throw _privateConstructorUsedError;\n", field.ty, field.name));
    }
    code.push_str("\n");
    
    // Add toJson method
    code.push_str("  /// Serializes this ");
    code.push_str(&class.name);
    code.push_str(" to a JSON map.\n");
    code.push_str("  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;\n\n");
    
    // Add copyWith method
    code.push_str("  /// Create a copy of ");
    code.push_str(&class.name);
    code.push_str("\n");
    code.push_str("  /// with the given fields replaced by the non-null parameter values.\n");
    code.push_str("  @JsonKey(includeFromJson: false, includeToJson: false)\n");
    code.push_str(&format!("  ${}CopyWith<{}> get copyWith => throw _privateConstructorUsedError;\n", class.name, class.name));
    code.push_str("}\n\n");
    
    // Generate $ClassCopyWith abstract class
    code.push_str("/// @nodoc\n");
    code.push_str(&format!("abstract class ${}CopyWith<$Res> {{\n", class.name));
    code.push_str(&format!("  factory ${}CopyWith({} value, $Res Function({}) then) =\n", class.name, class.name, class.name));
    code.push_str(&format!("      _${}CopyWithImpl<$Res, {}>;\n", class.name, class.name));
    code.push_str("  @useResult\n");
    code.push_str(&format!("  $Res call({{"));
    for field in fields {
        code.push_str(&format!("\n      {} {},", field.ty, field.name));
    }
    code.push_str("\n  });\n");
    code.push_str("}\n\n");
    
    // Generate _$ClassCopyWithImpl class
    code.push_str("/// @nodoc\n");
    code.push_str(&format!("class _${}CopyWithImpl<$Res, $Val extends {}>\n", class.name, class.name));
    code.push_str(&format!("    implements ${}CopyWith<$Res> {{\n", class.name));
    code.push_str(&format!("  _${}CopyWithImpl(this._value, this._then);\n", class.name));
    code.push_str("\n");
    code.push_str("  // ignore: unused_field\n");
    code.push_str("  final $Val _value;\n");
    code.push_str("  // ignore: unused_field\n");
    code.push_str("  final $Res Function($Val) _then;\n");
    code.push_str("  /// Create a copy of ");
    code.push_str(&class.name);
    code.push_str("\n");
    code.push_str("  /// with the given fields replaced by the non-null parameter values.\n");
    code.push_str("  @pragma('vm:prefer-inline')\n");
    code.push_str("  @override\n");
    code.push_str(&format!("  $Res call({{"));
    for field in fields {
        let field_type = if field.ty.ends_with('?') {
            "Object?".to_string()
        } else {
            "Object?".to_string()
        };
        let default_value = if field.ty.ends_with('?') {
            "freezed".to_string()
        } else {
            "null".to_string()
        };
        code.push_str(&format!("\n    {} {} = {},", field_type, field.name, default_value));
    }
    code.push_str("\n  }) {\n");
    code.push_str("    return _then(_value.copyWith(\n");
    for field in fields {
        if field.ty.ends_with('?') {
            code.push_str(&format!("      {}: freezed == {}\n", field.name, field.name));
            code.push_str(&format!("          ? _value.{}\n", field.name));
            code.push_str(&format!("          : {} // ignore: cast_nullable_to_non_nullable\n", field.name));
            code.push_str(&format!("              as {},\n", field.ty));
        } else {
            code.push_str(&format!("      {}: null == {}\n", field.name, field.name));
            code.push_str(&format!("          ? _value.{}\n", field.name));
            code.push_str(&format!("          : {} // ignore: cast_nullable_to_non_nullable\n", field.name));
            code.push_str(&format!("              as {},\n", field.ty));
        }
    }
    code.push_str("    ) as $Val);\n");
    code.push_str("  }\n");
    code.push_str("}\n\n");
    
    // Generate _$$$ClassImplImplCopyWith abstract class
    code.push_str("/// @nodoc\n");
    code.push_str(&format!("abstract class _$$${}ImplImplCopyWith<$Res> implements ${}CopyWith<$Res> {{\n", class.name, class.name));
    code.push_str(&format!("  factory _$$${}ImplImplCopyWith(\n", class.name));
    code.push_str(&format!("          _$${}ImplImpl value, $Res Function(_$${}ImplImpl) then) =\n", class.name, class.name));
    code.push_str(&format!("      __$$${}ImplImplCopyWithImpl<$Res>;\n", class.name));
    code.push_str("  @override\n");
    code.push_str("  @useResult\n");
    code.push_str(&format!("  $Res call({{"));
    for field in fields {
        let field_type = if field.ty.ends_with('?') {
            "Object?".to_string()
        } else {
            "Object?".to_string()
        };
        let default_value = if field.ty.ends_with('?') {
            "freezed".to_string()
        } else {
            "null".to_string()
        };
        code.push_str(&format!("\n    {} {} = {},", field_type, field.name, default_value));
    }
    code.push_str("\n  });\n");
    code.push_str("}\n\n");
    
    // Generate __$$$ClassImplImplCopyWithImpl class
    code.push_str("/// @nodoc\n");
    code.push_str(&format!("class __$$${}ImplImplCopyWithImpl<$Res>\n", class.name));
    code.push_str(&format!("    extends _${}CopyWithImpl<$Res, _$${}ImplImpl>\n", class.name, class.name));
    code.push_str(&format!("    implements _$$${}ImplImplCopyWith<$Res> {{\n", class.name));
    code.push_str(&format!("  __$$${}ImplImplCopyWithImpl(\n", class.name));
    code.push_str(&format!("      _$${}ImplImpl _value, $Res Function(_$${}ImplImpl) _then)\n", class.name, class.name));
    code.push_str("      : super(_value, _then);\n");
    code.push_str("\n");
    code.push_str("  /// Create a copy of ");
    code.push_str(&class.name);
    code.push_str("\n");
    code.push_str("  /// with the given fields replaced by the non-null parameter values.\n");
    code.push_str("  @pragma('vm:prefer-inline')\n");
    code.push_str("  @override\n");
    code.push_str(&format!("  $Res call({{"));
    for field in fields {
        let field_type = if field.ty.ends_with('?') {
            "Object?".to_string()
        } else {
            "Object?".to_string()
        };
        let default_value = if field.ty.ends_with('?') {
            "freezed".to_string()
        } else {
            "null".to_string()
        };
        code.push_str(&format!("\n    {} {} = {},", field_type, field.name, default_value));
    }
    code.push_str("\n  }) {\n");
    code.push_str(&format!("    return _then(_$${}ImplImpl(\n", class.name));
    for field in fields {
        if field.ty.ends_with('?') {
            code.push_str(&format!("      {}: freezed == {}\n", field.name, field.name));
            code.push_str(&format!("          ? _value.{}\n", field.name));
            code.push_str(&format!("          : {} // ignore: cast_nullable_to_non_nullable\n", field.name));
            code.push_str(&format!("              as {},\n", field.ty));
        } else {
            code.push_str(&format!("      {}: null == {}\n", field.name, field.name));
            code.push_str(&format!("          ? _value.{}\n", field.name));
            code.push_str(&format!("          : {} // ignore: cast_nullable_to_non_nullable\n", field.name));
            code.push_str(&format!("              as {},\n", field.ty));
        }
    }
    code.push_str("    ));\n");
    code.push_str("  }\n");
    code.push_str("}\n\n");
    
    // Generate _$$ClassImplImpl class
    code.push_str("/// @nodoc\n");
    code.push_str("@JsonSerializable()\n");
    code.push_str(&format!("class _$${}ImplImpl implements _${}Impl {{\n", class.name, class.name));
    
    // Generate constructor
    code.push_str(&format!("  const _$${}ImplImpl(\n", class.name));
    code.push_str("      {");
    for field in fields {
        if field.ty.ends_with('?') {
            code.push_str(&format!("this.{},", field.name));
        } else if field.has_default {
            if let Some(default_val) = &field.default_value {
                // For list fields with default, use const
                if field.ty.contains("List<") && default_val == "[]" {
                    code.push_str(&format!("this.{} = const {},", field.name, default_val));
                } else {
                    code.push_str(&format!("this.{} = {},", field.name, default_val));
                }
            } else {
                code.push_str(&format!("this.{},", field.name));
            }
        } else {
            code.push_str(&format!("required this.{},", field.name));
        }
    }
    code.push_str("});\n\n");
    
    // fromJson factory
    code.push_str(&format!("  factory _$${}ImplImpl.fromJson(Map<String, dynamic> json) =>\n", class.name));
    code.push_str(&format!("      _$${}ImplImplFromJson(json);\n\n", class.name));
    
    // Generate fields
    for field in fields {
        code.push_str(&format!("  @override\n"));
        code.push_str(&format!("  final {} {};\n", field.ty, field.name));
    }
    code.push_str("\n");
    
    // toString method
    code.push_str("  @override\n");
    code.push_str("  String toString() {\n");
    let field_names: Vec<String> = fields.iter().map(|f| format!("{}: ${}", f.name, f.name)).collect();
    code.push_str(&format!("    return '{}({})';\n", class.name, field_names.join(", ")));
    code.push_str("  }\n\n");
    
    // equality operator
    code.push_str("  @override\n");
    code.push_str("  bool operator ==(Object other) {\n");
    code.push_str("    return identical(this, other) ||\n");
    code.push_str(&format!("        (other.runtimeType == runtimeType &&\n"));
    code.push_str(&format!("            other is _$${}ImplImpl &&\n", class.name));
    for field in fields {
        if field.name == "tags" {
            code.push_str(&format!("            const DeepCollectionEquality().equals(other.tags, tags) &&\n"));
        } else if field.name == "attendees" {
            code.push_str(&format!("            const DeepCollectionEquality()\n"));
            code.push_str(&format!("                .equals(other.attendees, attendees) &&\n"));
        } else {
            code.push_str(&format!("            (identical(other.{}, {}) || other.{} == {}) &&\n", field.name, field.name, field.name, field.name));
        }
    }
    // Remove the last " &&" and add closing parenthesis
    if code.ends_with(" &&\n") {
        code.truncate(code.len() - 4);
    }
    code.push_str(");\n");
    code.push_str("  }\n\n");
    
    // hashCode
    code.push_str("  @JsonKey(includeFromJson: false, includeToJson: false)\n");
    code.push_str("  @override\n");
    code.push_str("  int get hashCode => Object.hash(\n");
    code.push_str("      runtimeType,\n");
    for field in fields {
        if field.name == "tags" {
            code.push_str(&format!("      const DeepCollectionEquality().hash(tags),\n"));
        } else if field.name == "attendees" {
            code.push_str(&format!("      const DeepCollectionEquality().hash(attendees),\n"));
        } else {
            code.push_str(&format!("      {},\n", field.name));
        }
    }
    code.push_str("  );\n\n");
    
    // copyWith method
    code.push_str("  /// Create a copy of ");
    code.push_str(&class.name);
    code.push_str("\n");
    code.push_str("  /// with the given fields replaced by the non-null parameter values.\n");
    code.push_str("  @JsonKey(includeFromJson: false, includeToJson: false)\n");
    code.push_str("  @override\n");
    code.push_str("  @pragma('vm:prefer-inline')\n");
    code.push_str(&format!("  _$$${}ImplImplCopyWith<_$${}ImplImpl> get copyWith =>\n", class.name, class.name));
    code.push_str(&format!("      __$$${}ImplImplCopyWithImpl<_$${}ImplImpl>(this, _$identity);\n", class.name, class.name));
    code.push_str("\n");
    code.push_str("  @override\n");
    code.push_str("  Map<String, dynamic> toJson() {\n");
    code.push_str(&format!("    return _$${}ImplImplToJson(\n", class.name));
    code.push_str("      this,\n");
    code.push_str("    );\n");
    code.push_str("  }\n");
    code.push_str("}\n\n");
    
    // Generate abstract class _$EventImpl
    code.push_str(&format!("abstract class _${}Impl implements {} {{\n", class.name, class.name));
    code.push_str(&format!("  const factory _${}Impl(\n", class.name));
    code.push_str("    {\n");
    for field in fields {
        if field.ty.ends_with('?') || field.has_default {
            code.push_str(&format!("      final {} {},\n", field.ty, field.name));
        } else {
            code.push_str(&format!("      required final {} {},\n", field.ty, field.name));
        }
    }
    code.push_str(&format!("    }}\n  ) = _$${}ImplImpl;\n\n", class.name));
    code.push_str(&format!("  factory _${}Impl.fromJson(Map<String, dynamic> json) =\n", class.name));
    code.push_str(&format!("      _$${}ImplImpl.fromJson;\n\n", class.name));
    
    for field in fields {
        code.push_str(&format!("  @override\n"));
        code.push_str(&format!("  {} get {};\n", field.ty, field.name));
    }
    code.push_str("\n");
    
    code.push_str("  /// Create a copy of ");
    code.push_str(&class.name);
    code.push_str("\n");
    code.push_str("  /// with the given fields replaced by the non-null parameter values.\n");
    code.push_str("  @override\n");
    code.push_str("  @JsonKey(includeFromJson: false, includeToJson: false)\n");
    code.push_str(&format!("  _$$${}ImplImplCopyWith<_$${}ImplImpl> get copyWith =>\n", class.name, class.name));
    code.push_str("      throw _privateConstructorUsedError;\n");
    code.push_str("}\n\n");
    

}

fn generate_union_type_code(code: &mut String, class: &DartClass, union_cases: &[CaseInfo], fields: &[DartField]) {
    // Generate mixin with all the required methods
    code.push_str("/// @nodoc\n");
    code.push_str(&format!("mixin _${} {{\n", class.name));
    
    // Generate when method
    code.push_str("  @optionalTypeArgs\n");
    code.push_str("  TResult when<TResult extends Object?>({\n");
    for case in union_cases {
        if case.fields.is_empty() {
            code.push_str(&format!("    required TResult Function() {},\n", case.case_name));
        } else {
            let params: Vec<String> = case.fields.iter().map(|f| format!("{} {}", f.ty, f.name)).collect();
            code.push_str(&format!("    required TResult Function({}) {},\n", params.join(", "), case.case_name));
        }
    }
    code.push_str("  }) => throw _privateConstructorUsedError;\n");
    
    // Generate whenOrNull method
    code.push_str("  @optionalTypeArgs\n");
    code.push_str("  TResult? whenOrNull<TResult extends Object?>({\n");
    for case in union_cases {
        if case.fields.is_empty() {
            code.push_str(&format!("    TResult? Function()? {},\n", case.case_name));
        } else {
            let params: Vec<String> = case.fields.iter().map(|f| format!("{} {}", f.ty, f.name)).collect();
            code.push_str(&format!("    TResult? Function({})? {},\n", params.join(", "), case.case_name));
        }
    }
    code.push_str("  }) => throw _privateConstructorUsedError;\n");
    
    // Generate maybeWhen method
    code.push_str("  @optionalTypeArgs\n");
    code.push_str("  TResult maybeWhen<TResult extends Object?>({\n");
    for case in union_cases {
        if case.fields.is_empty() {
            code.push_str(&format!("    TResult Function()? {},\n", case.case_name));
        } else {
            let params: Vec<String> = case.fields.iter().map(|f| format!("{} {}", f.ty, f.name)).collect();
            code.push_str(&format!("    TResult Function({})? {},\n", params.join(", "), case.case_name));
        }
    }
    code.push_str("    required TResult orElse(),\n");
    code.push_str("  }) => throw _privateConstructorUsedError;\n");
    
    // Generate map method
    code.push_str("  @optionalTypeArgs\n");
    code.push_str("  TResult map<TResult extends Object?>({\n");
    for case in union_cases {
        let case_class_name = format!("{}{}", class.name, to_pascal_case(&case.case_name));
        code.push_str(&format!("    required TResult Function({}) {},\n", case_class_name, case.case_name));
    }
    code.push_str("  }) => throw _privateConstructorUsedError;\n");
    
    // Generate mapOrNull method
    code.push_str("  @optionalTypeArgs\n");
    code.push_str("  TResult? mapOrNull<TResult extends Object?>({\n");
    for case in union_cases {
        let case_class_name = format!("{}{}", class.name, to_pascal_case(&case.case_name));
        code.push_str(&format!("    TResult? Function({})? {},\n", case_class_name, case.case_name));
    }
    code.push_str("  }) => throw _privateConstructorUsedError;\n");
    
    // Generate maybeMap method
    code.push_str("  @optionalTypeArgs\n");
    code.push_str("  TResult maybeMap<TResult extends Object?>({\n");
    for case in union_cases {
        let case_class_name = format!("{}{}", class.name, to_pascal_case(&case.case_name));
        code.push_str(&format!("    TResult Function({})? {},\n", case_class_name, case.case_name));
    }
    code.push_str("    required TResult orElse(),\n");
    code.push_str("  }) => throw _privateConstructorUsedError;\n\n");
    
    code.push_str("  /// Serializes this ");
    code.push_str(&class.name);
    code.push_str(" to a JSON map.\n");
    code.push_str("  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;\n");
    code.push_str("}\n\n");
    
    // Generate toJson implementation for union types
    code.push_str("/// @nodoc\n");
    code.push_str(&format!("extension {}Extension on {} {{\n", class.name, class.name));
    code.push_str("  Map<String, dynamic> toJson() => when(\n");
    for case in union_cases {
        code.push_str(&format!("    {}: (", case.case_name));
        if case.fields.is_empty() {
            code.push_str(") => <String, dynamic>{\n");
            code.push_str(&format!("      'type': '{}',\n", case.case_name));
        } else {
            let params: Vec<String> = case.fields.iter().map(|f| f.name.clone()).collect();
            code.push_str(&format!("{}) => <String, dynamic>{{\n", params.join(", ")));
            code.push_str(&format!("      'type': '{}',\n", case.case_name));
            for field in &case.fields {
                code.push_str(&format!("      '{}': {},\n", field.name, field.name));
            }
        }
        code.push_str("    },\n");
    }
    code.push_str("  );\n");
    code.push_str("}\n\n");
    
    // Union types don't have copyWith - skip copyWith generation
    
    // Generate each union case implementation
    for case in union_cases {
        let case_class_name = format!("{}{}", class.name, to_pascal_case(&case.case_name));
        let impl_class_name = format!("_${}Impl", case_class_name);
        

        

        

        
        // Generate abstract class for this case
        code.push_str(&format!("abstract class {} implements {} {{\n", case_class_name, class.name));
        if case.fields.is_empty() {
            code.push_str(&format!("  const factory {}() = {};\n\n", case_class_name, impl_class_name));
        } else {
            // Check if this is a named parameter case or regular parameter case
            let is_named_params = case.fields.iter().any(|f| f.is_named);
            if is_named_params {
                code.push_str(&format!("  const factory {}({{\n", case_class_name));
                for field in &case.fields {
                    if field.has_default || field.ty.ends_with('?') {
                        code.push_str(&format!("    this.{},\n", field.name));
                    } else {
                        code.push_str(&format!("    required this.{},\n", field.name));
                    }
                }
                code.push_str(&format!("  }}) = {};\n\n", impl_class_name));
            } else {
                // Regular parameters (not named) - but we need to handle them as named parameters for consistency
                code.push_str(&format!("  const factory {}({{\n", case_class_name));
                for field in &case.fields {
                    code.push_str(&format!("    required {} {},\n", field.ty, field.name));
                }
                code.push_str(&format!("  }}) = {};\n\n", impl_class_name));
            }
        }
        
        // Generate fields
        for field in &case.fields {
            code.push_str(&format!("  {} get {};\n", field.ty, field.name));
        }
        if !case.fields.is_empty() {
            code.push_str("\n");
        }
        
        // Add copyWith getter for union cases
        code.push_str("  /// Create a copy of ");
        code.push_str(&class.name);
        code.push_str("\n");
        code.push_str("  /// with the given fields replaced by the non-null parameter values.\n");
        code.push_str("  @JsonKey(includeFromJson: false, includeToJson: false)\n");
        code.push_str(&format!("  _$${}ImplCopyWith<{}> get copyWith =>\n", case_class_name, impl_class_name));
        code.push_str(&format!("      throw _privateConstructorUsedError;\n"));
        code.push_str("}\n\n");
        
        // Generate implementation class
        code.push_str("/// @nodoc\n");
        code.push_str("@JsonSerializable()\n");
        code.push_str(&format!("class {} implements {} {{\n", impl_class_name, case_class_name));
        
        // Constructor
        if case.fields.is_empty() {
            code.push_str(&format!("  const {}();\n\n", impl_class_name));
        } else {
            code.push_str(&format!("  const {}({{\n", impl_class_name));
            for field in &case.fields {
                if field.has_default || field.ty.ends_with('?') {
                    code.push_str(&format!("    this.{},\n", field.name));
                } else {
                    code.push_str(&format!("    required this.{},\n", field.name));
                }
            }
            code.push_str("  });\n\n");
        }
        
        // Fields
        if !case.fields.is_empty() {
            for field in &case.fields {
                code.push_str(&format!("  @override\n"));
                code.push_str(&format!("  final {} {};\n", field.ty, field.name));
            }
            code.push_str("\n");
        }
        
        // $type field
        code.push_str(&format!("  @override\n"));
        code.push_str(&format!("  String get $type => '{}';\n\n", case.case_name));
        
        // toString method
        code.push_str("  @override\n");
        code.push_str("  String toString() {\n");
        if case.fields.is_empty() {
            code.push_str(&format!("    return '{}';\n", case.case_name));
        } else {
            let field_names: Vec<String> = case.fields.iter().map(|f| format!("{}: ${}", f.name, f.name)).collect();
            code.push_str(&format!("    return '{}.{}({})';\n", class.name, case.case_name, field_names.join(", ")));
        }
        code.push_str("  }\n\n");
        
        // equality operator
        code.push_str("  @override\n");
        code.push_str("  bool operator ==(Object other) {\n");
        code.push_str("    return identical(this, other) ||\n");
        code.push_str(&format!("        (other.runtimeType == runtimeType && other is {});\n", impl_class_name));
        code.push_str("  }\n\n");
        
        // hashCode
        code.push_str("  @JsonKey(includeFromJson: false, includeToJson: false)\n");
        code.push_str("  @override\n");
        code.push_str("  int get hashCode => runtimeType.hashCode;\n\n");
        
        // copyWith getter (override needed since abstract class has copyWith getter)
        code.push_str("  @JsonKey(includeFromJson: false, includeToJson: false)\n");
        code.push_str("  @override\n");
        code.push_str(&format!("  _$${}ImplCopyWith<{}> get copyWith =>\n", case_class_name, impl_class_name));
        code.push_str(&format!("      __$${}ImplCopyWithImpl<{}>(this, _$identity);\n\n", case_class_name, impl_class_name));
        
        // when method implementation
        code.push_str("  @override\n");
        code.push_str("  @optionalTypeArgs\n");
        code.push_str("  TResult when<TResult extends Object?>({\n");
        for other_case in union_cases {
            if other_case.case_name == case.case_name {
                if other_case.fields.is_empty() {
                    code.push_str(&format!("    required TResult Function() {},\n", other_case.case_name));
                } else {
                    let params: Vec<String> = other_case.fields.iter().map(|f| format!("{} {}", f.ty, f.name)).collect();
                    code.push_str(&format!("    required TResult Function({}) {},\n", params.join(", "), other_case.case_name));
                }
            } else {
                if other_case.fields.is_empty() {
                    code.push_str(&format!("    required TResult Function() {},\n", other_case.case_name));
                } else {
                    let params: Vec<String> = other_case.fields.iter().map(|f| format!("{} {}", f.ty, f.name)).collect();
                    code.push_str(&format!("    required TResult Function({}) {},\n", params.join(", "), other_case.case_name));
                }
            }
        }
        code.push_str("  }) {\n");
        if case.fields.is_empty() {
            code.push_str(&format!("    return {}();\n", case.case_name));
        } else {
            let field_names: Vec<String> = case.fields.iter().map(|f| f.name.clone()).collect();
            code.push_str(&format!("    return {}({});\n", case.case_name, field_names.join(", ")));
        }
        code.push_str("  }\n\n");
        
        // whenOrNull method implementation
        code.push_str("  @override\n");
        code.push_str("  @optionalTypeArgs\n");
        code.push_str("  TResult? whenOrNull<TResult extends Object?>({\n");
        for other_case in union_cases {
            if other_case.case_name == case.case_name {
                if other_case.fields.is_empty() {
                    code.push_str(&format!("    TResult? Function()? {},\n", other_case.case_name));
                } else {
                    let params: Vec<String> = other_case.fields.iter().map(|f| format!("{} {}", f.ty, f.name)).collect();
                    code.push_str(&format!("    TResult? Function({})? {},\n", params.join(", "), other_case.case_name));
                }
            } else {
                if other_case.fields.is_empty() {
                    code.push_str(&format!("    TResult? Function()? {},\n", other_case.case_name));
                } else {
                    let params: Vec<String> = other_case.fields.iter().map(|f| format!("{} {}", f.ty, f.name)).collect();
                    code.push_str(&format!("    TResult? Function({})? {},\n", params.join(", "), other_case.case_name));
                }
            }
        }
        code.push_str("  }) {\n");
        if case.fields.is_empty() {
            code.push_str(&format!("    return {}?.call();\n", case.case_name));
        } else {
            let field_names: Vec<String> = case.fields.iter().map(|f| f.name.clone()).collect();
            code.push_str(&format!("    return {}?.call({});\n", case.case_name, field_names.join(", ")));
        }
        code.push_str("  }\n\n");
        
        // maybeWhen method implementation
        code.push_str("  @override\n");
        code.push_str("  @optionalTypeArgs\n");
        code.push_str("  TResult maybeWhen<TResult extends Object?>({\n");
        for other_case in union_cases {
            if other_case.case_name == case.case_name {
                if other_case.fields.is_empty() {
                    code.push_str(&format!("    TResult Function()? {},\n", other_case.case_name));
                } else {
                    let params: Vec<String> = other_case.fields.iter().map(|f| format!("{} {}", f.ty, f.name)).collect();
                    code.push_str(&format!("    TResult Function({})? {},\n", params.join(", "), other_case.case_name));
                }
            } else {
                if other_case.fields.is_empty() {
                    code.push_str(&format!("    TResult Function()? {},\n", other_case.case_name));
                } else {
                    let params: Vec<String> = other_case.fields.iter().map(|f| format!("{} {}", f.ty, f.name)).collect();
                    code.push_str(&format!("    TResult Function({})? {},\n", params.join(", "), other_case.case_name));
                }
            }
        }
        code.push_str("    required TResult orElse(),\n");
        code.push_str("  }) {\n");
        code.push_str(&format!("    if ({} != null) {{\n", case.case_name));
        if case.fields.is_empty() {
            code.push_str(&format!("      return {}();\n", case.case_name));
        } else {
            let field_names: Vec<String> = case.fields.iter().map(|f| f.name.clone()).collect();
            code.push_str(&format!("      return {}({});\n", case.case_name, field_names.join(", ")));
        }
        code.push_str("    }\n");
        code.push_str("    return orElse();\n");
        code.push_str("  }\n\n");
        
        // map method implementation
        code.push_str("  @override\n");
        code.push_str("  @optionalTypeArgs\n");
        code.push_str("  TResult map<TResult extends Object?>({\n");
        for other_case in union_cases {
            let other_case_class_name = format!("{}{}", class.name, to_pascal_case(&other_case.case_name));
            if other_case.case_name == case.case_name {
                code.push_str(&format!("    required TResult Function({}) {},\n", impl_class_name, other_case.case_name));
            } else {
                code.push_str(&format!("    required TResult Function({}) {},\n", other_case_class_name, other_case.case_name));
            }
        }
        code.push_str("  }) {\n");
        code.push_str(&format!("    return {}(this);\n", case.case_name));
        code.push_str("  }\n\n");
        
        // mapOrNull method implementation
        code.push_str("  @override\n");
        code.push_str("  @optionalTypeArgs\n");
        code.push_str("  TResult? mapOrNull<TResult extends Object?>({\n");
        for other_case in union_cases {
            let other_case_class_name = format!("{}{}", class.name, to_pascal_case(&other_case.case_name));
            if other_case.case_name == case.case_name {
                code.push_str(&format!("    TResult? Function({})? {},\n", impl_class_name, other_case.case_name));
            } else {
                code.push_str(&format!("    TResult? Function({})? {},\n", other_case_class_name, other_case.case_name));
            }
        }
        code.push_str("  }) {\n");
        code.push_str(&format!("    return {}?.call(this);\n", case.case_name));
        code.push_str("  }\n\n");
        
        // maybeMap method implementation
        code.push_str("  @override\n");
        code.push_str("  @optionalTypeArgs\n");
        code.push_str("  TResult maybeMap<TResult extends Object?>({\n");
        for other_case in union_cases {
            let other_case_class_name = format!("{}{}", class.name, to_pascal_case(&other_case.case_name));
            if other_case.case_name == case.case_name {
                code.push_str(&format!("    TResult Function({})? {},\n", impl_class_name, other_case.case_name));
            } else {
                code.push_str(&format!("    TResult Function({})? {},\n", other_case_class_name, other_case.case_name));
            }
        }
        code.push_str("    required TResult orElse(),\n");
        code.push_str("  }) {\n");
        code.push_str(&format!("    if ({} != null) {{\n", case.case_name));
        code.push_str(&format!("      return {}(this);\n", case.case_name));
        code.push_str("    }\n");
        code.push_str("    return orElse();\n");
        code.push_str("  }\n\n");
        
        // toJson method for union cases
        code.push_str("  @override\n");
        code.push_str("  Map<String, dynamic> toJson() {\n");
        code.push_str(&format!("    return <String, dynamic>{{\n"));
        code.push_str(&format!("      'type': '{}',\n", case.case_name));
        for field in &case.fields {
            code.push_str(&format!("      '{}': {},\n", field.name, field.name));
        }
        code.push_str("    };\n");
        code.push_str("  }\n");
        code.push_str("}\n\n");
        
        // Generate copyWith classes
        code.push_str(&format!("/// @nodoc\n"));
        code.push_str(&format!("abstract class _$${}ImplCopyWith<$Res> {{\n", case_class_name));
        code.push_str(&format!("  factory _$${}ImplCopyWith({} value, $Res Function({}) then) =\n", case_class_name, impl_class_name, impl_class_name));
        code.push_str(&format!("      __$${}ImplCopyWithImpl<$Res>;\n\n", case_class_name));
        
        if case.fields.is_empty() {
            code.push_str("  $Res call();\n");
        } else {
            code.push_str("  $Res call({\n");
            for field in &case.fields {
                code.push_str(&format!("    Object? {} = freezed,\n", field.name));
            }
            code.push_str("  });\n");
        }
        code.push_str("}\n\n");
        
        code.push_str(&format!("/// @nodoc\n"));
        code.push_str(&format!("class __$${}ImplCopyWithImpl<$Res> implements _$${}ImplCopyWith<$Res> {{\n", case_class_name, case_class_name));
        code.push_str(&format!("  __$${}ImplCopyWithImpl(this._value, this._then);\n\n", case_class_name));
        code.push_str(&format!("  final {} _value;\n", impl_class_name));
        code.push_str(&format!("  final $Res Function({}) _then;\n\n", impl_class_name));
        
        code.push_str("  @pragma('vm:prefer-inline')\n");
        code.push_str("  @override\n");
        if case.fields.is_empty() {
            code.push_str("  $Res call() {\n");
            code.push_str("    // _value is used to satisfy the unused_field warning\n");
            code.push_str("    _value;\n");
            code.push_str(&format!("    return _then({}());\n", impl_class_name));
        } else {
            code.push_str("  $Res call({\n");
            for field in &case.fields {
                code.push_str(&format!("    Object? {} = freezed,\n", field.name));
            }
            code.push_str("  }) {\n");
            code.push_str(&format!("    return _then({}(\n", impl_class_name));
            for field in &case.fields {
                code.push_str(&format!("      {}: {} == freezed\n", field.name, field.name));
                code.push_str(&format!("          ? _value.{}\n", field.name));
                code.push_str(&format!("          : {} as {},\n", field.name, field.ty));
            }
            code.push_str("    ));\n");
        }
        code.push_str("  }\n");
        code.push_str("}\n\n");
    }
}

fn to_pascal_case(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    let mut result = String::new();
    let mut capitalize_next = true;
    
    for c in s.chars() {
        if c == '_' || c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    
    result
}



// FromJson/ToJsonの本体・閉じカッコも2スペースに統一
pub fn generate_json_code(class: &DartClass) -> String {
    let mut code = String::new();
    let source_content = std::fs::read_to_string(&class.file_path).unwrap_or_default();
    let fields = extract_fields_from_dart_class(&source_content, &class.name);
    let union_cases = extract_union_cases_from_dart_class(&source_content, &class.name);
    if !union_cases.is_empty() {
        // Generate union type FromJson function
        let from_json_fn = format!("_${}FromJson", class.name);
        code.push_str(&format!("{} {}(\n", class.name, from_json_fn));
        code.push_str("  Map<String, dynamic> json,\n");
        code.push_str(") {\n");
        code.push_str("  switch (json['type'] as String) {\n");
        
        for case in &union_cases {
            code.push_str(&format!("    case '{}':\n", case.case_name));
            if case.fields.is_empty() {
                code.push_str(&format!("      return {}.{}();\n", class.name, case.case_name));
            } else {
                code.push_str(&format!("      return {}.{}(\n", class.name, case.case_name));
                for field in &case.fields {
                    let field_conversion = get_field_conversion(field);
                    let formatted_conversion = format_long_expression(&field_conversion);
                    code.push_str(&format!("        {}: {},\n", field.name, formatted_conversion));
                }
                code.push_str("      );\n");
            }
        }
        code.push_str("    default:\n");
        code.push_str("      throw ArgumentError('Unknown type: ' + json['type'].toString());\n");
        code.push_str("  }\n");
        code.push_str("}\n\n");
    } else {
        let impl_class = format!("_$${}ImplImpl", class.name);
        let from_json_fn = format!("_$${}ImplImplFromJson", class.name);
        let to_json_fn = format!("_$${}ImplImplToJson", class.name);
        
        // FromJson - JsonSerializableGenerator style
        code.push_str(&format!("{} {}(\n", impl_class, from_json_fn));
        code.push_str("  Map<String, dynamic> json,\n");
        code.push_str(&format!(") => {}(\n", impl_class));
        for field in &fields {
            let field_conversion = get_field_conversion(field);
            let formatted_conversion = format_long_expression(&field_conversion);
            code.push_str(&format!("  {}: {},\n", field.name, formatted_conversion));
        }
        code.push_str(");\n\n");
        
        // ToJson - JsonSerializableGenerator style
        code.push_str(&format!("Map<String, dynamic> {}(\n", to_json_fn));
        code.push_str(&format!("  {} instance,\n", impl_class));
        code.push_str(") => <String, dynamic>{\n");
        for field in &fields {
            let field_conversion = get_to_json_field_conversion(field);
            code.push_str(&format!("  '{}': {},\n", field.name, field_conversion));
        }
        code.push_str("};\n\n");
    }
    code
}

fn get_to_json_field_conversion(field: &DartField) -> String {
    let field_name = &field.name;
    let field_type = &field.ty;
    
    match field_type.as_str() {
        "DateTime" => format!("instance.{}.toIso8601String()", field_name),
        "DateTime?" => format!("instance.{}?.toIso8601String()", field_name),
        _ => format!("instance.{}", field_name),
    }
}

fn format_long_expression(expr: &str) -> String {
    // Always format nullable DateTime expressions
    if expr.contains("json['") && expr.contains("] == null ? null : DateTime.parse(") {
        let field_name = expr.split("json['").nth(1).unwrap().split("']").next().unwrap();
        return format!("json['{}'] == null\n          ? null\n          : DateTime.parse(json['{}'] as String)", field_name, field_name);
    }
    
    if expr.len() > 80 {
        // Split long expressions like ternary operators
        if expr.contains("? null :") {
            let parts: Vec<&str> = expr.split("? null :").collect();
            if parts.len() == 2 {
                return format!("{}\n          ? null\n          : {}", parts[0], parts[1]);
            }
        }
        // Split long map expressions
        if expr.contains("?.map(") && expr.contains(").toList()") {
            let map_start = expr.find("?.map(").unwrap();
            let map_end = expr.find(").toList()").unwrap() + 9;
            let before_map = &expr[..map_start];
            let map_content = &expr[map_start..map_end];
            let after_map = &expr[map_end..];
            
            return format!("{}\n          {}{}", before_map, map_content, after_map);
        }
    }
    expr.to_string()
}

fn get_field_conversion(field: &DartField) -> String {
    let field_name = &field.name;
    let field_type = &field.ty;
    match field_type.as_str() {
        "DateTime" => format!("DateTime.parse(json['{}'] as String)", field_name),
        "DateTime?" => format!("json['{}'] == null\n          ? null\n          : DateTime.parse(json['{}'] as String)", field_name, field_name),
        "int" => format!("(json['{}'] as num).toInt()", field_name),
        "int?" => format!("(json['{}'] as num?)?.toInt()", field_name),
        "List<String>" => {
            if field.has_default && field.default_value.as_deref() == Some("[]") {
                format!("(json['{}'] as List<dynamic>?)\n          ?.map((e) => e as String)\n          .toList() ??\n      const []", field_name)
            } else {
                format!("(json['{}'] as List<dynamic>?)?.map((e) => e as String).toList()", field_name)
            }
        }
        "List<String>?" => format!("(json['{}'] as List<dynamic>?)?.map((e) => e as String).toList()", field_name),
        "String" => {
            if field.has_default {
                let default_value = field.default_value.as_deref().unwrap_or("''");
                let clean_default = default_value.trim_matches('\'');
                format!("json['{}'] as String? ?? '{}'", field_name, clean_default)
            } else {
                format!("json['{}'] as String", field_name)
            }
        }
        "bool" => {
            if field.has_default {
                let default_value = field.default_value.as_deref().unwrap_or("false");
                let clean_default = default_value.trim_matches('\'');
                format!("json['{}'] as bool? ?? {}", field_name, clean_default)
            } else {
                format!("json['{}'] as bool", field_name)
            }
        }
        _ => {
            if field_type.ends_with('?') {
                format!("json['{}'] as {}", field_name, field_type)
            } else {
                format!("json['{}'] as {}", field_name, field_type)
            }
        }
    }
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
                        eprintln!("[DEBUG] Added field: {} {} (has_default: {})", field_clone.ty, field_clone.name, field_clone.has_default);
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
    let mut is_named = false;
    let mut has_default = false;
    let mut default_value = None;
    let mut param = param.to_string();
    // Remove @Default annotation
    if let Some(default_start) = param.find("@Default(") {
        if let Some(default_end) = param[default_start..].find(')') {
            let default_val = &param[default_start + 9..default_start + default_end];
            has_default = true;
            default_value = Some(default_val.trim().to_string());
            eprintln!("[DEBUG] Found @Default annotation: {} = {}", param, default_val);
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
            // Check if this is a named parameter (contains ':' or is in a named parameter context)
            is_named = param.contains(':') || param.contains('{') || param.contains('}');
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
                                // Handle both named parameters (wrapped in {}) and regular parameters
                                let params_content = params_content.trim();
                                if params_content.starts_with('{') && params_content.ends_with('}') {
                                    // Named parameters
                                    let inner_content = &params_content[1..params_content.len()-1];
                                    let mut params = Vec::new();
                                    let mut current_param = String::new();
                                    let mut brace_count = 0;
                                    let mut paren_count = 0;
                                    for ch in inner_content.chars() {
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
                                } else {
                                    // Regular parameters (not wrapped in {})
                                    let mut params = Vec::new();
                                    let mut current_param = String::new();
                                    let mut paren_count = 0;
                                    for ch in params_content.chars() {
                                        match ch {
                                            '(' => { paren_count += 1; current_param.push(ch); }
                                            ')' => { paren_count -= 1; current_param.push(ch); }
                                            ',' => {
                                                if paren_count == 0 {
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
    // Always output to the same directory as the source file
    let mut freezed_output_path = file_path.parent().unwrap().to_path_buf();
    freezed_output_path.push(format!("{}.freezed.dart", base_name));
    let mut g_dart_output_path = file_path.parent().unwrap().to_path_buf();
    g_dart_output_path.push(format!("{}.g.dart", base_name));
    (freezed_output_path, g_dart_output_path)
}

 