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
    // Use the Flutter project root directory
    let project_root = "test_flutter_app/aminomi";
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
    
    // Class header
    dart_code.push_str("// This file is auto-generated. Do not edit.\n");
    dart_code.push_str("// Generated by SuperFastGen\n\n");
    dart_code.push_str("class Assets {\n");
    
    // Generate asset constants
    for asset_file in asset_files {
        // Normalize duplicate slashes
        let normalized = asset_file.replace("//", "/");
        let constant_name = asset_file_to_constant_name(&normalized);
        println!("Debug: asset_file = {}, constant_name = {}", normalized, constant_name);
        dart_code.push_str(&format!("  static const String {} = '{}';\n", constant_name, normalized));
    }
    
    // Class end
    dart_code.push_str("}\n");
    
    dart_code
}

fn asset_file_to_constant_name(asset_file: &str) -> String {
    // Convert file path to constant name
    // Example: "assets/images/logo.png" -> "assetsImagesLogoPng"
    let mut constant_name = String::new();
    
    for part in asset_file.split('/') {
        if !part.is_empty() {
            // Capitalize the first character
            if let Some(first_char) = part.chars().next() {
                constant_name.push(first_char.to_uppercase().next().unwrap());
                constant_name.push_str(&part[1..]);
            }
        }
    }
    
    // Replace special characters with underscores instead of removing them
    constant_name = constant_name.replace(['.', '-'], "_");
    
    constant_name
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
        assert!(dart_code.contains("AssetsImagesLogopng"));
        assert!(dart_code.contains("assets/images/logo.png"));
        assert!(dart_code.contains("AssetsDataSamplejson"));
        assert!(dart_code.contains("assets/data/sample.json"));
    }

    #[test]
    fn test_asset_file_to_constant_name() {
        let test_cases = vec![
            ("assets/images/logo.png", "AssetsImagesLogopng"),
            ("assets/data/sample.json", "AssetsDataSamplejson"),
            ("assets/fonts/Roboto-Regular.ttf", "AssetsFontsRobotoRegularttf"),
        ];
        
        for (input, expected) in test_cases {
            let result = asset_file_to_constant_name(input);
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