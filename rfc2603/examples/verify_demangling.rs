//! Verify that rustc-generated v0 symbols can be demangled

use std::process::Command;

fn main() {
    let lib_path = "/home/user/test-symbols/target/debug/libtest_symbols.so";

    println!("=== Extracting and demangling v0 symbols from rustc ===\n");

    let output = Command::new("nm")
        .arg("-g")
        .arg(lib_path)
        .output()
        .expect("Failed to run nm");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut count = 0;

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let symbol = parts[2];

            // Only show non-generic v0 symbols for test_symbols
            if symbol.starts_with("_RNv") && symbol.contains("test_symbols") && !symbol.contains("INv") {
                if let Ok(demangled) = rustc_demangle::try_demangle(symbol) {
                    println!("Symbol: {}", symbol);
                    println!("Demangled: {:#}", demangled);
                    println!();
                    count += 1;

                    if count >= 15 {
                        break;
                    }
                }
            }
        }
    }

    println!("=== Showing path encoding structure ===\n");

    // Demonstrate how nested paths work
    let examples = vec![
        ("_RNvCs5GYaaS9NRMV_12test_symbols11float_types", "Simple function"),
        ("_RNvNtCs5GYaaS9NRMV_12test_symbols5inner14inner_function", "Nested module function"),
        ("_RNvNtNtCs5GYaaS9NRMV_12test_symbols5inner6nested22deeply_nested_function", "Deeply nested"),
        ("_RNvMsa_Cs5GYaaS9NRMV_12test_symbolsNtB5_12SimpleStruct3new", "Method"),
    ];

    for (symbol, desc) in examples {
        println!("{}: {}", desc, symbol);
        if let Ok(demangled) = rustc_demangle::try_demangle(symbol) {
            println!("  Demangled: {:#}", demangled);
        }

        // Break down the structure
        if symbol.starts_with("_RNvNtNt") {
            println!("  Structure: _R + Nv (value) + Nt (nested) + Nt (nested again) + C (crate) + ...");
        } else if symbol.starts_with("_RNvNt") {
            println!("  Structure: _R + Nv (value) + Nt (nested) + C (crate) + ...");
        } else if symbol.starts_with("_RNvM") {
            println!("  Structure: _R + Nv (value) + M (inherent impl) + ...");
        } else if symbol.starts_with("_RNv") {
            println!("  Structure: _R + Nv (value) + C (crate) + ...");
        }
        println!();
    }
}
