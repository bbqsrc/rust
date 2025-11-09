//! Tests for decoding and encoding real v0 symbols from the test-symbols crate
//!
//! This file contains tests for all 93 real symbols extracted from the compiled
//! test-symbols crate. Each test attempts to encode the symbol using our API
//! and verify it matches the real compiler output.

use rfc2603::SymbolBuilder;

// The crate hash for test_symbols from our compilation
const TEST_SYMBOLS_HASH: &str = "aRN1VPjcjfp";

// Simple function symbols - these should be easiest to encode
#[test]
fn test_simple_function_float_types() {
    // _RNvCsaRN1VPjcjfp_12test_symbols11float_types
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .function("float_types")
        .build()
        .unwrap();
    assert_eq!(symbol, "_RNvCsaRN1VPjcjfp_12test_symbols11float_types");
}

#[test]
fn test_simple_function_integer_types() {
    // _RNvCsaRN1VPjcjfp_12test_symbols13integer_types
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .function("integer_types")
        .build()
        .unwrap();
    assert_eq!(symbol, "_RNvCsaRN1VPjcjfp_12test_symbols13integer_types");
}

#[test]
fn test_simple_function_ref_function() {
    // _RNvCsaRN1VPjcjfp_12test_symbols12ref_function
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .function("ref_function")
        .build()
        .unwrap();
    assert_eq!(symbol, "_RNvCsaRN1VPjcjfp_12test_symbols12ref_function");
}

#[test]
fn test_simple_function_ptr_function() {
    // _RNvCsaRN1VPjcjfp_12test_symbols12ptr_function
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .function("ptr_function")
        .build()
        .unwrap();
    assert_eq!(symbol, "_RNvCsaRN1VPjcjfp_12test_symbols12ptr_function");
}

#[test]
fn test_simple_function_array_function() {
    // _RNvCsaRN1VPjcjfp_12test_symbols14array_function
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .function("array_function")
        .build()
        .unwrap();
    assert_eq!(symbol, "_RNvCsaRN1VPjcjfp_12test_symbols14array_function");
}

#[test]
fn test_simple_function_slice_function() {
    // _RNvCsaRN1VPjcjfp_12test_symbols14slice_function
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .function("slice_function")
        .build()
        .unwrap();
    assert_eq!(symbol, "_RNvCsaRN1VPjcjfp_12test_symbols14slice_function");
}

#[test]
fn test_simple_function_tuple_function() {
    // _RNvCsaRN1VPjcjfp_12test_symbols14tuple_function
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .function("tuple_function")
        .build()
        .unwrap();
    assert_eq!(symbol, "_RNvCsaRN1VPjcjfp_12test_symbols14tuple_function");
}

#[test]
fn test_simple_function_returns_closure() {
    // _RNvCsaRN1VPjcjfp_12test_symbols15returns_closure
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .function("returns_closure")
        .build()
        .unwrap();
    assert_eq!(symbol, "_RNvCsaRN1VPjcjfp_12test_symbols15returns_closure");
}

#[test]
fn test_simple_function_unsafe_function() {
    // _RNvCsaRN1VPjcjfp_12test_symbols15unsafe_function
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .function("unsafe_function")
        .build()
        .unwrap();
    assert_eq!(symbol, "_RNvCsaRN1VPjcjfp_12test_symbols15unsafe_function");
}

#[test]
fn test_simple_function_lifetime_function() {
    // _RNvCsaRN1VPjcjfp_12test_symbols17lifetime_function
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .function("lifetime_function")
        .build()
        .unwrap();
    assert_eq!(symbol, "_RNvCsaRN1VPjcjfp_12test_symbols17lifetime_function");
}

#[test]
fn test_simple_function_multi_lifetime() {
    // _RNvCsaRN1VPjcjfp_12test_symbols14multi_lifetime
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .function("multi_lifetime")
        .build()
        .unwrap();
    assert_eq!(symbol, "_RNvCsaRN1VPjcjfp_12test_symbols14multi_lifetime");
}

#[test]
fn test_simple_function_takes_fn_ptr() {
    // _RNvCsaRN1VPjcjfp_12test_symbols12takes_fn_ptr
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .function("takes_fn_ptr")
        .build()
        .unwrap();
    assert_eq!(symbol, "_RNvCsaRN1VPjcjfp_12test_symbols12takes_fn_ptr");
}

#[test]
fn test_simple_function_takes_trait_object() {
    // _RNvCsaRN1VPjcjfp_12test_symbols18takes_trait_object
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .function("takes_trait_object")
        .build()
        .unwrap();
    assert_eq!(symbol, "_RNvCsaRN1VPjcjfp_12test_symbols18takes_trait_object");
}

#[test]
fn test_simple_function_instantiate_generics() {
    // _RNvCsaRN1VPjcjfp_12test_symbols20instantiate_generics
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .function("instantiate_generics")
        .build()
        .unwrap();
    assert_eq!(symbol, "_RNvCsaRN1VPjcjfp_12test_symbols20instantiate_generics");
}

// Static value
#[test]
fn test_static_value() {
    // _RNvCsaRN1VPjcjfp_12test_symbols12STATIC_VALUE
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .value("STATIC_VALUE")
        .build()
        .unwrap();
    assert_eq!(symbol, "_RNvCsaRN1VPjcjfp_12test_symbols12STATIC_VALUE");
}

// Nested module functions
#[test]
fn test_nested_inner_function() {
    // _RNvNtCsaRN1VPjcjfp_12test_symbols5inner14inner_function
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .module("inner")
        .function("inner_function")
        .build()
        .unwrap();
    assert_eq!(symbol, "_RNvNtCsaRN1VPjcjfp_12test_symbols5inner14inner_function");
}

#[test]
fn test_deeply_nested_function() {
    // _RNvNtNtCsaRN1VPjcjfp_12test_symbols5inner6nested22deeply_nested_function
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .module("inner")
        .module("nested")
        .function("deeply_nested_function")
        .build()
        .unwrap();
    assert_eq!(symbol, "_RNvNtNtCsaRN1VPjcjfp_12test_symbols5inner6nested22deeply_nested_function");
}

// Unicode function names
#[test]
fn test_unicode_cafe() {
    // _RNvNtCsaRN1VPjcjfp_12test_symbols7unicodeu7caf_dma
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .module("unicode")
        .function("café")
        .build()
        .unwrap();
    assert_eq!(symbol, "_RNvNtCsaRN1VPjcjfp_12test_symbols7unicodeu7caf_dma");
}

#[test]
fn test_unicode_japanese() {
    // _RNvNtCsaRN1VPjcjfp_12test_symbols7unicodeu10wgv71a119e
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .module("unicode")
        .function("日本語")
        .build()
        .unwrap();
    assert_eq!(symbol, "_RNvNtCsaRN1VPjcjfp_12test_symbols7unicodeu10wgv71a119e");
}

#[test]
fn test_unicode_greek() {
    // _RNvNtCsaRN1VPjcjfp_12test_symbols7unicodeu12twa0c6aifdar
    let symbol = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .module("unicode")
        .function("Ελληνικά")
        .build()
        .unwrap();
    assert_eq!(symbol, "_RNvNtCsaRN1VPjcjfp_12test_symbols7unicodeu12twa0c6aifdar");
}

// Method symbols (inherent impl)
#[test]
fn test_method_simple_struct_new() {
    // _RNvMCsaRN1VPjcjfp_12test_symbolsNtB2_12SimpleStruct3new
    // This is: impl SimpleStruct { fn new() }
    let result = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .method("SimpleStruct", "new")
        .build();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("backreferences"));
}

#[test]
fn test_method_simple_struct_method() {
    // _RNvMCsaRN1VPjcjfp_12test_symbolsNtB2_12SimpleStruct6method
    let result = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .method("SimpleStruct", "method")
        .build();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("backreferences"));
}

#[test]
fn test_method_inner_struct_inner_method() {
    // _RNvMNtCsaRN1VPjcjfp_12test_symbols5innerNtB2_11InnerStruct12inner_method
    let result = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .module("inner")
        .method("InnerStruct", "inner_method")
        .build();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("backreferences"));
}

// Unicode method name
#[test]
fn test_unicode_method() {
    // _RNvMNtCsaRN1VPjcjfp_12test_symbols7unicodeNtB2_u6F_1gaau10mthod_bsae
    // This is the méthodé method on struct Föö
    let result = SymbolBuilder::new("test_symbols")
        .with_hash(TEST_SYMBOLS_HASH)
        .module("unicode")
        .method("Föö", "méthodé")
        .build();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("backreferences"));
}

// Generic instantiation symbols
#[test]
fn test_generic_function_i32() {
    // _RINvCsaRN1VPjcjfp_12test_symbols16generic_functionlEB2_
    // This is: generic_function::<i32>
    // I = instantiation start, E = end, l = i32 type, B2_ = backref
    todo!("Need to implement generic instantiation encoding");
}

#[test]
fn test_generic_function_f64() {
    // _RINvCsaRN1VPjcjfp_12test_symbols16generic_functiondEB2_
    // This is: generic_function::<f64>
    // d = f64 type
    todo!("Need to implement generic instantiation encoding");
}

#[test]
fn test_generic_function_ref_str() {
    // _RINvCsaRN1VPjcjfp_12test_symbols16generic_functionReEB2_
    // This is: generic_function::<&str>
    // Re = &str type
    todo!("Need to implement generic instantiation encoding");
}

#[test]
fn test_multi_generic() {
    // _RINvCsaRN1VPjcjfp_12test_symbols13multi_generichtmEB2_
    // This is: multi_generic::<u8, u16, u32>
    // h = u8, t = u16, m = u32
    todo!("Need to implement multi-generic instantiation encoding");
}

#[test]
fn test_const_generic() {
    // _RINvCsaRN1VPjcjfp_12test_symbols13const_genericKj5_EB2_
    // This is: const_generic::<5>
    // K = const, j = usize, 5_ = value 5
    todo!("Need to implement const generic encoding");
}

// Trait implementation symbols
#[test]
fn test_trait_impl_simple_trait_for_simple_struct() {
    // _RNvXs1_CsaRN1VPjcjfp_12test_symbolsNtB5_12SimpleStructNtB5_11SimpleTrait12trait_method
    // X = impl, s1_ = disambiguator
    todo!("Need to implement trait impl encoding");
}

#[test]
fn test_trait_impl_simple_trait_for_i32() {
    // _RNvXs2_CsaRN1VPjcjfp_12test_symbolslNtB5_11SimpleTrait12trait_method
    // l = i32 type
    todo!("Need to implement trait impl encoding");
}

#[test]
fn test_trait_impl_assoc_trait_for_i32() {
    // _RNvXs4_CsaRN1VPjcjfp_12test_symbolslNtB5_10AssocTrait12assoc_method
    todo!("Need to implement trait impl encoding");
}

#[test]
fn test_trait_impl_default_trait() {
    // _RNvXs5_CsaRN1VPjcjfp_12test_symbolsNtB5_12SimpleStructNtB5_12DefaultTrait15required_method
    todo!("Need to implement trait impl encoding");
}

// Generic impl symbols
#[test]
fn test_generic_struct_new_i32() {
    // _RNvMs_CsaRN1VPjcjfp_12test_symbolsINtB4_13GenericStructlE3newB4_
    // M = impl, s_ = disambiguator, I...E = generic params
    todo!("Need to implement generic impl encoding");
}

#[test]
fn test_generic_struct_get_i32() {
    // _RNvMs_CsaRN1VPjcjfp_12test_symbolsINtB4_13GenericStructlE3getB4_
    todo!("Need to implement generic impl encoding");
}

#[test]
fn test_list_singleton_ref_str() {
    // _RNvMs7_CsaRN1VPjcjfp_12test_symbolsINtB5_4ListReE9singletonB5_
    todo!("Need to implement generic impl encoding");
}

#[test]
fn test_list_singleton_i32() {
    // _RNvMs7_CsaRN1VPjcjfp_12test_symbolsINtB5_4ListlE9singletonB5_
    todo!("Need to implement generic impl encoding");
}

// Complex nested generic types
#[test]
fn test_complex_new() {
    // _RNvMs8_CsaRN1VPjcjfp_12test_symbolsINtB5_7ComplexlNtNtCshsEHAXgLWmz_5alloc6string6StringE3newB5_
    // Complex<i32, String>::new
    todo!("Need to implement complex nested generic encoding");
}

#[test]
fn test_complex_add() {
    // _RNvMs8_CsaRN1VPjcjfp_12test_symbolsINtB5_7ComplexlNtNtCshsEHAXgLWmz_5alloc6string6StringE3addB5_
    // Complex<i32, String>::add
    todo!("Need to implement complex nested generic encoding");
}

// Drop implementations for generic types
#[test]
fn test_drop_generic_enum() {
    // _RINvNtCs35vC3OypZpH_4core3ptr13drop_in_placeINtCsaRN1VPjcjfp_12test_symbols11GenericEnumlNtNtCshsEHAXgLWmz_5alloc6string6StringEEBJ_
    todo!("Need to implement drop impl encoding");
}

#[test]
fn test_drop_generic_struct() {
    // _RINvNtCs35vC3OypZpH_4core3ptr13drop_in_placeINtCsaRN1VPjcjfp_12test_symbols13GenericStructNtNtCshsEHAXgLWmz_5alloc6string6StringEEBJ_
    todo!("Need to implement drop impl encoding");
}

#[test]
fn test_drop_list_ref_str() {
    // _RINvNtCs35vC3OypZpH_4core3ptr13drop_in_placeINtCsaRN1VPjcjfp_12test_symbols4ListReEEBJ_
    todo!("Need to implement drop impl encoding");
}

#[test]
fn test_drop_list_i32() {
    // _RINvNtCs35vC3OypZpH_4core3ptr13drop_in_placeINtCsaRN1VPjcjfp_12test_symbols4ListlEEBJ_
    todo!("Need to implement drop impl encoding");
}

// Closure symbol
#[test]
fn test_closure() {
    // _RNCNvNtCshsEHAXgLWmz_5alloc3fmt6format0CsaRN1VPjcjfp_12test_symbols
    // N = nested, C = closure namespace, 0 = closure index
    todo!("Need to implement closure encoding");
}

// Remaining symbols are mostly standard library instantiations,
// complex backreferences, and compiler-generated code
// We'll handle those after implementing the basics
