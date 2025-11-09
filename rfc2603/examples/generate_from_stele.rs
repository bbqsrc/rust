//! Example: Generate v0 mangled symbols from facet-stele exported shapes
//!
//! This demonstrates using rustc_port's V0SymbolMangler with actual shapes
//! loaded from a compiled library via facet-stele.

use dlopen2::wrapper::{Container, WrapperApi};
use rfc2603::rustc_port::{V0SymbolMangler, DefId, GenericArg};
use stele_inventory::ExportedItem;

#[derive(WrapperApi)]
struct SteleApi {
    stele_get_exports: unsafe fn() -> &'static [ExportedItem],
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

    for item in exports.iter() {
        match item {
            ExportedItem::Struct(s) => {
                println!("Struct: {} ({})", s.name, s.module_path);
                println!("  Shape ID: {:?}", s.shape.id);
                println!("  Type: {:?}", s.shape.ty);

                // Try to generate symbol using V0SymbolMangler
                let mut mangler = V0SymbolMangler::new();
                match mangler.print_type(s.shape) {
                    Ok(()) => println!("  Mangled type: {}", mangler.out),
                    Err(e) => println!("  Error mangling: {:?}", e),
                }
                println!();
            }
            ExportedItem::Enum(e) => {
                println!("Enum: {} ({})", e.name, e.module_path);
                println!("  Shape ID: {:?}", e.shape.id);
                println!("  Type: {:?}", e.shape.ty);
                println!();
            }
            ExportedItem::Function(f) => {
                println!("Function: {} ({})", f.name, f.module_path);
                println!("  Parameters:");
                for param in f.parameters {
                    println!("    {}: {:?}", param.name, param.shape.ty);

                    // Try to mangle each parameter type
                    let mut mangler = V0SymbolMangler::new();
                    match mangler.print_type(param.shape) {
                        Ok(()) => println!("      Mangled: {}", mangler.out),
                        Err(e) => println!("      Error: {:?}", e),
                    }
                }
                println!("  Return type: {:?}", f.return_type.ty);

                let mut mangler = V0SymbolMangler::new();
                match mangler.print_type(f.return_type) {
                    Ok(()) => println!("    Mangled: {}", mangler.out),
                    Err(e) => println!("    Error: {:?}", e),
                }
                println!();
            }
            ExportedItem::Method(m) => {
                println!("Method: {}::{} ({})", m.receiver_type, m.name, m.module_path);
                println!("  Parameters:");
                for param in m.parameters {
                    println!("    {}: {:?}", param.name, param.shape.ty);
                }
                println!("  Return type: {:?}", m.return_type.ty);
                println!();
            }
        }
    }
}
