//! Verify that our symbol generation matches real nm output

use rfc2603::{SymbolBuilder, create_symbol_iterator};
use std::process::Command;

#[test]
fn test_symbols_match_real_nm_output() {
    let lib_path = "/home/user/test-symbols/target/debug/libtest_symbols.so";

    // Skip if library doesn't exist
    if !std::path::Path::new(lib_path).exists() {
        eprintln!("Skipping test - library not found at {}", lib_path);
        return;
    }

    // Extract real symbols from nm
    let output = Command::new("nm")
        .arg("-g")
        .arg(lib_path)
        .output()
        .expect("Failed to run nm");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut real_symbols = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let symbol = parts[2];
            if symbol.starts_with("_RNvCs5GYaaS9NRMV_12test_symbols") {
                real_symbols.push(symbol.to_string());
            }
        }
    }

    println!("Found {} real symbols from nm", real_symbols.len());

    // Test some specific functions we know exist
    let test_functions = vec![
        ("float_types", "_RNvCs5GYaaS9NRMV_12test_symbols11float_types"),
        ("integer_types", "_RNvCs5GYaaS9NRMV_12test_symbols13integer_types"),
        ("ptr_function", "_RNvCs5GYaaS9NRMV_12test_symbols12ptr_function"),
    ];

    for (func_name, expected_symbol) in test_functions {
        // Generate symbol using SymbolBuilder
        let generated = SymbolBuilder::new("test_symbols")
            .with_hash("5GYaaS9NRMV")
            .function(func_name)
            .build()
            .unwrap();

        println!("Testing {}: generated={}, expected={}", func_name, generated, expected_symbol);

        // Verify it matches the real nm output
        assert_eq!(generated, expected_symbol,
            "Generated symbol doesn't match nm output for {}", func_name);

        // Verify it exists in real symbols
        assert!(
            real_symbols.contains(&expected_symbol.to_string()),
            "Expected symbol {} not found in nm output",
            expected_symbol
        );
    }
}

#[test]
fn test_iterator_generates_correct_format() {
    // Test that create_symbol_iterator produces correctly formatted symbols
    let functions = &["foo", "bar", "baz"];
    let symbols: Vec<String> = create_symbol_iterator("mycrate", functions)
        .collect();

    // Each symbol should follow the pattern: _RNvC{len}{crate}{len}{func}
    assert_eq!(symbols[0], "_RNvC7mycrate3foo");
    assert_eq!(symbols[1], "_RNvC7mycrate3bar");
    assert_eq!(symbols[2], "_RNvC7mycrate3baz");

    // All should be valid v0 format
    for symbol in &symbols {
        assert!(symbol.starts_with("_R"), "Symbol should start with _R prefix");
        assert!(symbol.contains("Nv"), "Symbol should contain Nv (value namespace)");
        assert!(symbol.contains("C"), "Symbol should contain C (crate)");
    }
}
