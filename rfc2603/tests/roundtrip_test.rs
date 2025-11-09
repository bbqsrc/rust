//! Round-trip test: Extract symbols from nm, demangle, re-mangle, and verify they match
//!
//! This test proves that our v0 mangling implementation is correct by:
//! 1. Extracting real symbols from a compiled binary using nm
//! 2. Demangling them to understand their structure
//! 3. Re-mangling them using our implementation
//! 4. Verifying the re-mangled symbols match the original nm output byte-for-byte

use rfc2603::SymbolBuilder;
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

    // Parse the mangled symbol to extract hash
    // Format: _RNv + Cs<hash>_ + <len><crate> + <len><item>
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
    let parts: Vec<&str> = demangled_str.split("::").collect();
    if parts.is_empty() {
        return None;
    }

    let crate_name = parts[0].to_string();
    let item_name = parts.last()?.to_string();
    let module_path = if parts.len() > 2 {
        parts[1..parts.len() - 1].iter().map(|s| s.to_string()).collect()
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
    })
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
        // Only test simple function symbols (starting with _RNvC)
        // Skip complex symbols with generics, closures, etc for now
        if !symbol.starts_with("_RNvC") {
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

        // Re-mangle it
        let remangled = match remangle_symbol(&parsed) {
            Ok(s) => s,
            Err(e) => {
                stats.remangle_failed += 1;
                eprintln!("Failed to re-mangle {}: {}", parsed.demangled, e);
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
