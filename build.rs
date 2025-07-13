use std::path::PathBuf;

fn main() {
    // Source directory for tree-sitter-dart
    let tree_sitter_dart_dir = PathBuf::from("tree-sitter-dart");
    
    // Warn if tree-sitter-dart directory does not exist
    if !tree_sitter_dart_dir.exists() {
        println!("cargo:warning=tree-sitter-dart directory not found. Please run: git submodule update --init");
        return;
    }
    
    // Build the parser for tree-sitter-dart
    let parser_c = tree_sitter_dart_dir.join("src/parser.c");
    let scanner_c = tree_sitter_dart_dir.join("src/scanner.c");
    
    if parser_c.exists() {
        cc::Build::new()
            .file(parser_c)
            .file(scanner_c)
            .include(&tree_sitter_dart_dir.join("src"))
            .flag_if_supported("-Wno-unused-parameter")  // Suppress unused parameter warnings
            .flag_if_supported("-Wno-unused-function")   // Suppress unused function warnings
            .compile("tree-sitter-dart");
        
        println!("cargo:rerun-if-changed=tree-sitter-dart/src/parser.c");
        println!("cargo:rerun-if-changed=tree-sitter-dart/src/scanner.c");
        println!("cargo:rerun-if-changed=tree-sitter-dart/src/parser.h");
    } else {
        println!("cargo:warning=tree-sitter-dart parser.c not found");
    }
} 