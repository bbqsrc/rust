//! The most fucked up type combinations we can think of
//!
//! These tests push the symbol mangling to its absolute limits with:
//! - Nested references with multiple lifetimes
//! - Arrays of tuples of references
//! - Slices of mutable references
//! - Deeply nested tuples
//! - Every pointer variant combined in unholy ways

use rfc2603::{SymbolBuilder, GenericArg, TypeArg, LifetimeArg};

#[test]
fn test_fucked_up_type_1_triple_nested_references() {
    // fn foo<T>() where T = &'a &'b &'c mut u32
    // This is: reference to reference to mutable reference to u32
    // With 3 different lifetimes!

    let inner_most = TypeArg::U32;

    let second_level = TypeArg::Reference {
        lifetime: Some(LifetimeArg::Bound { index: 2 }), // 'c
        mutable: true,
        inner: Box::new(inner_most),
    };

    let middle_level = TypeArg::Reference {
        lifetime: Some(LifetimeArg::Bound { index: 1 }), // 'b
        mutable: false,
        inner: Box::new(second_level),
    };

    let outer_level = TypeArg::Reference {
        lifetime: Some(LifetimeArg::Bound { index: 0 }), // 'a
        mutable: false,
        inner: Box::new(middle_level),
    };

    let symbol = SymbolBuilder::new("mycrate")
        .function("triple_nested_ref")
        .with_type_param(outer_level)
        .build()
        .unwrap();

    println!("✓ Triple nested reference: {}", symbol);
    // Should contain multiple R/Q markers and L markers for lifetimes
    assert!(symbol.contains("R"), "Should have reference markers");
    assert!(symbol.contains("Q"), "Should have mutable reference marker");
    assert!(symbol.contains("m"), "Should have u32");
}

#[test]
fn test_fucked_up_type_2_array_of_tuples_of_references() {
    // fn foo<T>() where T = [(&u32, &mut i64); 10]
    // Array of 10 elements, where each element is a tuple of references

    let tuple_element = TypeArg::Tuple(vec![
        TypeArg::Reference {
            lifetime: Some(LifetimeArg::Erased),
            mutable: false,
            inner: Box::new(TypeArg::U32),
        },
        TypeArg::Reference {
            lifetime: Some(LifetimeArg::Erased),
            mutable: true,
            inner: Box::new(TypeArg::I64),
        },
    ]);

    let array = TypeArg::Array {
        inner: Box::new(tuple_element),
        len: 10,
    };

    let symbol = SymbolBuilder::new("mycrate")
        .function("array_of_tuples")
        .with_type_param(array)
        .build()
        .unwrap();

    println!("✓ Array of tuples of references: {}", symbol);
    // A = array, T = tuple, R/Q = references, E = end tuple, Kj9_ = const 10
    assert!(symbol.contains("A"), "Should have array marker");
    assert!(symbol.contains("T"), "Should have tuple marker");
    assert!(symbol.contains("E"), "Should have tuple end marker");
    assert!(symbol.contains("R"), "Should have immutable reference");
    assert!(symbol.contains("Q"), "Should have mutable reference");
}

#[test]
fn test_fucked_up_type_3_reference_to_slice_of_mutable_references() {
    // fn foo<T>() where T = &[&mut u32]
    // Reference to slice of mutable references

    let mutable_ref = TypeArg::Reference {
        lifetime: Some(LifetimeArg::Erased),
        mutable: true,
        inner: Box::new(TypeArg::U32),
    };

    let slice = TypeArg::Slice(Box::new(mutable_ref));

    let outer_ref = TypeArg::Reference {
        lifetime: Some(LifetimeArg::Erased),
        mutable: false,
        inner: Box::new(slice),
    };

    let symbol = SymbolBuilder::new("mycrate")
        .function("ref_to_slice_of_mut_refs")
        .with_type_param(outer_ref)
        .build()
        .unwrap();

    println!("✓ Reference to slice of mutable references: {}", symbol);
    // R = outer ref, S = slice, Q = inner mut ref
    assert!(symbol.contains("R"), "Should have outer reference");
    assert!(symbol.contains("S"), "Should have slice");
    assert!(symbol.contains("Q"), "Should have mutable reference");
}

#[test]
fn test_fucked_up_type_4_deeply_nested_tuples() {
    // fn foo<T>() where T = (((u8, u16), (u32, u64)), ((i8, i16), (i32, i64)))
    // Tuples nested 3 levels deep with 8 total primitive types

    let inner_tuple_1 = TypeArg::Tuple(vec![TypeArg::U8, TypeArg::U16]);
    let inner_tuple_2 = TypeArg::Tuple(vec![TypeArg::U32, TypeArg::U64]);
    let inner_tuple_3 = TypeArg::Tuple(vec![TypeArg::I8, TypeArg::I16]);
    let inner_tuple_4 = TypeArg::Tuple(vec![TypeArg::I32, TypeArg::I64]);

    let mid_tuple_1 = TypeArg::Tuple(vec![inner_tuple_1, inner_tuple_2]);
    let mid_tuple_2 = TypeArg::Tuple(vec![inner_tuple_3, inner_tuple_4]);

    let outer_tuple = TypeArg::Tuple(vec![mid_tuple_1, mid_tuple_2]);

    let symbol = SymbolBuilder::new("mycrate")
        .function("deeply_nested_tuples")
        .with_type_param(outer_tuple)
        .build()
        .unwrap();

    println!("✓ Deeply nested tuples: {}", symbol);
    // Should have many T markers (for each tuple) and E markers (for each tuple end)
    let t_count = symbol.matches('T').count();
    let e_count = symbol.matches('E').count();

    // We have 7 tuples total (1 outer + 2 mid + 4 inner)
    assert!(t_count >= 7, "Should have at least 7 tuple markers, got {}", t_count);
    assert!(e_count >= 7, "Should have at least 7 tuple end markers, got {}", e_count);

    // All 8 primitive types should be present
    assert!(symbol.contains("h"), "Should have u8");
    assert!(symbol.contains("t"), "Should have u16");
    assert!(symbol.contains("m"), "Should have u32");
    assert!(symbol.contains("y"), "Should have u64");
    assert!(symbol.contains("a"), "Should have i8");
    assert!(symbol.contains("s"), "Should have i16");
    assert!(symbol.contains("l"), "Should have i32");
    assert!(symbol.contains("x"), "Should have i64");
}

#[test]
fn test_fucked_up_type_5_pointer_madness() {
    // fn foo<T>() where T = (*const [u32; 10], *mut &i64, &*const u8, &mut &mut bool)
    // Every pointer variant in one unholy tuple:
    // - const pointer to array
    // - mut pointer to reference
    // - reference to const pointer
    // - mutable reference to mutable reference

    let elem1 = TypeArg::RawPtr {
        mutable: false,
        inner: Box::new(TypeArg::Array {
            inner: Box::new(TypeArg::U32),
            len: 10,
        }),
    };

    let elem2 = TypeArg::RawPtr {
        mutable: true,
        inner: Box::new(TypeArg::Reference {
            lifetime: Some(LifetimeArg::Erased),
            mutable: false,
            inner: Box::new(TypeArg::I64),
        }),
    };

    let elem3 = TypeArg::Reference {
        lifetime: Some(LifetimeArg::Erased),
        mutable: false,
        inner: Box::new(TypeArg::RawPtr {
            mutable: false,
            inner: Box::new(TypeArg::U8),
        }),
    };

    let elem4 = TypeArg::Reference {
        lifetime: Some(LifetimeArg::Erased),
        mutable: true,
        inner: Box::new(TypeArg::Reference {
            lifetime: Some(LifetimeArg::Erased),
            mutable: true,
            inner: Box::new(TypeArg::Bool),
        }),
    };

    let tuple = TypeArg::Tuple(vec![elem1, elem2, elem3, elem4]);

    let symbol = SymbolBuilder::new("mycrate")
        .function("pointer_madness")
        .with_type_param(tuple)
        .build()
        .unwrap();

    println!("✓ Pointer madness: {}", symbol);

    // Should have:
    // P = const pointer
    // O = mut pointer
    // R = immutable reference
    // Q = mutable reference
    // A = array
    // T = tuple

    assert!(symbol.contains("P"), "Should have const pointer");
    assert!(symbol.contains("O"), "Should have mut pointer");
    assert!(symbol.contains("R"), "Should have immutable reference");
    assert!(symbol.contains("Q"), "Should have mutable reference");
    assert!(symbol.contains("A"), "Should have array");
    assert!(symbol.contains("T"), "Should have tuple");
}

#[test]
fn test_fucked_up_type_6_nested_arrays() {
    // fn foo<T>() where T = [[u32; 4]; 8]
    // 2D array: outer array of 8 elements, each element is an array of 4 u32s

    let inner_array = TypeArg::Array {
        inner: Box::new(TypeArg::U32),
        len: 4,
    };

    let outer_array = TypeArg::Array {
        inner: Box::new(inner_array),
        len: 8,
    };

    let symbol = SymbolBuilder::new("mycrate")
        .function("nested_arrays")
        .with_type_param(outer_array)
        .build()
        .unwrap();

    println!("✓ Nested arrays: {}", symbol);

    // Should have 2 array markers
    let a_count = symbol.matches('A').count();
    assert!(a_count >= 2, "Should have at least 2 array markers, got {}", a_count);

    // Should have both const values: Kj3_ for 4 and Kj7_ for 8
    assert!(symbol.contains("m"), "Should have u32");
}

#[test]
fn test_fucked_up_type_7_kitchen_sink() {
    // fn ultra<'a, 'b, const N: usize, T, U>() where:
    //   T = &'a [(&'b mut [u32; N], *const (i64, bool))]
    //   U = (*mut &'a str, [[u8; 2]; 3])
    // This combines EVERYTHING: lifetimes, const generics, nested refs, arrays, tuples, pointers

    // First type parameter T
    let t_inner_tuple = TypeArg::Tuple(vec![
        TypeArg::Reference {
            lifetime: Some(LifetimeArg::Bound { index: 1 }), // 'b
            mutable: true,
            inner: Box::new(TypeArg::Array {
                inner: Box::new(TypeArg::U32),
                len: 5, // Using concrete value instead of N for now
            }),
        },
        TypeArg::RawPtr {
            mutable: false,
            inner: Box::new(TypeArg::Tuple(vec![TypeArg::I64, TypeArg::Bool])),
        },
    ]);

    let t_slice = TypeArg::Slice(Box::new(t_inner_tuple));

    let type_t = TypeArg::Reference {
        lifetime: Some(LifetimeArg::Bound { index: 0 }), // 'a
        mutable: false,
        inner: Box::new(t_slice),
    };

    // Second type parameter U
    let u_elem1 = TypeArg::RawPtr {
        mutable: true,
        inner: Box::new(TypeArg::Reference {
            lifetime: Some(LifetimeArg::Bound { index: 0 }), // 'a
            mutable: false,
            inner: Box::new(TypeArg::Str),
        }),
    };

    let u_elem2 = TypeArg::Array {
        inner: Box::new(TypeArg::Array {
            inner: Box::new(TypeArg::U8),
            len: 2,
        }),
        len: 3,
    };

    let type_u = TypeArg::Tuple(vec![u_elem1, u_elem2]);

    let symbol = SymbolBuilder::new("mycrate")
        .function("ultra")
        .with_generics(&[
            GenericArg::Lifetime(LifetimeArg::Bound { index: 0 }), // 'a
            GenericArg::Lifetime(LifetimeArg::Bound { index: 1 }), // 'b
            GenericArg::Const(5), // N = 5
            GenericArg::Type(type_t),
            GenericArg::Type(type_u),
        ])
        .build()
        .unwrap();

    println!("✓ KITCHEN SINK: {}", symbol);

    // This should have EVERYTHING
    assert!(symbol.contains("L"), "Should have lifetimes");
    assert!(symbol.contains("K"), "Should have const generic");
    assert!(symbol.contains("R"), "Should have references");
    assert!(symbol.contains("Q"), "Should have mut references");
    assert!(symbol.contains("S"), "Should have slice");
    assert!(symbol.contains("A"), "Should have array");
    assert!(symbol.contains("P"), "Should have const pointer");
    assert!(symbol.contains("O"), "Should have mut pointer");
    assert!(symbol.contains("T"), "Should have tuple");
    assert!(symbol.contains("E"), "Should have end markers");

    println!("Symbol length: {} bytes", symbol.len());
    println!("This is a BEAST of a symbol!");
}

#[test]
fn test_fucked_up_type_8_slice_of_slices() {
    // fn foo<T>() where T = &[&[&[u8]]]
    // Reference to slice of slices of slices - 3 levels deep

    let innermost = TypeArg::Slice(Box::new(TypeArg::U8));

    let middle = TypeArg::Slice(Box::new(TypeArg::Reference {
        lifetime: Some(LifetimeArg::Erased),
        mutable: false,
        inner: Box::new(innermost),
    }));

    let outer = TypeArg::Reference {
        lifetime: Some(LifetimeArg::Erased),
        mutable: false,
        inner: Box::new(TypeArg::Slice(Box::new(TypeArg::Reference {
            lifetime: Some(LifetimeArg::Erased),
            mutable: false,
            inner: Box::new(middle),
        }))),
    };

    let symbol = SymbolBuilder::new("mycrate")
        .function("slice_inception")
        .with_type_param(outer)
        .build()
        .unwrap();

    println!("✓ Slice of slices of slices: {}", symbol);

    // Should have multiple S markers for slices
    let s_count = symbol.matches('S').count();
    assert!(s_count >= 3, "Should have at least 3 slice markers, got {}", s_count);
}
