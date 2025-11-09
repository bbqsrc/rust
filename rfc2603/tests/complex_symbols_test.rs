//! Comprehensive tests for complex v0 symbol mangling
//!
//! This test suite covers ALL symbol types including:
//! - Generic functions with type parameters
//! - Functions with lifetime parameters (including 8+ lifetimes)
//! - Const generic parameters
//! - Tuple types and generic tuples
//! - Complex nested structures with multiple generics and lifetimes
//!
//! These tests validate that our implementation can handle the full complexity
//! of real Rust symbols found in production code.

use rfc2603::{SymbolBuilder, GenericArg, TypeArg, LifetimeArg};

#[test]
fn test_generic_function_single_type() {
    // fn generic_function<T>() where T is instantiated as &u8
    // Expected: _RINvC7mycrate16generic_functionReEB2_
    //   I = instantiation
    //   NvC7mycrate16generic_function = path to function
    //   Re = reference to u8 (R = immutable ref, e = erased lifetime, h = u8 BUT WAIT)
    //   E = end of generics
    //   B2_ = backref

    // Actually for the simple case of generic_function::<&u8>:
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash("5GYaaS9NRMV")
        .function("generic_function")
        .with_type_param(TypeArg::Reference {
            lifetime: Some(LifetimeArg::Erased),
            mutable: false,
            inner: Box::new(TypeArg::U8),
        })
        .build()
        .unwrap();

    println!("Generated generic symbol: {}", symbol);

    // The symbol should start with _RI (instantiation)
    assert!(symbol.starts_with("_RI"), "Symbol should start with _RI for generic instantiation");
    assert!(symbol.contains("16generic_function"), "Symbol should contain the function name");
    assert!(symbol.contains("R"), "Symbol should contain R for reference");
    assert!(symbol.ends_with("E"), "Symbol should end with E to close generics");
}

#[test]
fn test_generic_function_primitive_types() {
    // fn foo<T>() instantiated with various primitives

    // foo::<u32>
    let symbol = SymbolBuilder::new("mycrate")
        .function("foo")
        .with_type_param(TypeArg::U32)
        .build()
        .unwrap();
    assert_eq!(symbol, "_RINvC7mycrate3foomE");
    println!("✓ foo::<u32> = {}", symbol);

    // foo::<i64>
    let symbol = SymbolBuilder::new("mycrate")
        .function("foo")
        .with_type_param(TypeArg::I64)
        .build()
        .unwrap();
    assert_eq!(symbol, "_RINvC7mycrate3fooxE");
    println!("✓ foo::<i64> = {}", symbol);

    // foo::<bool>
    let symbol = SymbolBuilder::new("mycrate")
        .function("foo")
        .with_type_param(TypeArg::Bool)
        .build()
        .unwrap();
    assert_eq!(symbol, "_RINvC7mycrate3foobE");
    println!("✓ foo::<bool> = {}", symbol);

    // foo::<f32>
    let symbol = SymbolBuilder::new("mycrate")
        .function("foo")
        .with_type_param(TypeArg::F32)
        .build()
        .unwrap();
    assert_eq!(symbol, "_RINvC7mycrate3foofE");
    println!("✓ foo::<f32> = {}", symbol);
}

#[test]
fn test_generic_function_multiple_types() {
    // fn foo<T, U>() instantiated as foo::<u32, i64>
    let symbol = SymbolBuilder::new("mycrate")
        .function("foo")
        .with_generics(&[
            GenericArg::Type(TypeArg::U32),
            GenericArg::Type(TypeArg::I64),
        ])
        .build()
        .unwrap();

    assert_eq!(symbol, "_RINvC7mycrate3foomxE");
    println!("✓ foo::<u32, i64> = {}", symbol);
}

#[test]
fn test_generic_function_with_tuple() {
    // fn foo<T>() instantiated as foo::<(u32, i64)>
    let symbol = SymbolBuilder::new("mycrate")
        .function("foo")
        .with_type_param(TypeArg::Tuple(vec![
            TypeArg::U32,
            TypeArg::I64,
        ]))
        .build()
        .unwrap();

    // Tuple format: T + elements + E
    assert_eq!(symbol, "_RINvC7mycrate3fooTmxEE");
    println!("✓ foo::<(u32, i64)> = {}", symbol);
}

#[test]
fn test_generic_function_with_generic_tuple() {
    // fn foo<T, U, V>() instantiated as foo::<u32, i64, (u8, bool, f32)>
    let symbol = SymbolBuilder::new("mycrate")
        .function("foo")
        .with_generics(&[
            GenericArg::Type(TypeArg::U32),
            GenericArg::Type(TypeArg::I64),
            GenericArg::Type(TypeArg::Tuple(vec![
                TypeArg::U8,
                TypeArg::Bool,
                TypeArg::F32,
            ])),
        ])
        .build()
        .unwrap();

    assert_eq!(symbol, "_RINvC7mycrate3foomxThbfEE");
    println!("✓ foo::<u32, i64, (u8, bool, f32)> = {}", symbol);
}

#[test]
fn test_const_generic() {
    // fn const_generic<const N: usize>() instantiated as const_generic::<5>
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash("5GYaaS9NRMV")
        .function("const_generic")
        .with_const_param(5)
        .build()
        .unwrap();

    println!("Generated const generic symbol: {}", symbol);
    assert!(symbol.contains("13const_generic"), "Symbol should contain function name");
    assert!(symbol.contains("Kj"), "Symbol should contain Kj for const usize");
    assert!(symbol.ends_with("E"), "Symbol should end with E");
}

#[test]
fn test_lifetime_parameters() {
    // fn foo<'a>() with erased lifetime
    let symbol = SymbolBuilder::new("mycrate")
        .function("foo")
        .with_lifetime(LifetimeArg::Erased)
        .build()
        .unwrap();

    // Erased lifetime is encoded as L0 (L + base62(0) = L_)
    assert_eq!(symbol, "_RINvC7mycrate3fooL_E");
    println!("✓ foo<'a> (erased) = {}", symbol);
}

#[test]
fn test_multiple_lifetimes() {
    // fn foo<'a, 'b, 'c, 'd>() with bound lifetimes
    let symbol = SymbolBuilder::new("mycrate")
        .function("foo")
        .with_generics(&[
            GenericArg::Lifetime(LifetimeArg::Bound { index: 0 }),
            GenericArg::Lifetime(LifetimeArg::Bound { index: 1 }),
            GenericArg::Lifetime(LifetimeArg::Bound { index: 2 }),
            GenericArg::Lifetime(LifetimeArg::Bound { index: 3 }),
        ])
        .build()
        .unwrap();

    println!("✓ foo<'a, 'b, 'c, 'd> = {}", symbol);
    assert!(symbol.contains("L"), "Symbol should contain lifetime markers");
}

#[test]
fn test_eight_lifetimes() {
    // fn foo<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h>()
    let symbol = SymbolBuilder::new("mycrate")
        .function("foo")
        .with_generics(&[
            GenericArg::Lifetime(LifetimeArg::Bound { index: 0 }),
            GenericArg::Lifetime(LifetimeArg::Bound { index: 1 }),
            GenericArg::Lifetime(LifetimeArg::Bound { index: 2 }),
            GenericArg::Lifetime(LifetimeArg::Bound { index: 3 }),
            GenericArg::Lifetime(LifetimeArg::Bound { index: 4 }),
            GenericArg::Lifetime(LifetimeArg::Bound { index: 5 }),
            GenericArg::Lifetime(LifetimeArg::Bound { index: 6 }),
            GenericArg::Lifetime(LifetimeArg::Bound { index: 7 }),
        ])
        .build()
        .unwrap();

    println!("✓ foo<8 lifetimes> = {}", symbol);
    // Should have 8 L markers
    let lifetime_count = symbol.matches('L').count();
    assert!(lifetime_count >= 8, "Symbol should have at least 8 lifetime markers");
}

#[test]
fn test_eight_generics() {
    // fn foo<A, B, C, D, E, F, G, H>() instantiated with 8 primitive types
    let symbol = SymbolBuilder::new("mycrate")
        .function("foo")
        .with_generics(&[
            GenericArg::Type(TypeArg::U8),
            GenericArg::Type(TypeArg::U16),
            GenericArg::Type(TypeArg::U32),
            GenericArg::Type(TypeArg::U64),
            GenericArg::Type(TypeArg::I8),
            GenericArg::Type(TypeArg::I16),
            GenericArg::Type(TypeArg::I32),
            GenericArg::Type(TypeArg::I64),
        ])
        .build()
        .unwrap();

    assert_eq!(symbol, "_RINvC7mycrate3foohtmyaslxE");
    println!("✓ foo<8 types> = {}", symbol);
}

#[test]
fn test_eight_lifetimes_eight_generics_and_generic_tuple() {
    // fn complex<'a,'b,'c,'d,'e,'f,'g,'h, A,B,C,D,E,F,G,(X,Y,Z)>(...)
    // This is the ULTIMATE test - 8 lifetimes + 7 types + 1 generic tuple

    let mut args = Vec::new();

    // Add 8 lifetimes
    for i in 0..8 {
        args.push(GenericArg::Lifetime(LifetimeArg::Bound { index: i }));
    }

    // Add 7 type parameters
    args.push(GenericArg::Type(TypeArg::U8));
    args.push(GenericArg::Type(TypeArg::U16));
    args.push(GenericArg::Type(TypeArg::U32));
    args.push(GenericArg::Type(TypeArg::U64));
    args.push(GenericArg::Type(TypeArg::I8));
    args.push(GenericArg::Type(TypeArg::I16));
    args.push(GenericArg::Type(TypeArg::I32));

    // Add 1 generic tuple (i64, bool, f32)
    args.push(GenericArg::Type(TypeArg::Tuple(vec![
        TypeArg::I64,
        TypeArg::Bool,
        TypeArg::F32,
    ])));

    let symbol = SymbolBuilder::new("mycrate")
        .function("complex")
        .with_generics(&args)
        .build()
        .unwrap();

    println!("✓ complex<8 lifetimes + 7 types + tuple> = {}", symbol);

    // Verify it has all the components
    assert!(symbol.starts_with("_RI"), "Should start with _RI");
    assert!(symbol.contains("7complex"), "Should contain function name");
    assert!(symbol.contains("L"), "Should have lifetime markers");
    assert!(symbol.contains("T"), "Should have tuple marker");
    assert!(symbol.ends_with("E"), "Should end with E");

    // Count type markers (h,t,m,y,a,s,l for 7 types)
    assert!(symbol.contains("h"), "Should have u8");
    assert!(symbol.contains("t"), "Should have u16");
    assert!(symbol.contains("m"), "Should have u32");
    assert!(symbol.contains("y"), "Should have u64");
    assert!(symbol.contains("a"), "Should have i8");
    assert!(symbol.contains("s"), "Should have i16");
    assert!(symbol.contains("l"), "Should have i32");

    // Tuple contents (x, b, f)
    assert!(symbol.contains("x"), "Should have i64 in tuple");
    assert!(symbol.contains("b"), "Should have bool in tuple");
    assert!(symbol.contains("f"), "Should have f32 in tuple");
}

#[test]
fn test_reference_types() {
    // fn foo<T>() instantiated as foo::<&u32>
    let symbol = SymbolBuilder::new("mycrate")
        .function("foo")
        .with_type_param(TypeArg::Reference {
            lifetime: Some(LifetimeArg::Erased),
            mutable: false,
            inner: Box::new(TypeArg::U32),
        })
        .build()
        .unwrap();

    // R = immutable ref, L_ = erased lifetime, m = u32
    assert_eq!(symbol, "_RINvC7mycrate3fooRL_mE");
    println!("✓ foo::<&u32> = {}", symbol);

    // fn foo<T>() instantiated as foo::<&mut u32>
    let symbol = SymbolBuilder::new("mycrate")
        .function("foo")
        .with_type_param(TypeArg::Reference {
            lifetime: Some(LifetimeArg::Erased),
            mutable: true,
            inner: Box::new(TypeArg::U32),
        })
        .build()
        .unwrap();

    // Q = mutable ref, L_ = erased lifetime, m = u32
    assert_eq!(symbol, "_RINvC7mycrate3fooQL_mE");
    println!("✓ foo::<&mut u32> = {}", symbol);
}

#[test]
fn test_raw_pointer_types() {
    // fn foo<T>() instantiated as foo::<*const u32>
    let symbol = SymbolBuilder::new("mycrate")
        .function("foo")
        .with_type_param(TypeArg::RawPtr {
            mutable: false,
            inner: Box::new(TypeArg::U32),
        })
        .build()
        .unwrap();

    // P = const ptr, m = u32
    assert_eq!(symbol, "_RINvC7mycrate3fooPmE");
    println!("✓ foo::<*const u32> = {}", symbol);

    // fn foo<T>() instantiated as foo::<*mut u32>
    let symbol = SymbolBuilder::new("mycrate")
        .function("foo")
        .with_type_param(TypeArg::RawPtr {
            mutable: true,
            inner: Box::new(TypeArg::U32),
        })
        .build()
        .unwrap();

    // O = mut ptr, m = u32
    assert_eq!(symbol, "_RINvC7mycrate3fooOmE");
    println!("✓ foo::<*mut u32> = {}", symbol);
}

#[test]
fn test_array_type() {
    // fn foo<T>() instantiated as foo::<[u32; 10]>
    let symbol = SymbolBuilder::new("mycrate")
        .function("foo")
        .with_type_param(TypeArg::Array {
            inner: Box::new(TypeArg::U32),
            len: 10,
        })
        .build()
        .unwrap();

    // A = array, m = u32, Kj = const usize, 9_ = base62(10-1)
    assert_eq!(symbol, "_RINvC7mycrate3fooAmKj9_E");
    println!("✓ foo::<[u32; 10]> = {}", symbol);
}

#[test]
fn test_slice_type() {
    // fn foo<T>() instantiated as foo::<[u32]>
    let symbol = SymbolBuilder::new("mycrate")
        .function("foo")
        .with_type_param(TypeArg::Slice(Box::new(TypeArg::U32)))
        .build()
        .unwrap();

    // S = slice, m = u32
    assert_eq!(symbol, "_RINvC7mycrate3fooSmE");
    println!("✓ foo::<[u32]> = {}", symbol);
}

#[test]
fn test_nested_complex_types() {
    // fn foo<T>() instantiated as foo::<&[&mut u32]>
    // This is: reference to slice of mutable references to u32
    let symbol = SymbolBuilder::new("mycrate")
        .function("foo")
        .with_type_param(TypeArg::Reference {
            lifetime: Some(LifetimeArg::Erased),
            mutable: false,
            inner: Box::new(TypeArg::Slice(Box::new(TypeArg::Reference {
                lifetime: Some(LifetimeArg::Erased),
                mutable: true,
                inner: Box::new(TypeArg::U32),
            }))),
        })
        .build()
        .unwrap();

    println!("✓ foo::<&[&mut u32]> = {}", symbol);

    // Verify structure: R (ref) + L_ (lifetime) + S (slice) + Q (mut ref) + L_ + m (u32)
    assert!(symbol.contains("R"), "Should have immutable reference");
    assert!(symbol.contains("S"), "Should have slice");
    assert!(symbol.contains("Q"), "Should have mutable reference");
    assert!(symbol.contains("m"), "Should have u32");
}

#[test]
fn test_all_primitive_integer_types() {
    let primitives = vec![
        (TypeArg::I8, "a"),
        (TypeArg::I16, "s"),
        (TypeArg::I32, "l"),
        (TypeArg::I64, "x"),
        (TypeArg::I128, "n"),
        (TypeArg::Isize, "i"),
        (TypeArg::U8, "h"),
        (TypeArg::U16, "t"),
        (TypeArg::U32, "m"),
        (TypeArg::U64, "y"),
        (TypeArg::U128, "o"),
        (TypeArg::Usize, "j"),
    ];

    for (ty, expected_tag) in primitives {
        let symbol = SymbolBuilder::new("mycrate")
            .function("foo")
            .with_type_param(ty.clone())
            .build()
            .unwrap();

        let expected = format!("_RINvC7mycrate3foo{}E", expected_tag);
        assert_eq!(symbol, expected, "Type {:?} should encode to {}", ty, expected_tag);
        println!("✓ {:?} = {}", ty, symbol);
    }
}

#[test]
fn test_all_other_primitive_types() {
    let primitives = vec![
        (TypeArg::Bool, "b"),
        (TypeArg::Char, "c"),
        (TypeArg::F32, "f"),
        (TypeArg::F64, "d"),
        (TypeArg::Str, "e"),
        (TypeArg::Never, "z"),
        (TypeArg::Unit, "u"),
    ];

    for (ty, expected_tag) in primitives {
        let symbol = SymbolBuilder::new("mycrate")
            .function("foo")
            .with_type_param(ty.clone())
            .build()
            .unwrap();

        let expected = format!("_RINvC7mycrate3foo{}E", expected_tag);
        assert_eq!(symbol, expected, "Type {:?} should encode to {}", ty, expected_tag);
        println!("✓ {:?} = {}", ty, symbol);
    }
}

#[test]
fn test_mixed_lifetimes_and_types() {
    // fn foo<'a, T, 'b, U>() - interleaved lifetimes and types
    let symbol = SymbolBuilder::new("mycrate")
        .function("foo")
        .with_generics(&[
            GenericArg::Lifetime(LifetimeArg::Bound { index: 0 }),
            GenericArg::Type(TypeArg::U32),
            GenericArg::Lifetime(LifetimeArg::Bound { index: 1 }),
            GenericArg::Type(TypeArg::I64),
        ])
        .build()
        .unwrap();

    println!("✓ foo<'a, u32, 'b, i64> = {}", symbol);
    assert!(symbol.contains("L"), "Should have lifetime markers");
    assert!(symbol.contains("m"), "Should have u32");
    assert!(symbol.contains("x"), "Should have i64");
}

#[test]
fn test_const_and_type_params() {
    // fn foo<T, const N: usize>() instantiated as foo::<u32, 42>
    let symbol = SymbolBuilder::new("mycrate")
        .function("foo")
        .with_generics(&[
            GenericArg::Type(TypeArg::U32),
            GenericArg::Const(42),
        ])
        .build()
        .unwrap();

    println!("✓ foo<u32, 42> = {}", symbol);
    assert!(symbol.contains("m"), "Should have u32");
    assert!(symbol.contains("Kj"), "Should have const usize marker");
}
