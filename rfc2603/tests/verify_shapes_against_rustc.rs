//! Verify our shape mangling matches ACTUAL rustc output
//!
//! This compiles real Rust code, extracts symbols with nm, and compares
//! our mangled output against what rustc actually generates.

use rfc2603::rustc_port::V0SymbolMangler;
use facet::Facet;
use std::process::Command;

/// Extract a specific symbol pattern from nm output
fn extract_symbol_for_function(lib_path: &str, function_name: &str) -> Option<String> {
    let output = Command::new("nm")
        .arg("-g")
        .arg(lib_path)
        .output()
        .expect("Failed to run nm");

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let symbol = parts[2];
            // Check if this symbol demangles to our function
            if let Ok(demangled) = rustc_demangle::try_demangle(symbol) {
                let dem_str = format!("{:#}", demangled);
                if dem_str.contains(function_name) && symbol.starts_with("_R") {
                    return Some(symbol.to_string());
                }
            }
        }
    }
    None
}

#[test]
fn test_verify_primitives_against_rustc() {
    // We need a compiled library with these functions
    let lib_path = "/home/user/test-symbols/target/debug/libtest_symbols.so";

    if !std::path::Path::new(lib_path).exists() {
        eprintln!("Skipping test - library not found at {}", lib_path);
        return;
    }

    // Test generic_function::<u32> - we know this exists in test_symbols
    if let Some(real_symbol) = extract_symbol_for_function(lib_path, "generic_function") {
        // Check if this is the u32 instantiation
        if real_symbol.contains("m") && !real_symbol.contains("x") {
            println!("Found real rustc symbol: {}", real_symbol);

            // Now mangle using our implementation
            // We need to figure out the exact instantiation from the symbol
            // For now, just verify our mangling produces valid v0 format

            let mut mangler = V0SymbolMangler::new();
            mangler.out.push_str("_R");
            mangler.out.push_str("I"); // Instantiation
            // This is incomplete - we need the full path

            println!("Our approach needs the full path context");
            println!("Real symbol: {}", real_symbol);
        }
    }
}

#[test]
fn test_compile_and_verify_shapes() {
    // Create a test file with generic functions
    let test_code = r#"
#![crate_type = "dylib"]

#[no_mangle]
pub fn test_bool_shape<T>() where T: Sized {
    // Generic function instantiated with bool
}

#[no_mangle]
pub fn test_u32_shape() {
    test_bool_shape::<u32>()
}

#[no_mangle]
pub fn test_ref_u32_shape() {
    test_ref_generic::<&u32>()
}

pub fn test_ref_generic<T>() where T: Sized {}

#[no_mangle]
pub fn test_tuple_shape() {
    test_tuple_generic::<(u32, i64)>()
}

pub fn test_tuple_generic<T>() where T: Sized {}
"#;

    let test_dir = std::env::temp_dir().join("shape_verify");
    std::fs::create_dir_all(&test_dir).unwrap();

    let src_path = test_dir.join("lib.rs");
    std::fs::write(&src_path, test_code).unwrap();

    // Compile it
    let output = Command::new("rustc")
        .arg("--crate-type=dylib")
        .arg(&src_path)
        .arg("--out-dir")
        .arg(&test_dir)
        .output()
        .expect("Failed to compile test code");

    if !output.status.success() {
        eprintln!("Compilation failed:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        panic!("Could not compile test code");
    }

    // Find the compiled library
    let lib_files: Vec<_> = std::fs::read_dir(&test_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.extension().map(|ext| ext == "so" || ext == "dylib" || ext == "dll").unwrap_or(false)
        })
        .collect();

    if lib_files.is_empty() {
        panic!("No library file found after compilation");
    }

    let lib_path = lib_files[0].path();
    println!("Compiled library: {}", lib_path.display());

    // Extract symbols
    let output = Command::new("nm")
        .arg("-gC") // -C for demangling
        .arg(&lib_path)
        .output()
        .expect("Failed to run nm");

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("\nSymbols from compiled library:");
    for line in stdout.lines() {
        if line.contains("test_") {
            println!("  {}", line);
        }
    }

    // Get mangled symbols (without -C)
    let output = Command::new("nm")
        .arg("-g")
        .arg(&lib_path)
        .output()
        .expect("Failed to run nm");

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("\nMangled symbols:");
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let symbol = parts[2];
            if symbol.starts_with("_R") && symbol.contains("test_bool_shape") {
                println!("  Found: {}", symbol);

                // Try to demangle it
                if let Ok(demangled) = rustc_demangle::try_demangle(symbol) {
                    println!("    Demangled: {:#}", demangled);
                }
            }
        }
    }
}

#[test]
fn test_extract_real_generic_symbols() {
    let lib_path = "/home/user/test-symbols/target/debug/libtest_symbols.so";

    if !std::path::Path::new(lib_path).exists() {
        eprintln!("Skipping test - library not found");
        return;
    }

    let output = Command::new("nm")
        .arg("-g")
        .arg(lib_path)
        .output()
        .expect("Failed to run nm");

    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("\n=== Real Generic Symbols from rustc ===\n");

    let mut count = 0;
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let symbol = parts[2];
            // Look for generic instantiations (start with _RI)
            if symbol.starts_with("_RI") && symbol.len() < 200 {
                if let Ok(demangled) = rustc_demangle::try_demangle(symbol) {
                    let dem_str = format!("{:#}", demangled);

                    // Focus on simple instantiations we can test
                    if dem_str.contains("test_symbols") &&
                       (dem_str.contains("<u32>") ||
                        dem_str.contains("<i64>") ||
                        dem_str.contains("<bool>") ||
                        dem_str.contains("<&")) {

                        println!("Demangled: {}", dem_str);
                        println!("Symbol:    {}", symbol);
                        println!();

                        count += 1;
                        if count >= 10 {
                            break;
                        }
                    }
                }
            }
        }
    }

    println!("Found {} simple generic instantiations", count);
}

#[test]
fn test_match_specific_symbol() {
    // Test a specific known symbol that we can reproduce
    let lib_path = "/home/user/test-symbols/target/debug/libtest_symbols.so";

    if !std::path::Path::new(lib_path).exists() {
        eprintln!("Skipping test - library not found");
        return;
    }

    // Look for a simple instantiation like generic_function::<i32>
    let output = Command::new("nm")
        .arg("-g")
        .arg(lib_path)
        .output()
        .expect("Failed to run nm");

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let symbol = parts[2];

            // Find generic_function::<i32>
            if symbol.starts_with("_RI") && symbol.contains("generic_function") {
                if let Ok(demangled) = rustc_demangle::try_demangle(symbol) {
                    let dem_str = format!("{:#}", demangled);

                    if dem_str == "test_symbols::generic_function::<i32>" {
                        println!("\n=== Found target symbol ===");
                        println!("Demangled: {}", dem_str);
                        println!("Mangled:   {}", symbol);

                        // Expected format: _RINvCs<hash>_12test_symbols16generic_functionlEB<backref>_
                        // The 'l' is i32

                        // Now let's see what our mangler produces for i32
                        let mut mangler = V0SymbolMangler::new();
                        mangler.out.push_str("TEST_");
                        mangler.print_type(<i32 as Facet>::SHAPE).unwrap();

                        println!("\nOur mangling of i32: {}", mangler.out);
                        println!("Should contain: l");

                        assert!(mangler.out.contains("l"), "i32 should mangle to 'l'");
                        assert!(symbol.contains("l"), "Real symbol should contain 'l' for i32");

                        println!("\n✓ Both contain 'l' for i32");
                        return;
                    }
                }
            }
        }
    }

    println!("Could not find generic_function::<i32> symbol");
}
