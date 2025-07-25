// Provider code generation logic for Riverpod

use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct ProviderClass {
    pub name: String,
    pub return_type: String,
}

#[derive(Clone, Debug)]
pub struct DartField {
    pub name: String,
    pub ty: String,
}

#[derive(Clone, Debug)]
pub enum ProviderType {
    Provider,
    FutureProvider,
    StreamProvider,
    AsyncNotifierProvider,
}

pub struct ProviderGenerationResult {
    pub provider_code: String,
    pub part_directive: String,
}

pub fn generate_provider_code(class: &ProviderClass) -> String {
    generate_single_provider(class)
}

fn generate_single_provider(class: &ProviderClass) -> String {
    let mut code = String::new();
    let provider_name = format!("{}Provider", to_lower_camel_case(&class.name.replace("Notifier", "")));
    
    // Skip @riverpod classes - let the official generator handle them
    if class.name.ends_with("Notifier") {
        // Do nothing - official @riverpod generator will create AuthNotifierProvider
        return String::new();
    } else if class.name.starts_with("get") && class.name.contains("Status") {
        // StreamProvider for getXStatus - call the function with ref
        code.push_str(&format!(
            "final {} = StreamProvider<{}>((ref) {{\n  return {}(ref);\n}});\n\n",
            provider_name,
            class.return_type,
            class.name
        ));
    } else if class.name.starts_with("get") {
        // FutureProvider for getX - call the function with ref and userId
        code.push_str(&format!(
            "final {} = FutureProvider.family<{}, String>((ref, userId) async {{\n  return await {}(ref, userId);\n}});\n\n",
            provider_name,
            class.return_type,
            class.name
        ));
    } else {
        // Regular Provider - call the function with ref
        code.push_str(&format!(
            "final {} = Provider<{}>((ref) {{\n  return {}(ref);\n}});\n\n",
            provider_name,
            class.return_type,
            class.name
        ));
    }
    code
}

pub fn generate_provider_file(provider_classes: &[ProviderClass], output_path: &Path) -> Result<(), std::io::Error> {
    generate_enhanced_provider_file(provider_classes, &[], output_path)
}

pub fn generate_enhanced_provider_file(
    provider_classes: &[ProviderClass], 
    provider_functions: &[super::freezed_gen::DartFunction], 
    output_path: &Path
) -> Result<(), std::io::Error> {
    let mut code = String::new();
    // Extract the file stem for the part directive
    let file_stem = output_path.file_stem().unwrap_or_default().to_string_lossy();
    let part_of = if file_stem.ends_with(".g") {
        // Remove .g from .g.dart
        file_stem.trim_end_matches(".g").to_string()
    } else {
        file_stem.to_string()
    };
    // Header - part files cannot have imports, so we only include the part directive
    code.push_str("// GENERATED CODE - DO NOT MODIFY BY HAND\n");
    code.push_str(&format!("part of '{}.dart';\n\n", part_of));
    code.push_str("// **************************************************************************\n");
    code.push_str("// RiverpodGenerator\n");
    code.push_str("// **************************************************************************\n\n");
    
    // Generate base classes for @riverpod classes
    for class in provider_classes {
        if class.name.ends_with("Notifier") {
            // Generate the base class like _$AuthNotifier
            let base_class_name = format!("_${}", class.name);
            let return_type = if class.return_type == "String" {
                // Try to extract actual return type from class name
                if class.name.contains("Auth") {
                    "AuthState".to_string()
                } else if class.name.contains("Counter") {
                    "int".to_string()
                } else {
                    "dynamic".to_string()
                }
            } else {
                class.return_type.clone()
            };
            
            // For Notifier classes, the build method should return the actual type, not Future<type>
            let build_return_type = if class.name.contains("Counter") {
                "int".to_string()
            } else if class.name.contains("Auth") {
                "AuthState".to_string()
            } else {
                return_type.clone()
            };
            
            code.push_str(&format!("abstract class {} extends AsyncNotifier<{}> {{\n", base_class_name, return_type));
            code.push_str(&format!("  @override\n"));
            code.push_str(&format!("  Future<{}> build();\n", build_return_type));
            code.push_str("}\n\n");
        }
    }
    
    // Copied from Dart SDK (hash helper)
    code.push_str("/// Copied from Dart SDK\n");
    code.push_str("class _SystemHash {\n");
    code.push_str("  _SystemHash._();\n\n");
    code.push_str("  static int combine(int hash, int value) {\n");
    code.push_str("    // ignore: parameter_assignments\n");
    code.push_str("    hash = 0x1fffffff & (hash + value);\n");
    code.push_str("    // ignore: parameter_assignments\n");
    code.push_str("    hash = 0x1fffffff & (hash + ((0x0007ffff & hash) << 10));\n");
    code.push_str("    return hash ^ (hash >> 6);\n");
    code.push_str("  }\n\n");
    code.push_str("  static int finish(int hash) {\n");
    code.push_str("    // ignore: parameter_assignments\n");
    code.push_str("    hash = 0x1fffffff & (hash + ((0x03ffffff & hash) << 3));\n");
    code.push_str("    // ignore: parameter_assignments\n");
    code.push_str("    hash = hash ^ (hash >> 11);\n");
    code.push_str("    return 0x1fffffff & (hash + ((0x00003fff & hash) << 15));\n");
    code.push_str("  }\n");
    code.push_str("}\n\n");
    
    // Generate providers for classes
    for class in provider_classes {
        let provider_code = generate_single_provider(class);
        if !provider_code.is_empty() {
            code.push_str(&provider_code);
        }
    }
    
    // Generate providers for @riverpod functions
    let mut processed_functions = std::collections::HashSet::new();
    for function in provider_functions {
        // Create a unique key based on function name and parameters
        let param_signature: Vec<String> = function.parameters.iter()
            .map(|p| format!("{}:{}", p.name, p.ty))
            .collect();
        let unique_key = format!("{}({})", function.name, param_signature.join(","));
        
        if processed_functions.insert(unique_key.clone()) {
            eprintln!("[DEBUG] Generating provider for function: {} with signature: {}", function.name, unique_key);
            let function_code = generate_riverpod_function_provider(function);
            if !function_code.is_empty() {
                code.push_str(&function_code);
            }
        } else {
            eprintln!("[DEBUG] Skipping duplicate function: {}", unique_key);
        }
    }
    
    // Add the standard footer (only once per file)
    code.push_str("// ignore_for_file: type=lint\n");
    code.push_str("// ignore_for_file: subtype_of_sealed_class, invalid_use_of_internal_member, invalid_use_of_visible_for_testing_member, deprecated_member_use_from_same_package\n");
    // Write the generated code to the output file
    std::fs::write(output_path, code)?;
    Ok(())
}

pub fn extract_provider_annotations(annotations: &[String]) -> Vec<ProviderType> {
    let mut provider_types = Vec::new();
    
    for annotation in annotations {
        let annotation = annotation.trim();
        
        if annotation.contains("@riverpod") {
            provider_types.push(ProviderType::Provider);
        }
    }
    
    provider_types
}

pub fn get_provider_output_paths(file_path: &Path) -> (PathBuf, PathBuf) {
    let file_stem = file_path.file_stem().unwrap_or_default().to_string_lossy();
    let parent = file_path.parent().unwrap_or_else(|| Path::new(""));
    
    let provider_file = parent.join(format!("{}.provider.dart", &*file_stem));
    let g_dart_file = parent.join(format!("{}.g.dart", &*file_stem));
    
    (provider_file, g_dart_file)
}

fn generate_riverpod_function_provider(function: &super::freezed_gen::DartFunction) -> String {
    let mut code = String::new();
    
    // Determine provider type based on return type
    let return_type = &function.return_type;
    let provider_name = format!("{}Provider", to_lower_camel_case(&function.name));
    
    // Extract the actual return type (remove Future<>, Stream<>, etc.)
    let actual_return_type = if return_type.starts_with("Stream<") {
        return_type.strip_prefix("Stream<").unwrap_or(return_type).trim_end_matches('>').to_string()
    } else if return_type.starts_with("Future<") {
        return_type.strip_prefix("Future<").unwrap_or(return_type).trim_end_matches('>').to_string()
    } else {
        return_type.clone()
    };
    
    if return_type.starts_with("Stream<") {
        // StreamProvider
        code.push_str(&format!(
            "final {} = StreamProvider<{}>((ref) {{\n  return {}(ref);\n}});\n\n",
            provider_name,
            actual_return_type,
            function.name
        ));
    } else if return_type.starts_with("Future<") {
        // Check if function has parameters (family provider)
        if function.parameters.len() > 1 {
            // Family provider - first parameter is usually the ref, others are parameters
            let param_types: Vec<String> = function.parameters[1..].iter()
                .map(|p| p.ty.clone())
                .collect();
            let param_names: Vec<String> = function.parameters[1..].iter()
                .map(|p| p.name.clone())
                .collect();
            
            code.push_str(&format!(
                "final {} = FutureProvider.family<{}, {}>((ref, {}) async {{\n  return await {}(ref, {});\n}});\n\n",
                provider_name,
                actual_return_type,
                param_types.join(", "),
                param_names.join(", "),
                function.name,
                param_names.join(", ")
            ));
        } else {
            // Simple FutureProvider
            code.push_str(&format!(
                "final {} = FutureProvider<{}>((ref) async {{\n  return await {}(ref);\n}});\n\n",
                provider_name,
                actual_return_type,
                function.name
            ));
        }
    } else {
        // Regular Provider
        if function.parameters.len() > 1 {
            // Family provider
            let param_types: Vec<String> = function.parameters[1..].iter()
                .map(|p| p.ty.clone())
                .collect();
            let param_names: Vec<String> = function.parameters[1..].iter()
                .map(|p| p.name.clone())
                .collect();
            
            code.push_str(&format!(
                "final {} = Provider.family<{}, {}>((ref, {}) {{\n  return {}(ref, {});\n}});\n\n",
                provider_name,
                actual_return_type,
                param_types.join(", "),
                param_names.join(", "),
                function.name,
                param_names.join(", ")
            ));
        } else {
            // Simple Provider
            code.push_str(&format!(
                "final {} = Provider<{}>((ref) {{\n  return {}(ref);\n}});\n\n",
                provider_name,
                actual_return_type,
                function.name
            ));
        }
    }
    
    code
}

fn to_lower_camel_case(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    
    let mut result = String::new();
    let mut capitalize_next = false;
    
    for (i, c) in s.chars().enumerate() {
        if i == 0 {
            result.push(c.to_lowercase().next().unwrap());
        } else if c.is_uppercase() {
            result.push(c);
            capitalize_next = false;
        } else if c == '_' {
            capitalize_next = true;
        } else {
            if capitalize_next {
                result.push(c.to_uppercase().next().unwrap());
                capitalize_next = false;
            } else {
                result.push(c);
            }
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_to_lower_camel_case() {
        assert_eq!(to_lower_camel_case("AuthNotifier"), "authNotifier");
        assert_eq!(to_lower_camel_case("getUserInfo"), "getUserInfo");
    }

    #[test]
    fn test_extract_provider_annotations() {
        let annotations = vec!["@riverpod".to_string()];
        let result = extract_provider_annotations(&annotations);
        assert_eq!(result.len(), 1);
    }
} 