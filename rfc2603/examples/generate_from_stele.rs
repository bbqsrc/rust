//! Example: Generate v0 mangled symbols from facet-stele exported shapes
//!
//! This demonstrates generating complete RFC 2603 v0 symbols from shapes
//! loaded from a compiled library via facet-stele.

use dlopen2::wrapper::{Container, WrapperApi};
use rfc2603::{push_ident, push_integer_62};
use stele_inventory::ExportedItem;
use std::collections::HashMap;

#[derive(WrapperApi)]
struct SteleApi {
    stele_get_exports: unsafe fn() -> &'static [ExportedItem],
}

/// Generate a v0 mangled symbol for a function
fn mangle_function(crate_name: &str, module_path: &str, function_name: &str, crate_hash: Option<&str>) -> String {
    let mut out = String::from("_R"); // v0 prefix

    // Nv = value namespace (function)
    out.push_str("Nv");

    // Crate root - Cs<hash>_<len><name> (with hash) or C<len><name> (without)
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

    // Module path segments (if any)
    if !module_path.is_empty() && module_path != crate_name {
        for segment in module_path.split("::").skip(1) {
            out.push_str("Nt"); // Type namespace for module
            push_ident(segment, &mut out);
        }
    }

    // Function name
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
    let mut out = String::from("_R"); // v0 prefix
    let start_offset = 2;

    // Nv = value namespace (method is a value)
    out.push_str("Nv");

    // M = inherent impl with disambiguator
    out.push('M');
    out.push('s');
    out.push('a'); // disambiguator 'a' (base62 for 0)
    out.push('_');

    // Record position for backref (after Msa_)
    let impl_path_pos = out.len();

    // Crate root - Cs<hash>_<len><name> (with hash) or C<len><name> (without)
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

    // Module path segments (if any)
    if !module_path.is_empty() && module_path != crate_name {
        for segment in module_path.split("::").skip(1) {
            out.push_str("Nt"); // Type namespace for module
            push_ident(segment, &mut out);
        }
    }

    // Type: Nt + backref to impl path + type name
    out.push_str("Nt");

    // Backref: B + base62(offset from start)
    let offset = impl_path_pos - start_offset;
    out.push('B');
    push_integer_62(offset as u64, &mut out);

    // Type name
    push_ident(type_name, &mut out);

    // Method name
    push_ident(method_name, &mut out);

    out
}

/// Generate a v0 mangled symbol for a struct/enum
fn mangle_type(crate_name: &str, module_path: &str, type_name: &str, crate_hash: Option<&str>) -> String {
    let mut out = String::from("_R"); // v0 prefix

    // No Nv/Nt prefix for the outermost type - it's implied

    // Crate root - Cs<hash>_<len><name> (with hash) or C<len><name> (without)
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

    // Module path segments (if any)
    if !module_path.is_empty() && module_path != crate_name {
        for segment in module_path.split("::").skip(1) {
            out.push_str("Nt"); // Type namespace for module
            push_ident(segment, &mut out);
        }
    }

    // Type name
    push_ident(type_name, &mut out);

    out
}

fn main() {
    let lib_path = std::env::args()
        .nth(1)
        .expect("Usage: generate_from_stele <path-to-libtest_symbols.so>");

    println!("Loading library: {}", lib_path);

    let container: Container<SteleApi> =
        unsafe { Container::load(&lib_path).expect("Failed to load library") };

    let exports = unsafe { container.stele_get_exports() };

    println!("\nFound {} exported items\n", exports.len());
    println!("Generating v0 mangled symbols:\n");

    // Crate hash for test_symbols (extracted from nm output)
    // TODO: Extract this from library metadata or compute it
    let crate_hash = Some("5GYaaS9NRMV");

    for item in exports.iter() {
        match item {
            ExportedItem::Struct(s) => {
                let crate_name = s.module_path.split("::").next().unwrap_or("unknown");
                let symbol = mangle_type(crate_name, s.module_path, s.name, crate_hash);
                println!("Struct: {} ({})", s.name, s.module_path);
                println!("  Symbol: {}", symbol);
                println!();
            }
            ExportedItem::Enum(e) => {
                let crate_name = e.module_path.split("::").next().unwrap_or("unknown");
                let symbol = mangle_type(crate_name, e.module_path, e.name, crate_hash);
                println!("Enum: {} ({})", e.name, e.module_path);
                println!("  Symbol: {}", symbol);
                println!();
            }
            ExportedItem::Function(f) => {
                let crate_name = f.module_path.split("::").next().unwrap_or("unknown");
                let symbol = mangle_function(crate_name, f.module_path, f.name, crate_hash);
                println!("Function: {} ({})", f.name, f.module_path);
                println!("  Symbol: {}", symbol);
                println!();
            }
            ExportedItem::Method(m) => {
                let crate_name = m.module_path.split("::").next().unwrap_or("unknown");
                let symbol = mangle_method(crate_name, m.module_path, m.receiver_type, m.name, crate_hash);
                println!("Method: {}::{} ({})", m.receiver_type, m.name, m.module_path);
                println!("  Symbol: {}", symbol);
                println!();
            }
        }
    }
}
