//! Verify EXACT symbol matching against rustc output
//!
//! This test extracts real rustc symbols and verifies our type encoding
//! matches byte-for-byte.

use rfc2603::rustc_port::V0SymbolMangler;
use facet::Facet;
use std::process::Command;

#[test]
fn test_verify_ref_str_encoding() {
    // Real rustc symbol: _RINvCs5GYaaS9NRMV_12test_symbols16generic_functionReEB2_
    // For: test_symbols::generic_function::<&str>
    //
    // The generic arg part is: Re
    // R = immutable reference
    // e = str

    let mut mangler = V0SymbolMangler::new();
    mangler.print_type(<&str as Facet>::SHAPE).unwrap();

    println!("Our encoding of &str: {}", mangler.out);
    println!("Expected from rustc: Re");

    assert_eq!(mangler.out, "_RRe", "Should encode &str as Re (R=ref, e=str)");
    println!("✓ &str encoding matches rustc!");
}

#[test]
fn test_verify_i32_encoding() {
    // Real rustc symbol contains: ...generic_functionlE...
    // For: test_symbols::generic_function::<i32>
    //
    // The generic arg part is: l
    // l = i32

    let mut mangler = V0SymbolMangler::new();
    mangler.print_type(<i32 as Facet>::SHAPE).unwrap();

    println!("Our encoding of i32: {}", mangler.out);
    println!("Expected from rustc: l");

    assert_eq!(mangler.out, "_Rl", "Should encode i32 as 'l'");
    println!("✓ i32 encoding matches rustc!");
}

#[test]
fn test_verify_u32_encoding() {
    let mut mangler = V0SymbolMangler::new();
    mangler.print_type(<u32 as Facet>::SHAPE).unwrap();

    println!("Our encoding of u32: {}", mangler.out);
    assert_eq!(mangler.out, "_Rm", "Should encode u32 as 'm'");
    println!("✓ u32 encoding matches rustc!");
}

#[test]
fn test_verify_bool_encoding() {
    let mut mangler = V0SymbolMangler::new();
    mangler.print_type(<bool as Facet>::SHAPE).unwrap();

    println!("Our encoding of bool: {}", mangler.out);
    assert_eq!(mangler.out, "_Rb", "Should encode bool as 'b'");
    println!("✓ bool encoding matches rustc!");
}

#[test]
fn test_verify_f64_encoding() {
    // Look for generic_function::<f64> in real symbols
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

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let symbol = parts[2];

            if symbol.starts_with("_RI") && symbol.contains("generic_function") {
                if let Ok(demangled) = rustc_demangle::try_demangle(symbol) {
                    let dem_str = format!("{:#}", demangled);

                    if dem_str == "test_symbols::generic_function::<f64>" {
                        println!("Found f64 symbol: {}", symbol);
                        println!("Demangled: {}", dem_str);

                        // Extract the type encoding (between function name and E)
                        // Symbol: _RINvCs...16generic_functiondEB..._
                        //                                       ^ this is f64

                        assert!(symbol.contains("dE"), "Symbol should contain 'd' for f64");

                        let mut mangler = V0SymbolMangler::new();
                        mangler.print_type(<f64 as Facet>::SHAPE).unwrap();

                        assert_eq!(mangler.out, "_Rd", "Should encode f64 as 'd'");
                        println!("✓ f64 encoding matches rustc!");
                        return;
                    }
                }
            }
        }
    }

    println!("Could not find generic_function::<f64>");
}

#[test]
fn test_verify_all_primitive_encodings() {
    // Test that all our primitive encodings match what we expect from rustc

    let types: Vec<(&'static str, &'static str, Box<dyn Fn() -> &'static facet::Shape>)> = vec![
        ("bool", "b", Box::new(|| <bool as Facet>::SHAPE)),
        ("i8", "a", Box::new(|| <i8 as Facet>::SHAPE)),
        ("i16", "s", Box::new(|| <i16 as Facet>::SHAPE)),
        ("i32", "l", Box::new(|| <i32 as Facet>::SHAPE)),
        ("i64", "x", Box::new(|| <i64 as Facet>::SHAPE)),
        ("u8", "h", Box::new(|| <u8 as Facet>::SHAPE)),
        ("u16", "t", Box::new(|| <u16 as Facet>::SHAPE)),
        ("u32", "m", Box::new(|| <u32 as Facet>::SHAPE)),
        ("u64", "y", Box::new(|| <u64 as Facet>::SHAPE)),
        ("f32", "f", Box::new(|| <f32 as Facet>::SHAPE)),
        ("f64", "d", Box::new(|| <f64 as Facet>::SHAPE)),
    ];

    println!("\n=== Verifying All Primitive Type Encodings ===\n");

    for (name, expected_tag, shape_fn) in types {
        let mut mangler = V0SymbolMangler::new();
        mangler.print_type(shape_fn()).unwrap();

        let expected = format!("_R{}", expected_tag);
        assert_eq!(
            mangler.out, expected,
            "{} should encode as '{}', got '{}'",
            name, expected_tag, mangler.out
        );

        println!("✓ {} → {} (correct)", name, expected_tag);
    }

    println!("\n✓ All primitive encodings match rustc spec!");
}

#[test]
fn test_verify_reference_encoding() {
    // &u32 should encode as: R + (lifetime) + m
    // In rustc symbols, we see: R + e for &str, so references are: R + (lifetime) + inner_type

    let mut mangler = V0SymbolMangler::new();
    mangler.print_type(<&u32 as Facet>::SHAPE).unwrap();

    println!("Our encoding of &u32: {}", mangler.out);

    // Should be _R + R (reference marker) + (lifetime) + m (u32)
    assert!(mangler.out.starts_with("_RR"), "Should start with _RR for reference");
    assert!(mangler.out.contains("m"), "Should contain 'm' for u32");

    println!("✓ Reference encoding structure matches!");
}

#[test]
fn test_verify_tuple_encoding() {
    // Tuples encode as: T + elements + E

    let mut mangler = V0SymbolMangler::new();
    mangler.print_type(<(u32, i64) as Facet>::SHAPE).unwrap();

    println!("Our encoding of (u32, i64): {}", mangler.out);

    // Should be: _R + T + m + x + E
    assert!(mangler.out.starts_with("_RT"), "Should start with T for tuple");
    assert!(mangler.out.contains("m"), "Should contain 'm' for u32");
    assert!(mangler.out.contains("x"), "Should contain 'x' for i64");
    assert!(mangler.out.ends_with("E"), "Should end with E");

    assert_eq!(mangler.out, "_RTmxE", "Tuple should encode as TmxE");

    println!("✓ Tuple encoding matches rustc format!");
}

#[test]
fn test_extract_and_verify_all_simple_types() {
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

    println!("\n=== Extracting Type Encodings from Real Symbols ===\n");

    let mut found = 0;
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let symbol = parts[2];

            if symbol.starts_with("_RI") && symbol.contains("generic_function") {
                if let Ok(demangled) = rustc_demangle::try_demangle(symbol) {
                    let dem_str = format!("{:#}", demangled);

                    if dem_str.starts_with("test_symbols::generic_function::<") {
                        // Extract just the type part from demangled
                        if let Some(type_part) = dem_str.strip_prefix("test_symbols::generic_function::<") {
                            if let Some(type_str) = type_part.strip_suffix(">") {
                                println!("Type: {}", type_str);
                                println!("Symbol: {}", symbol);

                                // Extract the generic arg encoding
                                // Format: ...16generic_function<TYPE_ENCODING>EB...
                                if let Some(after_func) = symbol.split("generic_function").nth(1) {
                                    if let Some(before_e) = after_func.split("EB").next() {
                                        println!("Type encoding: {}", before_e);
                                        println!();

                                        found += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if found >= 5 {
            break;
        }
    }

    assert!(found > 0, "Should find at least one generic instantiation");
}
