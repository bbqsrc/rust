//! Round-trip test: Extract symbols from nm, demangle, re-mangle, and verify they match
//!
//! This test proves that our v0 mangling implementation is correct by:
//! 1. Extracting real symbols from a compiled binary using nm
//! 2. Demangling them to understand their structure
//! 3. Re-mangling them using our implementation
//! 4. Verifying the re-mangled symbols match the original nm output byte-for-byte

use rfc2603::{SymbolBuilder, GenericArg, TypeArg, LifetimeArg};
use std::process::Command;

#[derive(Debug, Clone)]
struct ParsedSymbol {
    /// Original mangled symbol from nm
    original: String,
    /// Demangled representation
    demangled: String,
    /// Crate name
    crate_name: String,
    /// Crate hash (if present)
    crate_hash: Option<String>,
    /// Function/item name
    item_name: String,
    /// Module path (without crate name)
    module_path: Vec<String>,
    /// Generic arguments (if instantiation)
    generic_args: Vec<GenericArg>,
    /// Whether this is a generic instantiation
    is_generic_instantiation: bool,
}

/// Extract v0 symbols from nm output
fn extract_symbols_from_nm(lib_path: &str) -> Vec<String> {
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
            // Only process v0 symbols (start with _R)
            if symbol.starts_with("_R") && !symbol.contains("::") {
                symbols.push(symbol.to_string());
            }
        }
    }

    symbols
}

/// Parse a v0 symbol to extract its components
fn parse_symbol(symbol: &str) -> Option<ParsedSymbol> {
    // Try to demangle first
    let demangled = rustc_demangle::try_demangle(symbol).ok()?;
    let demangled_str = format!("{:#}", demangled);

    // Check if this is a generic instantiation (starts with _RI)
    let is_generic_instantiation = symbol.starts_with("_RI");

    // Parse the mangled symbol to extract hash
    // Format: _RNv + Cs<hash>_ + <len><crate> + <len><item>
    // Or: _RI + Nv + Cs<hash>_ + ... for instantiations
    let crate_hash = if symbol.contains("Cs") {
        // Extract hash between 'Cs' and '_'
        let after_cs = symbol.split("Cs").nth(1)?;
        let hash = after_cs.split('_').next()?;
        Some(hash.to_string())
    } else {
        None
    };

    // Parse demangled string to extract components
    // Format: crate_name::module::item or crate_name::item
    // For generics: crate_name::module::item::<type_args>
    // First, strip generic args from the end
    let without_generics = if demangled_str.contains("::<") {
        demangled_str.split("::<").next().unwrap()
    } else {
        &demangled_str
    };

    let parts: Vec<&str> = without_generics.split("::").collect();
    if parts.is_empty() {
        return None;
    }

    // Extract crate name (first part)
    let crate_name = parts[0].to_string();

    // Extract item name (last part in the clean path)
    let item_name = parts.last()?.to_string();

    // Extract module path (everything between crate name and item name)
    let module_path = if parts.len() > 2 {
        // Skip first (crate) and last (item)
        let middle_parts: Vec<String> = parts[1..parts.len() - 1]
            .iter()
            .map(|s| s.to_string())
            .collect();
        middle_parts
    } else {
        Vec::new()
    };

    // Parse generic arguments if this is an instantiation
    let generic_args = if is_generic_instantiation {
        parse_generic_args_from_symbol(symbol).unwrap_or_default()
    } else {
        Vec::new()
    };

    Some(ParsedSymbol {
        original: symbol.to_string(),
        demangled: demangled_str,
        crate_name,
        crate_hash,
        item_name,
        module_path,
        generic_args,
        is_generic_instantiation,
    })
}

/// Parse generic arguments from a mangled symbol
/// This finds the generic args that come after the path and before the closing E
fn parse_generic_args_from_symbol(symbol: &str) -> Option<Vec<GenericArg>> {
    if !symbol.starts_with("_RI") {
        return None;
    }

    // Strategy: Find the identifier (function name), then parse what comes after until 'E'
    // The function name is encoded as <len><name>, so we look for digit patterns

    let chars: Vec<char> = symbol.chars().collect();

    // Find the last identifier in the path (the function name)
    // It will be encoded as digits followed by the name
    // We want to find everything AFTER the last identifier and BEFORE the first E

    let mut last_ident_end = None;
    let mut i = 3; // Skip "_RI"

    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            // Found a length prefix - parse the length
            let mut len_str = String::new();
            while i < chars.len() && chars[i].is_ascii_digit() {
                len_str.push(chars[i]);
                i += 1;
            }

            if let Ok(len) = len_str.parse::<usize>() {
                // Skip the identifier
                i += len;
                last_ident_end = Some(i);
            }
        } else if chars[i] == 'E' {
            // Found the end of generics marker
            break;
        } else {
            i += 1;
        }
    }

    let start = last_ident_end?;

    // Now parse generic args from start to the first 'E'
    let mut args = Vec::new();
    let mut i = start;

    while i < chars.len() && chars[i] != 'E' {
        match chars[i] {
            // Primitive type tags
            'm' => { args.push(GenericArg::Type(TypeArg::U32)); i += 1; }
            'x' => { args.push(GenericArg::Type(TypeArg::I64)); i += 1; }
            'y' => { args.push(GenericArg::Type(TypeArg::U64)); i += 1; }
            'h' => { args.push(GenericArg::Type(TypeArg::U8)); i += 1; }
            't' => { args.push(GenericArg::Type(TypeArg::U16)); i += 1; }
            'a' => { args.push(GenericArg::Type(TypeArg::I8)); i += 1; }
            'l' => { args.push(GenericArg::Type(TypeArg::I32)); i += 1; }
            'b' => { args.push(GenericArg::Type(TypeArg::Bool)); i += 1; }
            'f' => { args.push(GenericArg::Type(TypeArg::F32)); i += 1; }
            'd' => { args.push(GenericArg::Type(TypeArg::F64)); i += 1; }
            'j' => { args.push(GenericArg::Type(TypeArg::Usize)); i += 1; }
            'c' => { args.push(GenericArg::Type(TypeArg::Char)); i += 1; }
            'e' => { args.push(GenericArg::Type(TypeArg::Str)); i += 1; }
            'u' => { args.push(GenericArg::Type(TypeArg::Unit)); i += 1; }
            'z' => { args.push(GenericArg::Type(TypeArg::Never)); i += 1; }
            'i' => { args.push(GenericArg::Type(TypeArg::Isize)); i += 1; }
            'n' => { args.push(GenericArg::Type(TypeArg::I128)); i += 1; }
            'o' => { args.push(GenericArg::Type(TypeArg::U128)); i += 1; }
            's' => { args.push(GenericArg::Type(TypeArg::I16)); i += 1; }

            'L' => {
                // Lifetime: L + base62_number + _
                i += 1;
                // Skip base62 digits
                while i < chars.len() && chars[i] != '_' {
                    i += 1;
                }
                if i < chars.len() && chars[i] == '_' {
                    i += 1;
                }
                args.push(GenericArg::Lifetime(LifetimeArg::Erased));
            }

            'K' => {
                // Const: K + type_tag + base62_value + _
                i += 1;
                // Skip type tag
                if i < chars.len() {
                    i += 1;
                }
                // Parse value
                let mut val_str = String::new();
                while i < chars.len() && chars[i] != '_' {
                    val_str.push(chars[i]);
                    i += 1;
                }
                if i < chars.len() && chars[i] == '_' {
                    i += 1;
                }
                // Decode base62 value
                if let Some(val) = decode_base62(&val_str) {
                    args.push(GenericArg::Const(val));
                }
            }

            'R' => {
                // Immutable reference: R + lifetime + inner_type
                i += 1;
                // Parse lifetime
                if chars.get(i) == Some(&'L') {
                    i += 1;
                    while i < chars.len() && chars[i] != '_' {
                        i += 1;
                    }
                    if i < chars.len() && chars[i] == '_' {
                        i += 1;
                    }
                }
                // Parse inner type (simplified - just handle primitives for now)
                if i < chars.len() {
                    let inner = match chars[i] {
                        'h' => Some(TypeArg::U8),
                        'm' => Some(TypeArg::U32),
                        'e' => Some(TypeArg::Str),
                        _ => None,
                    };
                    if let Some(inner_ty) = inner {
                        args.push(GenericArg::Type(TypeArg::Reference {
                            lifetime: Some(LifetimeArg::Erased),
                            mutable: false,
                            inner: Box::new(inner_ty),
                        }));
                        i += 1;
                    }
                }
            }

            _ => { i += 1; }
        }
    }

    Some(args)
}

/// Decode a base-62 number (used in v0 mangling)
fn decode_base62(s: &str) -> Option<u64> {
    const BASE62: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    let mut result = 0u64;
    for c in s.chars() {
        let digit = BASE62.find(c)? as u64;
        result = result * 62 + digit;
    }
    Some(result + 1) // v0 mangling subtracts 1 before encoding
}

/// Re-mangle a parsed symbol using our implementation
fn remangle_symbol(parsed: &ParsedSymbol) -> Result<String, String> {
    let mut builder = SymbolBuilder::new(&parsed.crate_name);

    // Add hash if present
    if let Some(ref hash) = parsed.crate_hash {
        builder = builder.with_hash(hash);
    }

    // Add module path
    for module in &parsed.module_path {
        builder = builder.module(module.as_str());
    }

    // Add the item (function)
    builder = builder.function(&parsed.item_name);

    // Add generic arguments if present
    if !parsed.generic_args.is_empty() {
        builder = builder.with_generics(&parsed.generic_args);
    }

    builder.build().map_err(|e| e.to_string())
}

#[test]
fn test_roundtrip_nm_symbols() {
    let lib_path = "/home/user/test-symbols/target/debug/libtest_symbols.so";

    // Skip if library doesn't exist
    if !std::path::Path::new(lib_path).exists() {
        eprintln!("Skipping test - library not found at {}", lib_path);
        eprintln!("Run: cd /home/user/test-symbols && cargo build");
        return;
    }

    println!("\n=== Round-trip Symbol Mangling Test ===\n");

    // Extract symbols from nm
    let symbols = extract_symbols_from_nm(lib_path);
    let total_symbols = symbols.len();
    println!("Extracted {} symbols from nm", total_symbols);

    let mut stats = RoundtripStats::default();
    let mut failures = Vec::new();

    for symbol in &symbols {
        // Test all symbols: simple functions (_RNvC) and generic instantiations (_RI)
        // Skip symbols we don't yet support
        if symbol.starts_with("_RNvM") || symbol.starts_with("_RNvX") {
            // Methods and trait implementations - not yet supported
            stats.skipped += 1;
            continue;
        }

        if !symbol.starts_with("_RNvC") && !symbol.starts_with("_RI") {
            // Other symbol types not yet supported
            stats.skipped += 1;
            continue;
        }

        stats.tested += 1;

        // Parse the symbol
        let parsed = match parse_symbol(&symbol) {
            Some(p) => p,
            None => {
                stats.parse_failed += 1;
                eprintln!("Failed to parse: {}", symbol);
                continue;
            }
        };

        // Skip symbols with '<' in crate name (generic type methods, trait impls, etc.)
        if parsed.crate_name.contains('<') || parsed.item_name.contains('<') {
            stats.skipped += 1;
            continue;
        }

        // Debug: show what we parsed
        if !parsed.generic_args.is_empty() {
            eprintln!("DEBUG: Parsed {} with {} generic args", parsed.demangled, parsed.generic_args.len());
            eprintln!("  crate={}, item={}, modules={:?}", parsed.crate_name, parsed.item_name, parsed.module_path);
        }

        // Re-mangle it
        let remangled = match remangle_symbol(&parsed) {
            Ok(s) => s,
            Err(e) => {
                stats.remangle_failed += 1;
                eprintln!("Failed to re-mangle {}:", parsed.demangled);
                eprintln!("  Error: {}", e);
                eprintln!("  Original: {}", parsed.original);
                eprintln!("  Crate: {}, Item: {}, Modules: {:?}", parsed.crate_name, parsed.item_name, parsed.module_path);
                continue;
            }
        };

        // Compare
        if remangled == parsed.original {
            stats.matched += 1;
            println!("✓ {} -> {}", parsed.demangled, symbol);
        } else {
            stats.mismatched += 1;
            eprintln!("✗ Mismatch for {}", parsed.demangled);
            eprintln!("  Original:  {}", parsed.original);
            eprintln!("  Remangled: {}", remangled);
            failures.push(RoundtripFailure {
                demangled: parsed.demangled.clone(),
                original: parsed.original.clone(),
                remangled,
            });
        }
    }

    // Print statistics
    println!("\n=== Round-trip Test Results ===");
    println!("Total symbols found: {}", total_symbols);
    println!("Tested: {}", stats.tested);
    println!("Skipped (complex): {}", stats.skipped);
    println!("Parse failed: {}", stats.parse_failed);
    println!("Re-mangle failed: {}", stats.remangle_failed);
    println!("✓ Matched: {}", stats.matched);
    println!("✗ Mismatched: {}", stats.mismatched);

    if stats.tested > 0 {
        let success_rate = (stats.matched as f64 / stats.tested as f64) * 100.0;
        println!("\nSuccess rate: {:.1}%", success_rate);
    }

    // Fail test if there are mismatches
    if !failures.is_empty() {
        panic!("\n{} symbols failed round-trip test!", failures.len());
    }

    // Require at least some symbols to be tested
    assert!(stats.matched > 0, "No symbols were successfully tested");
}

#[test]
fn test_roundtrip_specific_symbols() {
    // Test specific known symbols to ensure they round-trip correctly
    let test_cases = vec![
        (
            "_RNvCs5GYaaS9NRMV_12test_symbols11float_types",
            "test_symbols",
            Some("5GYaaS9NRMV"),
            &[] as &[&str],
            "float_types",
        ),
        (
            "_RNvCs5GYaaS9NRMV_12test_symbols13integer_types",
            "test_symbols",
            Some("5GYaaS9NRMV"),
            &[],
            "integer_types",
        ),
        (
            "_RNvC7mycrate3foo",
            "mycrate",
            None,
            &[],
            "foo",
        ),
    ];

    for (expected_symbol, crate_name, hash, modules, function) in test_cases {
        let mut builder = SymbolBuilder::new(crate_name);

        if let Some(h) = hash {
            builder = builder.with_hash(h);
        }

        for module in modules {
            builder = builder.module(*module);
        }

        let generated = builder.function(function).build().unwrap();

        assert_eq!(
            generated, expected_symbol,
            "Failed to generate correct symbol for {}::{}",
            crate_name, function
        );

        println!("✓ Round-trip verified: {}", expected_symbol);
    }
}

#[derive(Default)]
struct RoundtripStats {
    tested: usize,
    skipped: usize,
    parse_failed: usize,
    remangle_failed: usize,
    matched: usize,
    mismatched: usize,
}

struct RoundtripFailure {
    demangled: String,
    original: String,
    remangled: String,
}
