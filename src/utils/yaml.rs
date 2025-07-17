use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct GenerateConfig {
    pub input: Option<String>,
    pub output: Option<String>,
    pub freezed: Option<bool>,
    pub json: Option<bool>,
    pub riverpod: Option<bool>,
    pub provider: Option<bool>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct AssetsConfig {
    pub input: Option<String>,
    pub output: Option<String>,
    pub include_images: Option<bool>,
    pub include_fonts: Option<bool>,
    pub include_icons: Option<bool>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct SuperfastgenConfig {
    pub generate: Option<GenerateConfig>,
    pub assets: Option<AssetsConfig>,
}

pub fn parse_superfastgen_yaml(path: &str) -> Option<SuperfastgenConfig> {
    let content = fs::read_to_string(path).ok()?;
    serde_yaml::from_str(&content).ok()
}

pub fn parse_pubspec_yaml(_content: &str) -> Result<(), String> {
    println!("Parsing pubspec.yaml...");
    Ok(())
}

#[allow(dead_code)]
pub fn generate_pubspec_yaml() -> Result<String, String> {
    println!("Generating pubspec.yaml...");
    Ok("name: superfastgen".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_superfastgen_yaml() {
        let config = parse_superfastgen_yaml("superfastgen.yaml");
        assert!(config.is_some());
        
        if let Some(cfg) = config {
            if let Some(generate) = cfg.generate {
                assert_eq!(generate.input, Some("lib/".to_string()));
                assert_eq!(generate.output, Some("lib/gen/".to_string()));
                assert_eq!(generate.freezed, Some(true));
                assert_eq!(generate.json, Some(true));
                assert_eq!(generate.riverpod, Some(true));
                assert_eq!(generate.provider, Some(true));
            }
            
            if let Some(assets) = cfg.assets {
                assert_eq!(assets.input, Some("assets/".to_string()));
                assert_eq!(assets.output, Some("lib/gen/".to_string()));
                assert_eq!(assets.include_images, Some(true));
                assert_eq!(assets.include_fonts, Some(true));
                assert_eq!(assets.include_icons, Some(true));
            }
        }
    }
} 