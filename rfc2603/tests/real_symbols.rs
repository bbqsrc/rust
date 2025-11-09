//! Tests using real symbol names from compiled Rust code
//!
//! These symbols were extracted from the test-symbols crate to verify
//! that our mangling implementation produces components that match real symbols.

use rfc2603::{encode_integer_62, push_disambiguator, push_ident};

#[test]
fn test_real_symbol_components() {
    // Test crate name encoding: "test_symbols" (12 chars)
    let mut output = String::new();
    push_ident("test_symbols", &mut output);
    assert_eq!(output, "12test_symbols");

    // Test nested module: "inner"
    let mut output = String::new();
    push_ident("inner", &mut output);
    assert_eq!(output, "5inner");

    // Test further nested: "nested"
    let mut output = String::new();
    push_ident("nested", &mut output);
    assert_eq!(output, "6nested");
}

#[test]
fn test_unicode_function_names() {
    // café
    let mut output = String::new();
    push_ident("café", &mut output);
    assert!(output.starts_with("u")); // Should be Punycode encoded

    // Test Japanese characters: 日本語
    let mut output = String::new();
    push_ident("日本語", &mut output);
    assert!(output.starts_with("u"));

    // Test Greek: Ελληνικά
    let mut output = String::new();
    push_ident("Ελληνικά", &mut output);
    assert!(output.starts_with("u"));
}

#[test]
fn test_type_struct_names() {
    let mut output = String::new();
    push_ident("SimpleStruct", &mut output);
    assert_eq!(output, "12SimpleStruct");

    let mut output = String::new();
    push_ident("GenericStruct", &mut output);
    assert_eq!(output, "13GenericStruct");

    let mut output = String::new();
    push_ident("InnerStruct", &mut output);
    assert_eq!(output, "11InnerStruct");
}

#[test]
fn test_method_names() {
    let mut output = String::new();
    push_ident("new", &mut output);
    assert_eq!(output, "3new");

    let mut output = String::new();
    push_ident("method", &mut output);
    assert_eq!(output, "6method");

    let mut output = String::new();
    push_ident("trait_method", &mut output);
    assert_eq!(output, "12trait_method");

    let mut output = String::new();
    push_ident("inner_method", &mut output);
    assert_eq!(output, "12inner_method");
}

#[test]
fn test_function_names() {
    let names = vec![
        ("integer_types", "13integer_types"),
        ("float_types", "11float_types"),
        ("ref_function", "12ref_function"),
        ("ptr_function", "12ptr_function"),
        ("array_function", "14array_function"),
        ("slice_function", "14slice_function"),
        ("tuple_function", "14tuple_function"),
        ("const_generic", "13const_generic"),
        ("multi_generic", "13multi_generic"),
        ("generic_function", "16generic_function"),
        ("instantiate_generics", "20instantiate_generics"),
    ];

    for (name, expected) in names {
        let mut output = String::new();
        push_ident(name, &mut output);
        assert_eq!(output, expected, "Failed for function: {}", name);
    }
}

#[test]
fn test_base62_for_hash_values() {
    // Test some base-62 encodings that might appear in hashes
    // Encoding: x=0 -> "_", x>0 -> base62(x-1) + "_"
    assert_eq!(encode_integer_62(0), "_");
    assert_eq!(encode_integer_62(1), "0_");  // 0 in base62
    assert_eq!(encode_integer_62(10), "9_"); // 9 in base62
    assert_eq!(encode_integer_62(11), "a_"); // 10 in base62
    assert_eq!(encode_integer_62(36), "z_"); // 35 in base62
    assert_eq!(encode_integer_62(37), "A_"); // 36 in base62
    assert_eq!(encode_integer_62(62), "Z_"); // 61 in base62
    assert_eq!(encode_integer_62(63), "10_"); // 62 in base62
}

#[test]
fn test_v0_symbol_structure() {
    // Build a simple v0 symbol manually: _R + crate + function
    let mut symbol = String::from("_R");

    // Crate root tag
    symbol.push('C');

    // Disambiguator (0 means no disambiguator needed)
    push_disambiguator(0, &mut symbol);

    // Crate name
    push_ident("mycrate", &mut symbol);

    // Nested path tag
    symbol.push('N');

    // Value namespace
    symbol.push('v');

    // Parent path (backref to crate)
    symbol.push('C');
    push_disambiguator(0, &mut symbol);
    push_ident("mycrate", &mut symbol);

    // Function name
    push_ident("foo", &mut symbol);

    // Should be: _RC7mycrateNvC7mycrate3foo
    assert_eq!(symbol, "_RC7mycrateNvC7mycrate3foo");
}

#[test]
fn test_nested_module_symbol() {
    // Build: _R + crate + module + function
    let mut symbol = String::from("_R");

    symbol.push('C');
    push_ident("test", &mut symbol);

    symbol.push('N');
    symbol.push('v'); // value namespace

    // Module path
    symbol.push('N');
    symbol.push('t'); // type namespace for module
    symbol.push('C');
    push_ident("test", &mut symbol);
    push_ident("inner", &mut symbol);

    // Function name
    push_ident("func", &mut symbol);

    // _RC4testNvNtC4test5inner4func
    assert_eq!(symbol, "_RC4testNvNtC4test5inner4func");
}
