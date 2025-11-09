//! Test crate for generating various Rust symbol manglings
//! This crate contains many different Rust constructs to test symbol mangling

use std::fmt::Display;

// Simple function
#[no_mangle]
pub extern "C" fn simple_function() -> i32 {
    42
}

// Generic function
pub fn generic_function<T: Display>(value: T) -> String {
    format!("{}", value)
}

// Generic function with multiple parameters
pub fn multi_generic<A, B, C>(a: A, b: B, c: C) -> (A, B, C) {
    (a, b, c)
}

// Function with various integer types
pub fn integer_types(
    a: i8, b: i16, c: i32, d: i64, e: i128,
    f: u8, g: u16, h: u32, i: u64, j: u128,
    k: isize, l: usize
) -> i32 {
    0
}

// Function with float types
pub fn float_types(a: f32, b: f64) -> f32 {
    a + b as f32
}

// Function with references
pub fn ref_function(a: &str, b: &mut i32, c: &[u8]) -> usize {
    *b = 10;
    a.len() + c.len()
}

// Function with raw pointers
pub fn ptr_function(a: *const i32, b: *mut u8) -> usize {
    0
}

// Function with arrays
pub fn array_function(arr: [i32; 10]) -> i32 {
    arr[0]
}

// Function with slices
pub fn slice_function(s: &[u8]) -> usize {
    s.len()
}

// Function with tuples
pub fn tuple_function(t: (i32, &str, bool)) -> i32 {
    t.0
}

// Const generic function
pub fn const_generic<const N: usize>(arr: [u8; N]) -> usize {
    N
}

// Simple struct
pub struct SimpleStruct {
    pub x: i32,
    pub y: String,
}

impl SimpleStruct {
    pub fn new(x: i32, y: String) -> Self {
        Self { x, y }
    }

    pub fn method(&self) -> i32 {
        self.x
    }

    pub fn generic_method<T>(&self, _value: T) -> i32 {
        self.x
    }
}

// Generic struct
pub struct GenericStruct<T> {
    pub value: T,
}

impl<T> GenericStruct<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }

    pub fn get(&self) -> &T {
        &self.value
    }
}

impl<T: Clone> GenericStruct<T> {
    pub fn clone_value(&self) -> T {
        self.value.clone()
    }
}

// Tuple struct
pub struct TupleStruct(pub i32, pub String);

// Unit struct
pub struct UnitStruct;

// Enum
pub enum SimpleEnum {
    Variant1,
    Variant2(i32),
    Variant3 { x: i32, y: String },
}

// Generic enum
pub enum GenericEnum<T, U> {
    Left(T),
    Right(U),
    Both(T, U),
}

// Trait
pub trait SimpleTrait {
    fn trait_method(&self) -> i32;
}

impl SimpleTrait for SimpleStruct {
    fn trait_method(&self) -> i32 {
        self.x
    }
}

impl SimpleTrait for i32 {
    fn trait_method(&self) -> i32 {
        *self
    }
}

// Generic trait
pub trait GenericTrait<T> {
    fn generic_trait_method(&self, value: T) -> T;
}

impl<T: Clone> GenericTrait<T> for GenericStruct<T> {
    fn generic_trait_method(&self, value: T) -> T {
        value
    }
}

// Module with nested items
pub mod inner {
    pub fn inner_function() -> i32 {
        42
    }

    pub struct InnerStruct {
        pub value: i32,
    }

    impl InnerStruct {
        pub fn inner_method(&self) -> i32 {
            self.value
        }
    }

    pub mod nested {
        pub fn deeply_nested_function() -> i32 {
            100
        }
    }
}

// Unicode identifiers
pub mod unicode {
    pub fn café() -> i32 {
        1
    }

    pub fn 日本語() -> i32 {
        2
    }

    pub fn Ελληνικά() -> i32 {
        3
    }

    pub struct Föö {
        pub bär: i32,
    }

    impl Föö {
        pub fn méthodé(&self) -> i32 {
            self.bär
        }
    }
}

// Function that returns a closure
pub fn returns_closure() -> impl Fn(i32) -> i32 {
    |x| x + 1
}

// Function with lifetime parameters
pub fn lifetime_function<'a>(s: &'a str) -> &'a str {
    s
}

// Function with multiple lifetimes
pub fn multi_lifetime<'a, 'b>(a: &'a str, b: &'b str) -> (&'a str, &'b str) {
    (a, b)
}

// Associated type trait
pub trait AssocTrait {
    type Output;
    fn assoc_method(&self) -> Self::Output;
}

impl AssocTrait for i32 {
    type Output = String;
    fn assoc_method(&self) -> Self::Output {
        self.to_string()
    }
}

// Const items
pub const CONST_VALUE: i32 = 42;
pub const CONST_ARRAY: [u8; 4] = [1, 2, 3, 4];

// Static items
pub static STATIC_VALUE: i32 = 100;

// Trait with default implementation
pub trait DefaultTrait {
    fn default_method(&self) -> i32 {
        0
    }

    fn required_method(&self) -> i32;
}

impl DefaultTrait for SimpleStruct {
    fn required_method(&self) -> i32 {
        self.x
    }
}

// Impl with where clause
impl<T> GenericStruct<T>
where
    T: Display,
{
    pub fn display(&self) -> String {
        format!("{}", self.value)
    }
}

// Function pointer type
pub fn takes_fn_ptr(f: fn(i32) -> i32) -> i32 {
    f(10)
}

// Unsafe function
pub unsafe fn unsafe_function(ptr: *const i32) -> i32 {
    *ptr
}

// Extern "C" function (exported)
#[no_mangle]
pub extern "C" fn extern_c_function(x: i32) -> i32 {
    x * 2
}

// Variadic-like function (not really variadic, but takes different types)
pub fn variadic_like<T: Display>(args: &[T]) -> String {
    args.iter()
        .map(|x| format!("{}", x))
        .collect::<Vec<_>>()
        .join(", ")
}

// Function that instantiates generics with concrete types
// This is exported to force the compiler to generate all the mangled symbols
pub extern "C" fn instantiate_generics() {
    // Generic functions with different types
    let _a = generic_function(42i32);
    let _b = generic_function("hello");
    let _c = generic_function(3.14f64);
    let _d = multi_generic(1u8, 2u16, 3u32);
    let _e = const_generic([1u8, 2, 3, 4, 5]);

    // Generic structs
    let _f = GenericStruct::new(42);
    let _g = GenericStruct::new("test");
    let gs = GenericStruct::new(100i32);
    let _h = gs.get();
    let _i = gs.clone_value();

    // Struct methods
    let s = SimpleStruct::new(10, "foo".to_string());
    let _j = s.method();
    let _k = s.generic_method(42);

    // Inner module
    let _l = inner::inner_function();
    let is = inner::InnerStruct { value: 5 };
    let _m = is.inner_method();
    let _n = inner::nested::deeply_nested_function();

    // Unicode functions
    let _o = unicode::café();
    let _p = unicode::日本語();
    let _q = unicode::Ελληνικά();
    let uf = unicode::Föö { bär: 42 };
    let _r = uf.méthodé();

    // Other types
    let _s = integer_types(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
    let _t = float_types(1.0, 2.0);
    let mut x = 0;
    let _u = ref_function("test", &mut x, &[1, 2, 3]);
    let _v = array_function([0; 10]);
    let _w = slice_function(&[1, 2, 3]);
    let _x = tuple_function((1, "hi", true));

    // Enums
    let _y = SimpleEnum::Variant1;
    let _z = GenericEnum::<i32, String>::Left(42);

    // Traits
    let _aa = s.trait_method();
    let _ab = 42i32.trait_method();

    // Complex types
    let mut c = Complex::<i32, String>::new();
    c.add(1, "test".to_string());

    // Lists
    let _ac = List::singleton(42);
    let _ad = List::singleton("test");
}

// Recursive type
pub struct List<T> {
    pub head: T,
    pub tail: Option<Box<List<T>>>,
}

impl<T> List<T> {
    pub fn singleton(value: T) -> Self {
        Self {
            head: value,
            tail: None,
        }
    }
}

// Complex nested generic
pub struct Complex<A, B>
where
    A: Clone,
    B: Display,
{
    pub data: Vec<(A, GenericStruct<B>)>,
}

impl<A: Clone, B: Display> Complex<A, B> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn add(&mut self, a: A, b: B) {
        self.data.push((a, GenericStruct::new(b)));
    }
}

// Trait object
pub fn takes_trait_object(obj: &dyn SimpleTrait) -> i32 {
    obj.trait_method()
}

// Higher-ranked trait bounds
pub fn hrtb_function<F>(f: F) -> i32
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    f("test").len() as i32
}
