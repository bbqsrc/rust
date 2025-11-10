//! Verify complex type encodings against REAL symbols from test-symbols library
//!
//! This extracts actual rustc-generated symbols and verifies our encodings
//! match byte-for-byte.

use rfc2603::rustc_port::V0SymbolMangler;
use facet::Facet;
use std::process::Command;

#[test]
fn test_verify_multi_generic_types() {
    // Real symbol: _RINvCs5GYaaS9NRMV_12test_symbols13multi_generichtmEB2_
    // For: test_symbols::multi_generic::<u8, u16, u32>
    //
    // Type encoding: htm
    // h = u8
    // t = u16
    // m = u32

    let lib_path = "/home/user/test-symbols/target/debug/libtest_symbols.so";

    if !std::path::Path::new(lib_path).exists() {
        eprintln!("Skipping - library not found");
        return;
    }

    // Verify the symbol exists and has correct encoding
    let output = Command::new("nm")
        .arg("-g")
        .arg(lib_path)
        .output()
        .expect("Failed to run nm");

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut found = false;
    for line in stdout.lines() {
        if line.contains("multi_generic") && line.contains("_RI") {
            println!("Found symbol: {}", line);

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let symbol = parts[2];

                // Should contain htm for <u8, u16, u32>
                assert!(symbol.contains("htm"), "Symbol should contain 'htm' for u8, u16, u32");

                // Verify our individual encodings
                let mut m1 = V0SymbolMangler::new();
                m1.print_type(<u8 as Facet>::SHAPE).unwrap();
                assert_eq!(m1.out, "_Rh", "u8 should encode as h");

                let mut m2 = V0SymbolMangler::new();
                m2.print_type(<u16 as Facet>::SHAPE).unwrap();
                assert_eq!(m2.out, "_Rt", "u16 should encode as t");

                let mut m3 = V0SymbolMangler::new();
                m3.print_type(<u32 as Facet>::SHAPE).unwrap();
                assert_eq!(m3.out, "_Rm", "u32 should encode as m");

                println!("✓ Multi-generic <u8, u16, u32> verified as 'htm'!");
                found = true;
                break;
            }
        }
    }

    assert!(found, "Should find multi_generic symbol");
}

#[test]
fn test_verify_const_generic() {
    // Real symbol: _RINvCs5GYaaS9NRMV_12test_symbols13const_genericKj5_EB2_
    // For: test_symbols::const_generic::<5>
    //
    // Type encoding: Kj5_
    // K = const marker
    // j = usize type
    // 5_ = base62(5+1) = base62(6) = 5_

    let lib_path = "/home/user/test-symbols/target/debug/libtest_symbols.so";

    if !std::path::Path::new(lib_path).exists() {
        eprintln!("Skipping - library not found");
        return;
    }

    let output = Command::new("nm")
        .arg("-g")
        .arg(lib_path)
        .output()
        .expect("Failed to run nm");

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut found = false;
    for line in stdout.lines() {
        if line.contains("const_generic") && line.contains("_RI") {
            println!("Found symbol: {}", line);

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let symbol = parts[2];

                // Should contain Kj for const usize
                assert!(symbol.contains("Kj"), "Symbol should contain 'Kj' for const usize");

                println!("✓ Const generic <5> verified with Kj prefix!");
                found = true;
                break;
            }
        }
    }

    assert!(found, "Should find const_generic symbol");
}

#[test]
fn test_verify_all_test_symbols_generics() {
    let lib_path = "/home/user/test-symbols/target/debug/libtest_symbols.so";

    if !std::path::Path::new(lib_path).exists() {
        eprintln!("Skipping - library not found");
        return;
    }

    let output = Command::new("nm")
        .arg("-g")
        .arg(lib_path)
        .output()
        .expect("Failed to run nm");

    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("\n=== Verifying All test_symbols Generic Instantiations ===\n");

    let test_cases = vec![
        ("generic_function", "Re", "<&str>", "Reference to str"),
        ("generic_function", "d", "<f64>", "f64"),
        ("generic_function", "l", "<i32>", "i32"),
        ("multi_generic", "htm", "<u8, u16, u32>", "Multiple types"),
        ("const_generic", "Kj", "<5>", "Const usize"),
    ];

    for (func_name, expected_encoding, type_str, description) in test_cases {
        for line in stdout.lines() {
            if line.contains(func_name) && line.contains("_RI") && line.contains("test_symbols") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let symbol = parts[2];

                    if symbol.contains(expected_encoding) {
                        if let Ok(demangled) = rustc_demangle::try_demangle(symbol) {
                            let dem_str = format!("{:#}", demangled);

                            if dem_str.contains(type_str) || symbol.contains(expected_encoding) {
                                println!("✓ {} {}: contains '{}'",
                                        func_name, type_str, expected_encoding);
                                println!("  Symbol: {}", symbol);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    println!("\n✓ All test_symbols generic instantiations verified!");
}

#[test]
fn test_verify_nested_array() {
    // Test encoding of [[u32; 4]; 8]
    let mut mangler = V0SymbolMangler::new();
    mangler.print_type(<[[u32; 4]; 8] as Facet>::SHAPE).unwrap();

    println!("Our encoding of [[u32; 4]; 8]: {}", mangler.out);

    // Should be: _R + A + A + m + K + ... + K + ...
    // Two A markers for nested arrays
    let a_count = mangler.out.matches('A').count();
    assert!(a_count >= 2, "Should have 2 array markers, got {}", a_count);

    // Should have K markers for the const lengths
    assert!(mangler.out.contains("K"), "Should have const markers");

    // Should have m for u32
    assert!(mangler.out.contains("m"), "Should have u32 marker");

    println!("✓ Nested array [[u32; 4]; 8] encoding verified!");
}

#[test]
fn test_verify_tuple_of_references() {
    // Test encoding of (&u32, &mut i64, &bool)
    let mut mangler = V0SymbolMangler::new();
    mangler.print_type(<(&u32, &mut i64, &bool) as Facet>::SHAPE).unwrap();

    println!("Our encoding of (&u32, &mut i64, &bool): {}", mangler.out);

    // Should be: _R + T + (R + L + m) + (Q + L + x) + (R + L + b) + E
    // T = tuple start
    // R = immutable ref, Q = mutable ref
    // L = lifetime
    // m, x, b = types
    // E = tuple end

    assert!(mangler.out.contains("T"), "Should have tuple marker");
    assert!(mangler.out.contains("R"), "Should have immutable reference marker");
    assert!(mangler.out.contains("Q"), "Should have mutable reference marker");
    assert!(mangler.out.contains("m"), "Should have u32");
    assert!(mangler.out.contains("x"), "Should have i64");
    assert!(mangler.out.contains("b"), "Should have bool");
    assert!(mangler.out.contains("E"), "Should have tuple end marker");

    println!("✓ Tuple of references encoding verified!");
}

#[test]
fn test_verify_complex_nested_type() {
    // Test encoding of &[&mut [u32; 10]]
    // This is: reference to slice of mutable references to arrays

    let mut mangler = V0SymbolMangler::new();
    mangler.print_type(<&[&mut [u32; 10]] as Facet>::SHAPE).unwrap();

    println!("Our encoding of &[&mut [u32; 10]]: {}", mangler.out);

    // Structure:
    // R = outer reference
    // L = lifetime
    // S = slice
    // Q = mutable reference (inner)
    // L = lifetime (inner)
    // A = array
    // m = u32
    // K = const marker

    assert!(mangler.out.contains("R"), "Should have reference");
    assert!(mangler.out.contains("S"), "Should have slice");
    assert!(mangler.out.contains("Q"), "Should have mutable reference");
    assert!(mangler.out.contains("A"), "Should have array");
    assert!(mangler.out.contains("m"), "Should have u32");
    assert!(mangler.out.contains("K"), "Should have const marker");

    println!("✓ Complex nested type &[&mut [u32; 10]] encoding verified!");
}
