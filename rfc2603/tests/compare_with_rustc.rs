//! Integration test: Compare our generated symbols with rustc's actual v0 symbols

use dlopen2::wrapper::{Container, WrapperApi};
use rfc2603::{push_ident, push_integer_62};
use stele_inventory::ExportedItem;
use std::collections::HashMap;
use std::process::Command;

#[derive(WrapperApi)]
struct SteleApi {
    stele_get_exports: unsafe fn() -> &'static [ExportedItem],
}

/// Generate a v0 mangled symbol for a function
fn mangle_function(crate_name: &str, module_path: &str, function_name: &str, crate_hash: Option<&str>) -> String {
    let mut out = String::from("_R");
    out.push_str("Nv");

    if let Some(hash) = crate_hash {
        out.push('C');
        out.push('s');
        out.push_str(hash);
        out.push('_');
        push_ident(crate_name, &mut out);
    } else {
        out.push('C');
        push_ident(crate_name, &mut out);
    }

    if !module_path.is_empty() && module_path != crate_name {
        for segment in module_path.split("::").skip(1) {
            out.push_str("Nt");
            push_ident(segment, &mut out);
        }
    }

    push_ident(function_name, &mut out);
    out
}

/// Generate a v0 mangled symbol for a method
fn mangle_method(
    crate_name: &str,
    module_path: &str,
    type_name: &str,
    method_name: &str,
    crate_hash: Option<&str>,
) -> String {
    let mut out = String::from("_R");
    let start_offset = 2;

    out.push_str("Nv");
    out.push('M');
    out.push('s');
    out.push('a');
    out.push('_');

    let impl_path_pos = out.len();

    if let Some(hash) = crate_hash {
        out.push('C');
        out.push('s');
        out.push_str(hash);
        out.push('_');
        push_ident(crate_name, &mut out);
    } else {
        out.push('C');
        push_ident(crate_name, &mut out);
    }

    if !module_path.is_empty() && module_path != crate_name {
        for segment in module_path.split("::").skip(1) {
            out.push_str("Nt");
            push_ident(segment, &mut out);
        }
    }

    out.push_str("Nt");
    let offset = impl_path_pos - start_offset;
    out.push('B');
    push_integer_62(offset as u64, &mut out);
    push_ident(type_name, &mut out);
    push_ident(method_name, &mut out);

    out
}

/// Generate a v0 mangled symbol for a struct
fn mangle_type(crate_name: &str, module_path: &str, type_name: &str, crate_hash: Option<&str>) -> String {
    let mut out = String::from("_R");

    if let Some(hash) = crate_hash {
        out.push('C');
        out.push('s');
        out.push_str(hash);
        out.push('_');
        push_ident(crate_name, &mut out);
    } else {
        out.push('C');
        push_ident(crate_name, &mut out);
    }

    if !module_path.is_empty() && module_path != crate_name {
        for segment in module_path.split("::").skip(1) {
            out.push_str("Nt");
            push_ident(segment, &mut out);
        }
    }

    push_ident(type_name, &mut out);
    out
}

/// Extract actual v0 symbols from compiled library using nm
fn extract_rustc_symbols(lib_path: &str) -> HashMap<String, String> {
    let output = Command::new("nm")
        .arg("-g")
        .arg(lib_path)
        .output()
        .expect("Failed to run nm");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut symbols = HashMap::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let symbol = parts[2];
            if symbol.starts_with("_RNv") || symbol.starts_with("_RC") {
                // Extract a key from the symbol for matching
                // For functions: use the function name
                // For methods: use type::method
                if let Some(demangled) = rustc_demangle::try_demangle(symbol).ok() {
                    let demangled_str = format!("{:#}", demangled);
                    symbols.insert(demangled_str, symbol.to_string());
                }
            }
        }
    }

    symbols
}

#[test]
fn test_symbols_match_rustc() {
    let lib_path = "/home/user/test-symbols/target/debug/libtest_symbols.so";

    // Check if library exists
    if !std::path::Path::new(lib_path).exists() {
        eprintln!("Test library not found at {}", lib_path);
        eprintln!("Run: cd /home/user/test-symbols && cargo build");
        panic!("Test library not found");
    }

    // Extract actual rustc symbols
    let rustc_symbols = extract_rustc_symbols(lib_path);

    // Load stele exports
    let container: Container<SteleApi> =
        unsafe { Container::load(lib_path).expect("Failed to load library") };
    let exports = unsafe { container.stele_get_exports() };

    let crate_hash = Some("5GYaaS9NRMV");
    let mut matches = 0;
    let mut mismatches = Vec::new();

    for item in exports.iter() {
        let (generated_symbol, item_name) = match item {
            ExportedItem::Struct(s) => {
                let crate_name = s.module_path.split("::").next().unwrap_or("unknown");
                let symbol = mangle_type(crate_name, s.module_path, s.name, crate_hash);
                (symbol, format!("test_symbols::{}", s.name))
            }
            ExportedItem::Enum(e) => {
                let crate_name = e.module_path.split("::").next().unwrap_or("unknown");
                let symbol = mangle_type(crate_name, e.module_path, e.name, crate_hash);
                (symbol, format!("test_symbols::{}", e.name))
            }
            ExportedItem::Function(f) => {
                let crate_name = f.module_path.split("::").next().unwrap_or("unknown");
                let symbol = mangle_function(crate_name, f.module_path, f.name, crate_hash);
                (symbol, format!("{}::{}", f.module_path.replace("::", "::"), f.name))
            }
            ExportedItem::Method(m) => {
                let crate_name = m.module_path.split("::").next().unwrap_or("unknown");
                let symbol = mangle_method(crate_name, m.module_path, m.receiver_type, m.name, crate_hash);
                (symbol, format!("{}::{}::{}", m.module_path, m.receiver_type, m.name))
            }
        };

        // Check if this symbol exists in rustc output
        let found_in_rustc = rustc_symbols.values().any(|s| s == &generated_symbol);

        if found_in_rustc {
            matches += 1;
            println!("✓ {} -> {}", item_name, generated_symbol);
        } else {
            mismatches.push((item_name.clone(), generated_symbol.clone()));
            eprintln!("✗ {} -> {} (not found in rustc output)", item_name, generated_symbol);
        }

        // Verify it can be demangled
        if let Ok(demangled) = rustc_demangle::try_demangle(&generated_symbol) {
            println!("  Demangled: {:#}", demangled);
        } else {
            eprintln!("  WARNING: Symbol cannot be demangled!");
        }
    }

    println!("\n=== Summary ===");
    println!("Generated symbols: {}", exports.len());
    println!("Matches with rustc: {}", matches);
    println!("Mismatches: {}", mismatches.len());

    if !mismatches.is_empty() {
        eprintln!("\nMismatched symbols:");
        for (name, symbol) in &mismatches {
            eprintln!("  {} -> {}", name, symbol);
        }
    }

    // Test passes if at least 80% of symbols match
    let match_rate = matches as f64 / exports.len() as f64;
    assert!(
        match_rate >= 0.8,
        "Only {:.1}% of symbols matched rustc output (expected >= 80%)",
        match_rate * 100.0
    );
}

#[test]
fn test_specific_symbols() {
    let lib_path = "/home/user/test-symbols/target/debug/libtest_symbols.so";

    if !std::path::Path::new(lib_path).exists() {
        eprintln!("Test library not found, skipping");
        return;
    }

    let crate_hash = Some("5GYaaS9NRMV");

    // Test specific known symbols
    let test_cases = vec![
        ("float_types", "test_symbols", "", "_RNvCs5GYaaS9NRMV_12test_symbols11float_types"),
        ("integer_types", "test_symbols", "", "_RNvCs5GYaaS9NRMV_12test_symbols13integer_types"),
    ];

    for (func_name, crate_name, module, expected) in test_cases {
        let module_path = if module.is_empty() {
            crate_name
        } else {
            module
        };

        let generated = mangle_function(crate_name, module_path, func_name, crate_hash);

        assert_eq!(
            generated, expected,
            "Function {} generated incorrect symbol.\nExpected: {}\nGot:      {}",
            func_name, expected, generated
        );

        // Verify it can be demangled
        assert!(
            rustc_demangle::try_demangle(&generated).is_ok(),
            "Generated symbol {} cannot be demangled",
            generated
        );
    }
}
