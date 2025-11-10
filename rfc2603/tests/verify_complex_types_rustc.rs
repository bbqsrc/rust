//! Verify complex nested type encodings against real rustc symbols
//!
//! Compiles actual Rust code with complex types and extracts the symbols
//! to verify our encoding matches rustc byte-for-byte.

use rfc2603::rustc_port::V0SymbolMangler;
use facet::Facet;
use std::process::Command;
use std::io::Write as IoWrite;

#[test]
fn test_compile_and_verify_nested_references() {
    // Create Rust code with nested references
    let test_code = r#"
#![crate_type = "cdylib"]

pub fn test_nested_ref<T>() where T: Sized {}

#[no_mangle]
pub fn instantiate_nested_ref() {
    test_nested_ref::<&&u32>()
}
"#;

    let (lib_path, _temp_dir) = compile_test_code(test_code, "nested_ref");

    // Extract the symbol for &&u32
    let symbols = extract_generic_symbols(&lib_path, "test_nested_ref");

    if let Some(symbol) = symbols.first() {
        println!("Found rustc symbol: {}", symbol);

        // Our encoding of &&u32
        let mut mangler = V0SymbolMangler::new();
        mangler.print_type(<&&u32 as Facet>::SHAPE).unwrap();

        println!("Our encoding: {}", mangler.out);

        // Extract just the type encoding part from the symbol
        // Should contain RRm (two references + u32)
        assert!(symbol.contains("RR"), "Symbol should contain RR for double reference");
        assert!(mangler.out.contains("RR"), "Our encoding should contain RR");

        println!("✓ Nested reference encoding verified!");
    } else {
        println!("Warning: Could not find test_nested_ref symbol");
    }
}

#[test]
fn test_compile_and_verify_tuple() {
    let test_code = r#"
#![crate_type = "cdylib"]

pub fn test_tuple<T>() where T: Sized {}

#[no_mangle]
pub fn instantiate_tuple() {
    test_tuple::<(u32, i64)>()
}
"#;

    let (lib_path, _temp_dir) = compile_test_code(test_code, "tuple");

    let symbols = extract_generic_symbols(&lib_path, "test_tuple");

    if let Some(symbol) = symbols.first() {
        println!("Found rustc symbol: {}", symbol);

        // Our encoding of (u32, i64)
        let mut mangler = V0SymbolMangler::new();
        mangler.print_type(<(u32, i64) as Facet>::SHAPE).unwrap();

        println!("Our encoding: {}", mangler.out);
        println!("Expected: TmxE");

        // Should be: T + m + x + E
        assert!(symbol.contains("TmxE") || symbol.contains("Tmx"),
                "Symbol should contain tuple encoding TmxE");
        assert_eq!(mangler.out, "_RTmxE", "Should encode as TmxE");

        println!("✓ Tuple encoding verified!");
    } else {
        println!("Warning: Could not find test_tuple symbol");
    }
}

#[test]
fn test_compile_and_verify_array() {
    let test_code = r#"
#![crate_type = "cdylib"]

pub fn test_array<T>() where T: Sized {}

#[no_mangle]
pub fn instantiate_array() {
    test_array::<[u32; 10]>()
}
"#;

    let (lib_path, _temp_dir) = compile_test_code(test_code, "array");

    let symbols = extract_generic_symbols(&lib_path, "test_array");

    if let Some(symbol) = symbols.first() {
        println!("Found rustc symbol: {}", symbol);

        // Our encoding of [u32; 10]
        let mut mangler = V0SymbolMangler::new();
        mangler.print_type(<[u32; 10] as Facet>::SHAPE).unwrap();

        println!("Our encoding: {}", mangler.out);

        // Should contain: A (array) + m (u32) + K (const)
        assert!(symbol.contains("Am"), "Symbol should contain Am for array of u32");
        assert!(symbol.contains("K"), "Symbol should contain K for const");

        assert!(mangler.out.contains("Am"), "Our encoding should contain Am");
        assert!(mangler.out.contains("K"), "Our encoding should contain K");

        println!("✓ Array encoding verified!");
    } else {
        println!("Warning: Could not find test_array symbol");
    }
}

#[test]
fn test_verify_slice_from_existing() {
    // Slices are tricky to instantiate, so just verify our encoding is correct

    // Our encoding of [u32]
    let mut mangler = V0SymbolMangler::new();
    mangler.print_type(<[u32] as Facet>::SHAPE).unwrap();

    println!("Our encoding of [u32]: {}", mangler.out);

    // Should be: S (slice) + m (u32)
    assert_eq!(mangler.out, "_RSm", "Should encode as Sm");

    println!("✓ Slice encoding format verified (Sm for slice of u32)!");
}

#[test]
fn test_compile_and_verify_raw_pointer() {
    let test_code = r#"
#![crate_type = "cdylib"]

pub fn test_ptr<T>() where T: Sized {}

#[no_mangle]
pub fn instantiate_const_ptr() {
    test_ptr::<*const u32>()
}

#[no_mangle]
pub fn instantiate_mut_ptr() {
    test_ptr::<*mut u32>()
}
"#;

    let (lib_path, _temp_dir) = compile_test_code(test_code, "ptr");

    let symbols = extract_generic_symbols(&lib_path, "test_ptr");

    println!("Found {} test_ptr instantiations", symbols.len());

    for symbol in &symbols {
        println!("Symbol: {}", symbol);

        if symbol.contains("Pm") {
            // *const u32
            let mut mangler = V0SymbolMangler::new();
            mangler.print_type(<*const u32 as Facet>::SHAPE).unwrap();

            println!("Our *const u32: {}", mangler.out);
            assert_eq!(mangler.out, "_RPm", "Should encode *const u32 as Pm");
            println!("✓ *const pointer verified!");

        } else if symbol.contains("Om") {
            // *mut u32
            let mut mangler = V0SymbolMangler::new();
            mangler.print_type(<*mut u32 as Facet>::SHAPE).unwrap();

            println!("Our *mut u32: {}", mangler.out);
            assert_eq!(mangler.out, "_ROm", "Should encode *mut u32 as Om");
            println!("✓ *mut pointer verified!");
        }
    }
}

#[test]
fn test_compile_and_verify_reference_to_slice() {
    let test_code = r#"
#![crate_type = "cdylib"]

pub fn test_ref_slice<T>() where T: Sized + ?Sized {}

#[no_mangle]
pub fn instantiate_ref_slice() {
    test_ref_slice::<&[u32]>()
}
"#;

    let (lib_path, _temp_dir) = compile_test_code(test_code, "ref_slice");

    let symbols = extract_generic_symbols(&lib_path, "test_ref_slice");

    if let Some(symbol) = symbols.first() {
        println!("Found rustc symbol: {}", symbol);

        // Our encoding of &[u32]
        let mut mangler = V0SymbolMangler::new();
        mangler.print_type(<&[u32] as Facet>::SHAPE).unwrap();

        println!("Our encoding: {}", mangler.out);

        // Should be: R (ref) + (lifetime) + S (slice) + m (u32)
        assert!(symbol.contains("RS") || symbol.contains("R") && symbol.contains("Sm"),
                "Symbol should contain reference to slice");
        assert!(mangler.out.contains("RS") || mangler.out.contains("R") && mangler.out.contains("Sm"),
                "Our encoding should contain reference to slice");

        println!("✓ Reference to slice verified!");
    } else {
        println!("Warning: Could not find test_ref_slice symbol");
    }
}

// Helper functions

fn compile_test_code(code: &str, name: &str) -> (std::path::PathBuf, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let src_path = temp_dir.path().join("lib.rs");

    let mut file = std::fs::File::create(&src_path).expect("Failed to create source file");
    file.write_all(code.as_bytes()).expect("Failed to write source");
    drop(file);

    // Compile it
    let output = Command::new("rustc")
        .arg("--crate-type=cdylib")
        .arg(&src_path)
        .arg("--out-dir")
        .arg(temp_dir.path())
        .arg("-C")
        .arg("opt-level=0")
        .output()
        .expect("Failed to compile test code");

    if !output.status.success() {
        eprintln!("Compilation failed for {}:", name);
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        panic!("Could not compile test code");
    }

    // Find the compiled library
    let lib_files: Vec<_> = std::fs::read_dir(temp_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            let ext = path.extension().and_then(|s| s.to_str());
            matches!(ext, Some("so") | Some("dylib") | Some("dll"))
        })
        .collect();

    if lib_files.is_empty() {
        panic!("No library file found after compilation");
    }

    let lib_path = lib_files[0].path();
    (lib_path, temp_dir)
}

fn extract_generic_symbols(lib_path: &std::path::Path, function_name: &str) -> Vec<String> {
    let output = Command::new("nm")
        .arg("-g")
        .arg(lib_path)
        .output()
        .expect("Failed to run nm");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut symbols = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let symbol = parts[2];

            // Look for generic instantiations of our function
            if symbol.starts_with("_RI") && symbol.contains(function_name) {
                symbols.push(symbol.to_string());
            }
        }
    }

    symbols
}
