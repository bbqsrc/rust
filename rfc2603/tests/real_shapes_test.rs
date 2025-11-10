//! Tests using REAL facet shapes, not synthetic TypeArg enums
//!
//! This tests the actual V0SymbolMangler with real facet Shape instances

use rfc2603::rustc_port::V0SymbolMangler;
use facet::Facet;

#[test]
fn test_real_shape_primitives() {
    let mut mangler = V0SymbolMangler::new();

    // Test bool
    mangler.out.push_str("I"); // Start instantiation
    mangler.print_type(<bool as Facet>::SHAPE).unwrap();
    assert!(mangler.out.contains("b"), "bool should encode as 'b'");
    println!("✓ Real shape bool: {}", mangler.out);

    // Test u32
    let mut mangler = V0SymbolMangler::new();
    mangler.out.push_str("I");
    mangler.print_type(<u32 as Facet>::SHAPE).unwrap();
    assert!(mangler.out.contains("m"), "u32 should encode as 'm'");
    println!("✓ Real shape u32: {}", mangler.out);

    // Test i64
    let mut mangler = V0SymbolMangler::new();
    mangler.out.push_str("I");
    mangler.print_type(<i64 as Facet>::SHAPE).unwrap();
    assert!(mangler.out.contains("x"), "i64 should encode as 'x'");
    println!("✓ Real shape i64: {}", mangler.out);

    // Test f32
    let mut mangler = V0SymbolMangler::new();
    mangler.out.push_str("I");
    mangler.print_type(<f32 as Facet>::SHAPE).unwrap();
    assert!(mangler.out.contains("f"), "f32 should encode as 'f'");
    println!("✓ Real shape f32: {}", mangler.out);
}

#[test]
fn test_real_shape_references() {
    // Test &u32
    let mut mangler = V0SymbolMangler::new();
    mangler.out.push_str("I");
    mangler.print_type(<&u32 as Facet>::SHAPE).unwrap();
    println!("✓ Real shape &u32: {}", mangler.out);
    assert!(mangler.out.contains("R"), "Should have reference marker");
    assert!(mangler.out.contains("m"), "Should have u32");

    // Test &mut u32
    let mut mangler = V0SymbolMangler::new();
    mangler.out.push_str("I");
    mangler.print_type(<&mut u32 as Facet>::SHAPE).unwrap();
    println!("✓ Real shape &mut u32: {}", mangler.out);
    assert!(mangler.out.contains("Q"), "Should have mutable reference marker");
    assert!(mangler.out.contains("m"), "Should have u32");
}

#[test]
fn test_real_shape_raw_pointers() {
    // Test *const u32
    let mut mangler = V0SymbolMangler::new();
    mangler.out.push_str("I");
    mangler.print_type(<*const u32 as Facet>::SHAPE).unwrap();
    println!("✓ Real shape *const u32: {}", mangler.out);
    assert!(mangler.out.contains("P"), "Should have const pointer marker");
    assert!(mangler.out.contains("m"), "Should have u32");

    // Test *mut u32
    let mut mangler = V0SymbolMangler::new();
    mangler.out.push_str("I");
    mangler.print_type(<*mut u32 as Facet>::SHAPE).unwrap();
    println!("✓ Real shape *mut u32: {}", mangler.out);
    assert!(mangler.out.contains("O"), "Should have mut pointer marker");
    assert!(mangler.out.contains("m"), "Should have u32");
}

#[test]
fn test_real_shape_arrays() {
    // Test [u32; 10]
    let mut mangler = V0SymbolMangler::new();
    mangler.out.push_str("I");
    mangler.print_type(<[u32; 10] as Facet>::SHAPE).unwrap();
    println!("✓ Real shape [u32; 10]: {}", mangler.out);
    assert!(mangler.out.contains("A"), "Should have array marker");
    assert!(mangler.out.contains("m"), "Should have u32");
    assert!(mangler.out.contains("K"), "Should have const marker for length");
}

#[test]
fn test_real_shape_slices() {
    // Test [u32]
    let mut mangler = V0SymbolMangler::new();
    mangler.out.push_str("I");
    mangler.print_type(<[u32] as Facet>::SHAPE).unwrap();
    println!("✓ Real shape [u32]: {}", mangler.out);
    assert!(mangler.out.contains("S"), "Should have slice marker");
    assert!(mangler.out.contains("m"), "Should have u32");
}

#[test]
fn test_real_shape_tuples() {
    // Test (u32, i64)
    let mut mangler = V0SymbolMangler::new();
    mangler.out.push_str("I");
    mangler.print_type(<(u32, i64) as Facet>::SHAPE).unwrap();
    println!("✓ Real shape (u32, i64): {}", mangler.out);
    assert!(mangler.out.contains("T"), "Should have tuple marker");
    assert!(mangler.out.contains("m"), "Should have u32");
    assert!(mangler.out.contains("x"), "Should have i64");
    assert!(mangler.out.contains("E"), "Should have tuple end marker");

    // Test (bool, f32, u8)
    let mut mangler = V0SymbolMangler::new();
    mangler.out.push_str("I");
    mangler.print_type(<(bool, f32, u8) as Facet>::SHAPE).unwrap();
    println!("✓ Real shape (bool, f32, u8): {}", mangler.out);
    assert!(mangler.out.contains("T"), "Should have tuple marker");
    assert!(mangler.out.contains("b"), "Should have bool");
    assert!(mangler.out.contains("f"), "Should have f32");
    assert!(mangler.out.contains("h"), "Should have u8");
}

#[test]
fn test_real_shape_nested_references() {
    // Test &&u32
    let mut mangler = V0SymbolMangler::new();
    mangler.out.push_str("I");
    mangler.print_type(<&&u32 as Facet>::SHAPE).unwrap();
    println!("✓ Real shape &&u32: {}", mangler.out);
    // Should have two R markers
    let r_count = mangler.out.matches('R').count();
    assert!(r_count >= 2, "Should have at least 2 reference markers, got {}", r_count);
    assert!(mangler.out.contains("m"), "Should have u32");
}

#[test]
fn test_real_shape_reference_to_slice() {
    // Test &[u32]
    let mut mangler = V0SymbolMangler::new();
    mangler.out.push_str("I");
    mangler.print_type(<&[u32] as Facet>::SHAPE).unwrap();
    println!("✓ Real shape &[u32]: {}", mangler.out);
    assert!(mangler.out.contains("R"), "Should have reference marker");
    assert!(mangler.out.contains("S"), "Should have slice marker");
    assert!(mangler.out.contains("m"), "Should have u32");
}

#[test]
fn test_real_shape_array_of_arrays() {
    // Test [[u32; 4]; 8]
    let mut mangler = V0SymbolMangler::new();
    mangler.out.push_str("I");
    mangler.print_type(<[[u32; 4]; 8] as Facet>::SHAPE).unwrap();
    println!("✓ Real shape [[u32; 4]; 8]: {}", mangler.out);
    let a_count = mangler.out.matches('A').count();
    assert!(a_count >= 2, "Should have at least 2 array markers, got {}", a_count);
    assert!(mangler.out.contains("m"), "Should have u32");
}

#[test]
fn test_real_shape_complex_tuple() {
    // Test (u32, &str, *const bool)
    let mut mangler = V0SymbolMangler::new();
    mangler.out.push_str("I");
    mangler.print_type(<(u32, &str, *const bool) as Facet>::SHAPE).unwrap();
    println!("✓ Real shape (u32, &str, *const bool): {}", mangler.out);
    assert!(mangler.out.contains("T"), "Should have tuple marker");
    assert!(mangler.out.contains("m"), "Should have u32");
    assert!(mangler.out.contains("e"), "Should have str");
    assert!(mangler.out.contains("b"), "Should have bool");
    assert!(mangler.out.contains("R"), "Should have reference marker");
    assert!(mangler.out.contains("P"), "Should have const pointer marker");
}

#[test]
fn test_real_shape_all_integer_types() {
    let types = vec![
        (<i8 as Facet>::SHAPE, "a", "i8"),
        (<i16 as Facet>::SHAPE, "s", "i16"),
        (<i32 as Facet>::SHAPE, "l", "i32"),
        (<i64 as Facet>::SHAPE, "x", "i64"),
        (<i128 as Facet>::SHAPE, "n", "i128"),
        (<u8 as Facet>::SHAPE, "h", "u8"),
        (<u16 as Facet>::SHAPE, "t", "u16"),
        (<u32 as Facet>::SHAPE, "m", "u32"),
        (<u64 as Facet>::SHAPE, "y", "u64"),
        (<u128 as Facet>::SHAPE, "o", "u128"),
    ];

    for (shape, expected_tag, name) in types {
        let mut mangler = V0SymbolMangler::new();
        mangler.out.push_str("I");
        mangler.print_type(shape).unwrap();
        assert!(mangler.out.contains(expected_tag), "{} should encode as '{}'", name, expected_tag);
        println!("✓ Real shape {}: {}", name, mangler.out);
    }
}

#[test]
fn test_real_shape_unit_type() {
    // Test ()
    let mut mangler = V0SymbolMangler::new();
    mangler.out.push_str("I");
    mangler.print_type(<() as Facet>::SHAPE).unwrap();
    println!("✓ Real shape (): {}", mangler.out);
    assert!(mangler.out.contains("u"), "Unit type should encode as 'u'");
}

#[test]
fn test_real_shape_str_type() {
    // Test str (as a type, not &str)
    let mut mangler = V0SymbolMangler::new();
    mangler.out.push_str("I");
    mangler.print_type(<str as Facet>::SHAPE).unwrap();
    println!("✓ Real shape str: {}", mangler.out);
    assert!(mangler.out.contains("e"), "str should encode as 'e'");
}

#[test]
fn test_real_shape_reference_to_array() {
    // Test &[u32; 5]
    let mut mangler = V0SymbolMangler::new();
    mangler.out.push_str("I");
    mangler.print_type(<&[u32; 5] as Facet>::SHAPE).unwrap();
    println!("✓ Real shape &[u32; 5]: {}", mangler.out);
    assert!(mangler.out.contains("R"), "Should have reference marker");
    assert!(mangler.out.contains("A"), "Should have array marker");
    assert!(mangler.out.contains("m"), "Should have u32");
}

#[test]
fn test_real_shape_pointer_to_slice() {
    // Test *const [u8]
    let mut mangler = V0SymbolMangler::new();
    mangler.out.push_str("I");
    mangler.print_type(<*const [u8] as Facet>::SHAPE).unwrap();
    println!("✓ Real shape *const [u8]: {}", mangler.out);
    assert!(mangler.out.contains("P"), "Should have const pointer marker");
    assert!(mangler.out.contains("S"), "Should have slice marker");
    assert!(mangler.out.contains("h"), "Should have u8");
}

#[test]
fn test_real_shape_tuple_of_references() {
    // Test (&u32, &mut i64, &bool)
    let mut mangler = V0SymbolMangler::new();
    mangler.out.push_str("I");
    mangler.print_type(<(&u32, &mut i64, &bool) as Facet>::SHAPE).unwrap();
    println!("✓ Real shape (&u32, &mut i64, &bool): {}", mangler.out);
    assert!(mangler.out.contains("T"), "Should have tuple marker");
    assert!(mangler.out.contains("R"), "Should have immutable reference marker");
    assert!(mangler.out.contains("Q"), "Should have mutable reference marker");
    assert!(mangler.out.contains("m"), "Should have u32");
    assert!(mangler.out.contains("x"), "Should have i64");
    assert!(mangler.out.contains("b"), "Should have bool");
}
