use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Deserialize, Serialize)]
struct PubspecYaml {
    name: String,
    #[serde(default)]
    flutter: FlutterSection,
}

#[derive(Debug, Deserialize, Serialize)]
struct FlutterSection {
    #[serde(default)]
    assets: Vec<String>,
}

impl Default for FlutterSection {
    fn default() -> Self {
        Self {
            assets: Vec::new(),
        }
    }
}

pub fn generate_assets_from_path(project_path: &str) {
    println!("Generating assets from {}", project_path);
    
    // Load pubspec.yaml
    let pubspec_path = format!("{}/pubspec.yaml", project_path);
    let pubspec_content = match fs::read_to_string(&pubspec_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading {}: {}", pubspec_path, e);
            return;
        }
    };
    
    // Parse YAML
    let pubspec: PubspecYaml = match serde_yaml::from_str(&pubspec_content) {
        Ok(pubspec) => pubspec,
        Err(e) => {
            eprintln!("Error parsing pubspec.yaml: {}", e);
            return;
        }
    };
    
    // Collect asset files
    let asset_files = collect_asset_files_from_project(&pubspec.flutter.assets, project_path);
    
    // Generate Dart class
    let dart_code = generate_dart_assets_class(&asset_files);
    
    // Create output directory
    let output_dir = format!("{}/lib/gen", project_path);
    let output_path = Path::new(&output_dir);
    if let Err(e) = fs::create_dir_all(output_path) {
        eprintln!("Error creating output directory: {}", e);
        return;
    }
    
    // Write to file
    let output_file_path = format!("{}/assets.gen.dart", output_dir).replace("//", "/");
    if let Err(e) = fs::write(&output_file_path, dart_code) {
        eprintln!("Error writing assets.gen.dart: {}", e);
        return;
    }
    
    println!("Generated assets.gen.dart with {} asset constants", asset_files.len());
}

// FlutterGen-like behavior: explore based on pubspec.yaml assets configuration
pub fn generate_assets_with_paths(_assets_path: &str, output_path: &str) {
    println!("Generating assets from pubspec.yaml to {}", output_path);
    
    // Load pubspec.yaml from current directory
    let pubspec_content = match fs::read_to_string("pubspec.yaml") {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading pubspec.yaml: {}", e);
            return;
        }
    };
    
    // Parse YAML
    let pubspec: PubspecYaml = match serde_yaml::from_str(&pubspec_content) {
        Ok(pubspec) => pubspec,
        Err(e) => {
            eprintln!("Error parsing pubspec.yaml: {}", e);
            return;
        }
    };
    
    // Collect asset files using pubspec.yaml assets configuration
    // Use the current directory as project root
    let project_root = ".";
    let asset_files = collect_asset_files_from_project(&pubspec.flutter.assets, project_root);
    
    // Generate Dart class
    let dart_code = generate_dart_assets_class(&asset_files);
    
    // Create output directory
    let output_path_buf = Path::new(output_path);
    if let Err(e) = fs::create_dir_all(output_path_buf) {
        eprintln!("Error creating output directory: {}", e);
        return;
    }
    
    // Write to file
    let output_file_path = format!("{}/assets.gen.dart", output_path).replace("//", "/");
    if let Err(e) = fs::write(&output_file_path, dart_code) {
        eprintln!("Error writing assets.gen.dart: {}", e);
        return;
    }
    
    println!("Generated assets.gen.dart with {} asset constants", asset_files.len());
}

fn collect_asset_files_from_project(asset_paths: &[String], project_path: &str) -> Vec<String> {
    let mut asset_files = Vec::new();
    
    for path in asset_paths {
        // Only process paths that start with "assets/"
        if !path.starts_with("assets/") {
            continue;
        }
        
        let full_path = format!("{}/{}", project_path, path);
        let path_buf = PathBuf::from(&full_path);
        
        if path_buf.is_file() {
            // Single file case
            asset_files.push(path.to_string());
        } else if path_buf.is_dir() {
            // Directory case, recursively search
            for entry in WalkDir::new(&path_buf).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    if let Some(relative_path) = entry.path().strip_prefix(&path_buf).ok() {
                        let asset_path = format!("{}/{}", path, relative_path.to_string_lossy());
                        asset_files.push(asset_path);
                    }
                }
            }
        }
    }
    
    asset_files.sort();
    asset_files
}

// New function: configurable paths
fn collect_asset_files_from_paths(asset_paths: &[String], assets_base_path: &str) -> Vec<String> {
    let mut asset_files = Vec::new();
    
    println!("Debug: Processing {} asset paths", asset_paths.len());
    
    for path in asset_paths {
        println!("Debug: Checking path: {}", path);
        // Only process paths that start with "assets/"
        if !path.starts_with("assets/") {
            println!("Debug: Skipping non-asset path: {}", path);
            continue;
        }
        // If path starts with assets/, do not add assets_base_path
        let full_path = if path.starts_with("assets/") {
            path.to_string()
        } else {
            format!("{}/{}", assets_base_path, path)
        };
        let path_buf = PathBuf::from(&full_path);
        println!("Debug: Full path: {}", full_path);
        println!("Debug: Path exists: {}", path_buf.exists());
        println!("Debug: Is file: {}", path_buf.is_file());
        println!("Debug: Is dir: {}", path_buf.is_dir());
        if path_buf.is_file() {
            println!("Debug: Adding file: {}", path);
            asset_files.push(path.to_string());
        } else if path_buf.is_dir() {
            println!("Debug: Searching directory: {}", path);
            for entry in WalkDir::new(&path_buf).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    if let Some(relative_path) = entry.path().strip_prefix(&path_buf).ok() {
                        let asset_path = format!("{}/{}", path, relative_path.to_string_lossy());
                        println!("Debug: Found file in dir: {}", asset_path);
                        asset_files.push(asset_path);
                    }
                }
            }
        }
    }
    println!("Debug: Total asset files found: {}", asset_files.len());
    asset_files.sort();
    asset_files
}

fn generate_dart_assets_class(asset_files: &[String]) -> String {
    let mut dart_code = String::new();
    
    // Header - match flutter_gen exactly
    dart_code.push_str("// dart format width=80\n\n");
    dart_code.push_str("/// GENERATED CODE - DO NOT MODIFY BY HAND\n");
    dart_code.push_str("/// *****************************************************\n");
    dart_code.push_str("///  FlutterGen\n");
    dart_code.push_str("/// *****************************************************\n\n");
    dart_code.push_str("// coverage:ignore-file\n");
    dart_code.push_str("// ignore_for_file: type=lint\n");
    dart_code.push_str("// ignore_for_file: deprecated_member_use,directives_ordering,implicit_dynamic_list_literal,unnecessary_import\n\n");
    dart_code.push_str("import 'package:flutter/widgets.dart';\n\n");
    
    // Group assets by category
    let mut categorized_assets: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    
    for asset_file in asset_files {
        let normalized = asset_file.replace("//", "/");
        if let Some(category) = get_asset_category(&normalized) {
            categorized_assets.entry(category.to_string()).or_insert_with(Vec::new).push(normalized);
        }
    }
    
    // Generate category classes
    for (category, files) in &categorized_assets {
        let class_name = format!("$Assets{}Gen", capitalize_first(category));
        dart_code.push_str(&format!("class {} {{\n", class_name));
        dart_code.push_str(&format!("  const {}();\n\n", class_name));
        
        for file in files {
            let constant_name = asset_file_to_constant_name_camel_case(file);
            let asset_type = get_asset_type(file);
            
            match asset_type {
                "image" => {
                    dart_code.push_str(&format!("  /// File path: {}\n", file));
                    dart_code.push_str(&format!("  AssetGenImage get {} => const AssetGenImage('{}');\n", constant_name, file));
                },
                "font" => {
                    dart_code.push_str(&format!("  /// File path: {}\n", file));
                    dart_code.push_str(&format!("  String get {} => '{}';\n", constant_name, file));
                },
                _ => {
                    dart_code.push_str(&format!("  /// File path: {}\n", file));
                    dart_code.push_str(&format!("  String get {} => '{}';\n", constant_name, file));
                }
            }
        }
        
        // Add values list
        dart_code.push_str("\n  /// List of all assets\n");
        let asset_type = get_asset_type(&files[0]);
        match asset_type {
            "image" => {
                dart_code.push_str(&format!("  List<AssetGenImage> get values => [{}];\n", 
                    files.iter().map(|f| asset_file_to_constant_name_camel_case(f)).collect::<Vec<_>>().join(", ")));
            },
            _ => {
                dart_code.push_str(&format!("  List<String> get values => [{}];\n", 
                    files.iter().map(|f| asset_file_to_constant_name_camel_case(f)).collect::<Vec<_>>().join(", ")));
            }
        }
        
        dart_code.push_str("}\n\n");
    }
    
    // Generate main Assets class
    dart_code.push_str("class Assets {\n");
    dart_code.push_str("  const Assets._();\n\n");
    
    for category in categorized_assets.keys() {
        let class_name = format!("$Assets{}Gen", capitalize_first(category));
        dart_code.push_str(&format!("  static const {} {} = {}();\n", class_name, category, class_name));
    }
    
    dart_code.push_str("}\n\n");
    
    // Generate AssetGenImage class - match flutter_gen exactly
    dart_code.push_str("class AssetGenImage {\n");
    dart_code.push_str("  const AssetGenImage(\n");
    dart_code.push_str("    this._assetName, {\n");
    dart_code.push_str("    this.size,\n");
    dart_code.push_str("    this.flavors = const {},\n");
    dart_code.push_str("    this.animation,\n");
    dart_code.push_str("  });\n\n");
    dart_code.push_str("  final String _assetName;\n\n");
    dart_code.push_str("  final Size? size;\n");
    dart_code.push_str("  final Set<String> flavors;\n");
    dart_code.push_str("  final AssetGenImageAnimation? animation;\n\n");
    dart_code.push_str("  Image image({\n");
    dart_code.push_str("    Key? key,\n");
    dart_code.push_str("    AssetBundle? bundle,\n");
    dart_code.push_str("    ImageFrameBuilder? frameBuilder,\n");
    dart_code.push_str("    ImageErrorWidgetBuilder? errorBuilder,\n");
    dart_code.push_str("    String? semanticLabel,\n");
    dart_code.push_str("    bool excludeFromSemantics = false,\n");
    dart_code.push_str("    double? scale,\n");
    dart_code.push_str("    double? width,\n");
    dart_code.push_str("    double? height,\n");
    dart_code.push_str("    Color? color,\n");
    dart_code.push_str("    Animation<double>? opacity,\n");
    dart_code.push_str("    BlendMode? colorBlendMode,\n");
    dart_code.push_str("    BoxFit? fit,\n");
    dart_code.push_str("    AlignmentGeometry alignment = Alignment.center,\n");
    dart_code.push_str("    ImageRepeat repeat = ImageRepeat.noRepeat,\n");
    dart_code.push_str("    Rect? centerSlice,\n");
    dart_code.push_str("    bool matchTextDirection = false,\n");
    dart_code.push_str("    bool gaplessPlayback = true,\n");
    dart_code.push_str("    bool isAntiAlias = false,\n");
    dart_code.push_str("    String? package,\n");
    dart_code.push_str("    FilterQuality filterQuality = FilterQuality.medium,\n");
    dart_code.push_str("    int? cacheWidth,\n");
    dart_code.push_str("    int? cacheHeight,\n");
    dart_code.push_str("  }) {\n");
    dart_code.push_str("    return Image.asset(\n");
    dart_code.push_str("      _assetName,\n");
    dart_code.push_str("      key: key,\n");
    dart_code.push_str("      bundle: bundle,\n");
    dart_code.push_str("      frameBuilder: frameBuilder,\n");
    dart_code.push_str("      errorBuilder: errorBuilder,\n");
    dart_code.push_str("      semanticLabel: semanticLabel,\n");
    dart_code.push_str("      excludeFromSemantics: excludeFromSemantics,\n");
    dart_code.push_str("      scale: scale,\n");
    dart_code.push_str("      width: width,\n");
    dart_code.push_str("      height: height,\n");
    dart_code.push_str("      color: color,\n");
    dart_code.push_str("      opacity: opacity,\n");
    dart_code.push_str("      colorBlendMode: colorBlendMode,\n");
    dart_code.push_str("      fit: fit,\n");
    dart_code.push_str("      alignment: alignment,\n");
    dart_code.push_str("      repeat: repeat,\n");
    dart_code.push_str("      centerSlice: centerSlice,\n");
    dart_code.push_str("      matchTextDirection: matchTextDirection,\n");
    dart_code.push_str("      gaplessPlayback: gaplessPlayback,\n");
    dart_code.push_str("      isAntiAlias: isAntiAlias,\n");
    dart_code.push_str("      package: package,\n");
    dart_code.push_str("      filterQuality: filterQuality,\n");
    dart_code.push_str("      cacheWidth: cacheWidth,\n");
    dart_code.push_str("      cacheHeight: cacheHeight,\n");
    dart_code.push_str("    );\n");
    dart_code.push_str("  }\n\n");
    
    // Add provider method
    dart_code.push_str("  ImageProvider provider({AssetBundle? bundle, String? package}) {\n");
    dart_code.push_str("    return AssetImage(_assetName, bundle: bundle, package: package);\n");
    dart_code.push_str("  }\n\n");
    
    // Add path getter
    dart_code.push_str("  String get path => _assetName;\n\n");
    
    // Add keyName getter
    dart_code.push_str("  String get keyName => _assetName;\n");
    dart_code.push_str("}\n\n");
    
    // Generate AssetGenImageAnimation class - match flutter_gen exactly
    dart_code.push_str("class AssetGenImageAnimation {\n");
    dart_code.push_str("  const AssetGenImageAnimation({\n");
    dart_code.push_str("    required this.isAnimation,\n");
    dart_code.push_str("    required this.duration,\n");
    dart_code.push_str("    required this.frames,\n");
    dart_code.push_str("  });\n\n");
    dart_code.push_str("  final bool isAnimation;\n");
    dart_code.push_str("  final Duration duration;\n");
    dart_code.push_str("  final int frames;\n");
    dart_code.push_str("}\n");
    
    dart_code
}

fn get_asset_category(asset_file: &str) -> Option<&str> {
    if asset_file.starts_with("assets/") {
        let parts: Vec<&str> = asset_file.split('/').collect();
        if parts.len() >= 3 {
            return Some(parts[1]); // Return the category (images, fonts, data, etc.)
        }
    }
    None
}

fn get_asset_type(asset_file: &str) -> &str {
    if let Some(extension) = asset_file.split('.').last() {
        match extension.to_lowercase().as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" => "image",
            "ttf" | "otf" | "woff" | "woff2" => "font",
            _ => "data"
        }
    } else {
        "data"
    }
}

fn capitalize_first(s: &str) -> String {
    if let Some(first_char) = s.chars().next() {
        let mut result = String::new();
        result.push(first_char.to_uppercase().next().unwrap());
        result.push_str(&s[1..]);
        result
    } else {
        s.to_string()
    }
}

fn asset_file_to_constant_name_camel_case(asset_file: &str) -> String {
    // Convert file path to camelCase constant name (flutter_gen style)
    // Example: "assets/images/logo.png" -> "logo"
    // Example: "assets/fonts/Roboto-Regular.ttf" -> "robotoRegular"
    
    // Remove "assets/" prefix if present
    let path_without_assets = if asset_file.starts_with("assets/") {
        &asset_file[7..] // Skip "assets/"
    } else {
        asset_file
    };
    
    let mut constant_name = String::new();
    
    for part in path_without_assets.split('/') {
        if !part.is_empty() {
            // Remove file extension
            let part_without_ext = if let Some(dot_pos) = part.rfind('.') {
                &part[..dot_pos]
            } else {
                part
            };
            
            // Convert to camelCase
            let mut chars = part_without_ext.chars();
            if let Some(first_char) = chars.next() {
                constant_name.push(first_char.to_lowercase().next().unwrap());
                constant_name.extend(chars.map(|c| {
                    if c == '-' || c == '_' {
                        ' ' // Replace with space to be removed
                    } else {
                        c
                    }
                }));
            }
        }
    }
    
    // Convert kebab-case to camelCase (e.g., "Roboto-Regular" -> "robotoRegular")
    let mut result = String::new();
    let mut capitalize_next = false;
    
    for ch in constant_name.chars() {
        if ch == ' ' {
            capitalize_next = true;
        } else {
            if capitalize_next {
                result.push(ch.to_uppercase().next().unwrap());
                capitalize_next = false;
            } else {
                result.push(ch);
            }
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_collect_asset_files_from_project() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Create asset directories
        let assets_dir = project_path.join("assets");
        let images_dir = assets_dir.join("images");
        fs::create_dir_all(&images_dir).unwrap();
        
        // Create test files
        fs::write(images_dir.join("logo.png"), "fake image").unwrap();
        fs::write(assets_dir.join("data.json"), "fake data").unwrap();
        
        let asset_paths = vec![
            "assets/images/".to_string(),
            "assets/data.json".to_string(),
        ];
        
        let asset_files = collect_asset_files_from_project(&asset_paths, project_path.to_str().unwrap());
        
        // Check that we have the expected files
        assert!(asset_files.len() >= 2);
        assert!(asset_files.contains(&"assets/images//logo.png".to_string()));
        assert!(asset_files.contains(&"assets/data.json".to_string()));
    }

    #[test]
    fn test_generate_dart_assets_class() {
        let asset_files = vec![
            "assets/images/logo.png".to_string(),
            "assets/data/sample.json".to_string(),
        ];
        
        let dart_code = generate_dart_assets_class(&asset_files);
        
        assert!(dart_code.contains("class Assets"));
        assert!(dart_code.contains("class $AssetsImagesGen"));
        assert!(dart_code.contains("class $AssetsDataGen"));
        assert!(dart_code.contains("AssetGenImage"));
        assert!(dart_code.contains("assets/images/logo.png"));
        assert!(dart_code.contains("assets/data/sample.json"));
    }

    #[test]
    fn test_asset_file_to_constant_name() {
        let test_cases = vec![
            ("assets/images/logo.png", "logo"),
            ("assets/data/sample.json", "sample"),
            ("assets/fonts/Roboto-Regular.ttf", "robotoRegular"),
        ];
        
        for (input, expected) in test_cases {
            let result = asset_file_to_constant_name_camel_case(input);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_parse_pubspec_yaml() {
        let yaml_content = r#"
name: test_app
flutter:
  assets:
    - assets/images/
    - assets/data/sample.json
"#;
        
        let pubspec: PubspecYaml = serde_yaml::from_str(yaml_content).unwrap();
        
        assert_eq!(pubspec.name, "test_app");
        assert_eq!(pubspec.flutter.assets.len(), 2);
        assert!(pubspec.flutter.assets.contains(&"assets/images/".to_string()));
        assert!(pubspec.flutter.assets.contains(&"assets/data/sample.json".to_string()));
    }

    #[test]
    fn test_parse_pubspec_yaml_without_assets() {
        let yaml_content = r#"
name: test_app
flutter:
"#;
        
        let pubspec: PubspecYaml = serde_yaml::from_str(yaml_content).unwrap();
        
        assert_eq!(pubspec.name, "test_app");
        assert_eq!(pubspec.flutter.assets.len(), 0);
    }
} 