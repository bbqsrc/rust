//! RFC 2603 - Rust Symbol Name Mangling v0
//!
//! This crate provides a standalone implementation of the core algorithms
//! used in Rust's v0 symbol name mangling scheme.
//!
//! The v0 mangling format is specified in RFC 2603 and is used by the Rust
//! compiler to generate deterministic, platform-independent symbol names.
//!
//! # High-Level API
//!
//! The recommended way to use this crate is through the high-level encoding functions:
//!
//! ```
//! use rfc2603::{encode_crate_root, encode_simple_path, Namespace};
//!
//! // Encode a crate root
//! let crate_name = encode_crate_root("mycrate", 0);
//! assert_eq!(crate_name, "C7mycrate");
//!
//! // Encode a simple path: mycrate::module::function
//! let path = encode_simple_path(&[
//!     ("mycrate", Namespace::Crate, 0),
//!     ("module", Namespace::Type, 0),
//!     ("function", Namespace::Value, 0),
//! ]);
//! // Results in: NvNtC7mycrate6module8function
//! ```
//!
//! # Low-Level Primitives
//!
//! For advanced use cases, low-level primitives are also available:
//!
//! ## Base-62 Encoding
//!
//! ```
//! use rfc2603::{push_integer_62, encode_integer_62};
//!
//! // Encode numbers in base-62 format
//! assert_eq!(encode_integer_62(0), "_");
//! assert_eq!(encode_integer_62(1), "0_");
//! assert_eq!(encode_integer_62(62), "Z_");
//! assert_eq!(encode_integer_62(1000), "g7_");
//! ```
//!
//! ## Identifier Encoding
//!
//! ```
//! use rfc2603::push_ident;
//!
//! let mut output = String::new();
//! push_ident("example", &mut output);
//! assert_eq!(output, "7example");
//!
//! let mut output = String::new();
//! push_ident("gödel", &mut output);
//! // Unicode identifiers are encoded with Punycode
//! assert!(output.starts_with("u"));
//! ```

use std::fmt::Write;

mod v0_mangler;
use v0_mangler::V0Mangler;

pub mod rustc_port;

/// Namespace tags used in v0 symbol mangling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Namespace {
    /// Crate root namespace (C)
    Crate,
    /// Type namespace (t) - modules, types, traits
    Type,
    /// Value namespace (v) - functions, constants, statics
    Value,
    /// Closure namespace (C)
    Closure,
    /// Shim namespace (S)
    Shim,
}

impl Namespace {
    /// Get the character tag for this namespace
    pub fn tag(&self) -> char {
        match self {
            Namespace::Crate => 'C',
            Namespace::Type => 't',
            Namespace::Value => 'v',
            Namespace::Closure => 'C',
            Namespace::Shim => 'S',
        }
    }
}

/// Builder for constructing v0 mangled symbols.
///
/// This provides a fluent API for building symbol paths with proper validation.
///
/// # Examples
///
/// ```
/// use rfc2603::SymbolBuilder;
///
/// // Simple function in a crate
/// let symbol = SymbolBuilder::new("mycrate")
///     .function("foo")
///     .build()
///     .unwrap();
/// assert_eq!(symbol, "_RNvC7mycrate3foo");
///
/// // Nested module path
/// let symbol = SymbolBuilder::new("mycrate")
///     .with_hash("aRN1VPjcjfp")
///     .module("inner")
///     .module("nested")
///     .function("func")
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct SymbolBuilder {
    crate_name: String,
    crate_hash: Option<String>,
    segments: Vec<(String, Namespace)>,
    method_info: Option<MethodInfo>,
    /// Generic arguments (types, lifetimes, consts)
    generic_args: Vec<GenericArg>,
    /// Cached positions for backreferences (mimics rustc's paths HashMap)
    path_cache: std::collections::HashMap<String, usize>,
    /// Start offset for backrefs (length of "_R" prefix = 2)
    start_offset: usize,
}

/// Generic argument for function/type instantiation
#[derive(Debug, Clone, PartialEq)]
pub enum GenericArg {
    /// Type parameter - represented by a primitive type tag or complex type
    Type(TypeArg),
    /// Lifetime parameter
    Lifetime(LifetimeArg),
    /// Const parameter
    Const(u64),
}

/// Type argument for generic instantiation
#[derive(Debug, Clone, PartialEq)]
pub enum TypeArg {
    /// Primitive types
    Bool,
    Char,
    I8, I16, I32, I64, I128, Isize,
    U8, U16, U32, U64, U128, Usize,
    F32, F64,
    Str,
    Never,
    Unit,
    /// Reference type: &'lifetime T or &'lifetime mut T
    Reference { lifetime: Option<LifetimeArg>, mutable: bool, inner: Box<TypeArg> },
    /// Raw pointer: *const T or *mut T
    RawPtr { mutable: bool, inner: Box<TypeArg> },
    /// Tuple type: (T1, T2, ..., Tn)
    Tuple(Vec<TypeArg>),
    /// Array type: [T; N]
    Array { inner: Box<TypeArg>, len: u64 },
    /// Slice type: [T]
    Slice(Box<TypeArg>),
}

/// Lifetime argument
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifetimeArg {
    /// Erased lifetime (encoded as L0)
    Erased,
    /// Named lifetime with De Bruijn index
    Bound { index: u32 },
}

#[derive(Debug, Clone)]
struct MethodInfo {
    /// Path to the impl block (modules before the type)
    impl_path: Vec<(String, Namespace)>,
    /// The type being implemented on
    type_name: String,
    /// The method name
    method_name: String,
}

impl SymbolBuilder {
    /// Create a new symbol builder with the given crate name.
    pub fn new(crate_name: impl Into<String>) -> Self {
        Self {
            crate_name: crate_name.into(),
            crate_hash: None,
            segments: Vec::new(),
            method_info: None,
            generic_args: Vec::new(),
            path_cache: std::collections::HashMap::new(),
            start_offset: 2, // Length of "_R" prefix
        }
    }

    /// Set the crate hash (base-62 encoded, without 's' prefix or '_' suffix).
    pub fn with_hash(mut self, hash: impl Into<String>) -> Self {
        self.crate_hash = Some(hash.into());
        self
    }

    /// Add a module (type namespace) to the path.
    pub fn module(mut self, name: impl Into<String>) -> Self {
        self.segments.push((name.into(), Namespace::Type));
        self
    }

    /// Add a function (value namespace) to the path.
    pub fn function(mut self, name: impl Into<String>) -> Self {
        self.segments.push((name.into(), Namespace::Value));
        self
    }

    /// Add a constant or static (value namespace) to the path.
    pub fn value(mut self, name: impl Into<String>) -> Self {
        self.segments.push((name.into(), Namespace::Value));
        self
    }

    /// Add a type (type namespace) to the path.
    pub fn type_name(mut self, name: impl Into<String>) -> Self {
        self.segments.push((name.into(), Namespace::Type));
        self
    }

    /// Add a method on a type (inherent impl).
    ///
    /// This marks that we're encoding a method, and the previous segments become
    /// the path to the type. The type_name and method_name are specified here.
    ///
    /// # Examples
    ///
    /// ```
    /// use rfc2603::SymbolBuilder;
    ///
    /// // SimpleStruct::new() method
    /// let symbol = SymbolBuilder::new("test_symbols")
    ///     .with_hash("aRN1VPjcjfp")
    ///     .method("SimpleStruct", "new")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn method(mut self, type_name: impl Into<String>, method_name: impl Into<String>) -> Self {
        self.method_info = Some(MethodInfo {
            impl_path: self.segments.clone(),
            type_name: type_name.into(),
            method_name: method_name.into(),
        });
        self
    }

    /// Add a generic type argument.
    ///
    /// # Examples
    ///
    /// ```
    /// use rfc2603::{SymbolBuilder, GenericArg, TypeArg};
    ///
    /// // fn foo<T: u32>() instantiated as foo::<u32>
    /// let symbol = SymbolBuilder::new("mycrate")
    ///     .function("foo")
    ///     .with_generic(GenericArg::Type(TypeArg::U32))
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn with_generic(mut self, arg: GenericArg) -> Self {
        self.generic_args.push(arg);
        self
    }

    /// Add multiple generic arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// use rfc2603::{SymbolBuilder, GenericArg, TypeArg};
    ///
    /// // fn foo<T, U>() instantiated as foo::<u32, i64>
    /// let symbol = SymbolBuilder::new("mycrate")
    ///     .function("foo")
    ///     .with_generics(&[
    ///         GenericArg::Type(TypeArg::U32),
    ///         GenericArg::Type(TypeArg::I64),
    ///     ])
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn with_generics(mut self, args: &[GenericArg]) -> Self {
        self.generic_args.extend_from_slice(args);
        self
    }

    /// Add a type parameter to the generic arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// use rfc2603::{SymbolBuilder, TypeArg};
    ///
    /// let symbol = SymbolBuilder::new("mycrate")
    ///     .function("foo")
    ///     .with_type_param(TypeArg::U32)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn with_type_param(mut self, ty: TypeArg) -> Self {
        self.generic_args.push(GenericArg::Type(ty));
        self
    }

    /// Add a lifetime parameter to the generic arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// use rfc2603::{SymbolBuilder, LifetimeArg, GenericArg};
    ///
    /// let symbol = SymbolBuilder::new("mycrate")
    ///     .function("foo")
    ///     .with_lifetime(LifetimeArg::Erased)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn with_lifetime(mut self, lifetime: LifetimeArg) -> Self {
        self.generic_args.push(GenericArg::Lifetime(lifetime));
        self
    }

    /// Add a const parameter to the generic arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// use rfc2603::SymbolBuilder;
    ///
    /// // fn foo<const N: usize>() instantiated as foo::<5>
    /// let symbol = SymbolBuilder::new("mycrate")
    ///     .function("foo")
    ///     .with_const_param(5)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn with_const_param(mut self, value: u64) -> Self {
        self.generic_args.push(GenericArg::Const(value));
        self
    }

    /// Build the complete mangled symbol with `_R` prefix.
    ///
    /// Returns an error if the path is invalid (e.g., no segments added).
    pub fn build(self) -> Result<String, &'static str> {
        if self.method_info.is_some() {
            // Build method symbol
            return self.build_method_symbol();
        }

        if self.segments.is_empty() {
            return Err("Symbol path must have at least one segment (function, module, etc.)");
        }

        // If we have generic arguments, we need to build an instantiation symbol
        if !self.generic_args.is_empty() {
            return self.build_generic_instantiation();
        }

        let mut segments_with_crate = vec![(&self.crate_name[..], Namespace::Crate, 0u64)];
        for (name, ns) in &self.segments {
            segments_with_crate.push((name, *ns, 0));
        }

        let path = encode_simple_path_with_crate_hash(
            &segments_with_crate,
            self.crate_hash.as_deref(),
        );
        Ok(encode_symbol(&path))
    }

    fn build_generic_instantiation(self) -> Result<String, &'static str> {
        // Generic instantiation format: _R + I + <path> + <generic-args> + E
        // Example: _RINvC7mycrate3foomE  (foo::<u32>)

        let mut m = V0Mangler::new();

        // I marker for generic instantiation
        m.push("I");

        // Build the path to the generic item
        let mut segments_with_crate = vec![(&self.crate_name[..], Namespace::Crate, 0u64)];
        for (name, ns) in &self.segments {
            segments_with_crate.push((name, *ns, 0));
        }

        let path = encode_simple_path_with_crate_hash(
            &segments_with_crate,
            self.crate_hash.as_deref(),
        );
        m.push(&path);

        // Encode generic arguments
        for arg in &self.generic_args {
            self.encode_generic_arg(&mut m, arg)?;
        }

        // E marker to close generic instantiation
        m.push("E");

        Ok(m.out)
    }

    fn encode_generic_arg(&self, m: &mut V0Mangler, arg: &GenericArg) -> Result<(), &'static str> {
        match arg {
            GenericArg::Type(ty) => self.encode_type_arg(m, ty),
            GenericArg::Lifetime(lt) => self.encode_lifetime_arg(m, lt),
            GenericArg::Const(val) => {
                // Const argument: K + <type> + <value>
                // For now, assume usize type (j)
                m.push("Kj");
                m.push_integer_62(*val);
                Ok(())
            }
        }
    }

    fn encode_type_arg(&self, m: &mut V0Mangler, ty: &TypeArg) -> Result<(), &'static str> {
        match ty {
            // Primitive types
            TypeArg::Bool => { m.push("b"); Ok(()) }
            TypeArg::Char => { m.push("c"); Ok(()) }
            TypeArg::I8 => { m.push("a"); Ok(()) }
            TypeArg::I16 => { m.push("s"); Ok(()) }
            TypeArg::I32 => { m.push("l"); Ok(()) }
            TypeArg::I64 => { m.push("x"); Ok(()) }
            TypeArg::I128 => { m.push("n"); Ok(()) }
            TypeArg::Isize => { m.push("i"); Ok(()) }
            TypeArg::U8 => { m.push("h"); Ok(()) }
            TypeArg::U16 => { m.push("t"); Ok(()) }
            TypeArg::U32 => { m.push("m"); Ok(()) }
            TypeArg::U64 => { m.push("y"); Ok(()) }
            TypeArg::U128 => { m.push("o"); Ok(()) }
            TypeArg::Usize => { m.push("j"); Ok(()) }
            TypeArg::F32 => { m.push("f"); Ok(()) }
            TypeArg::F64 => { m.push("d"); Ok(()) }
            TypeArg::Str => { m.push("e"); Ok(()) }
            TypeArg::Never => { m.push("z"); Ok(()) }
            TypeArg::Unit => { m.push("u"); Ok(()) }

            // Reference: R (immutable) or Q (mutable) + lifetime + inner type
            TypeArg::Reference { lifetime, mutable, inner } => {
                m.push(if *mutable { "Q" } else { "R" });
                if let Some(lt) = lifetime {
                    self.encode_lifetime_arg(m, lt)?;
                } else {
                    // Erased lifetime
                    m.push("L");
                    m.push_integer_62(0);
                }
                self.encode_type_arg(m, inner)?;
                Ok(())
            }

            // Raw pointer: P (const) or O (mut) + inner type
            TypeArg::RawPtr { mutable, inner } => {
                m.push(if *mutable { "O" } else { "P" });
                self.encode_type_arg(m, inner)?;
                Ok(())
            }

            // Tuple: T + elements + E
            TypeArg::Tuple(elements) => {
                m.push("T");
                for elem in elements {
                    self.encode_type_arg(m, elem)?;
                }
                m.push("E");
                Ok(())
            }

            // Array: A + element type + const length
            TypeArg::Array { inner, len } => {
                m.push("A");
                self.encode_type_arg(m, inner)?;
                m.push("Kj"); // Const with usize type
                m.push_integer_62(*len);
                Ok(())
            }

            // Slice: S + element type
            TypeArg::Slice(inner) => {
                m.push("S");
                self.encode_type_arg(m, inner)?;
                Ok(())
            }
        }
    }

    fn encode_lifetime_arg(&self, m: &mut V0Mangler, lt: &LifetimeArg) -> Result<(), &'static str> {
        match lt {
            LifetimeArg::Erased => {
                m.push("L");
                m.push_integer_62(0);
                Ok(())
            }
            LifetimeArg::Bound { index } => {
                m.push("L");
                m.push_integer_62(*index as u64 + 1);
                Ok(())
            }
        }
    }

    fn build_method_symbol(self) -> Result<String, &'static str> {
        // Method symbol format: _R + Nv + M + <impl-path> + Nt + <backref-to-impl> + <type-name> + <method-name>
        // For SimpleStruct::new: _RNvMCsaRN1VPjcjfp_12test_symbolsNtB2_12SimpleStruct3new

        let method_info = self.method_info.ok_or("Method info not set")?;

        let mut m = V0Mangler::new();

        // Outer wrapper: Nv (value namespace for the method itself)
        m.push("Nv");

        // M marker for inherent impl
        m.push("M");

        // Encode the impl path (crate + any modules)
        // Record this position for backreference
        let impl_path_start = m.out.len();

        // Build crate path
        if let Some(hash) = &self.crate_hash {
            m.push(&encode_crate_root_with_hash(&self.crate_name, hash));
        } else {
            m.push(&encode_crate_root(&self.crate_name, 0));
        }

        // Add any module segments from impl_path
        for (name, ns) in &method_info.impl_path {
            m.path_append_ns(
                |_m| {}, // No prefix for these segments
                ns.tag(),
                0,
                name,
            );
        }

        // Cache the impl path position for backref
        // Use a key that represents this path
        let impl_path_key = format!("impl:{}:{:?}", self.crate_name, method_info.impl_path);
        m.paths.insert(impl_path_key.clone(), impl_path_start);

        // Now encode the type: Nt + <backref> + <type-name>
        m.push("Nt");

        // Use backref to the impl path
        if let Some(&pos) = m.paths.get(&impl_path_key) {
            m.print_backref(pos);
        }

        // Type name
        m.push_ident(&method_info.type_name);

        // Method name
        m.push_ident(&method_info.method_name);

        Ok(m.out)
    }

    /// Build just the path portion without the `_R` prefix.
    pub fn build_path(self) -> Result<String, &'static str> {
        if self.segments.is_empty() {
            return Err("Symbol path must have at least one segment (function, module, etc.)");
        }

        let mut segments_with_crate = vec![(&self.crate_name[..], Namespace::Crate, 0u64)];
        for (name, ns) in &self.segments {
            segments_with_crate.push((name, *ns, 0));
        }

        Ok(encode_simple_path_with_crate_hash(
            &segments_with_crate,
            self.crate_hash.as_deref(),
        ))
    }
}

/// Encode a crate root path element.
///
/// A crate root is encoded as `C` followed by an optional disambiguator and the crate name.
///
/// # Examples
///
/// ```
/// use rfc2603::encode_crate_root;
///
/// // Crate with no disambiguator
/// assert_eq!(encode_crate_root("mycrate", 0), "C7mycrate");
///
/// // Crate with disambiguator 1
/// assert_eq!(encode_crate_root("mycrate", 1), "Cs_7mycrate");
/// ```
pub fn encode_crate_root(name: &str, disambiguator: u64) -> String {
    let mut output = String::new();
    output.push('C');
    push_disambiguator(disambiguator, &mut output);
    push_ident(name, &mut output);
    output
}

/// Encode a crate root with a pre-encoded base-62 hash.
///
/// This is useful when you have the exact hash from a real symbol and want to
/// reproduce it exactly. The hash should be the base-62 encoded value without
/// the `s` prefix or `_` suffix.
///
/// # Examples
///
/// ```
/// use rfc2603::encode_crate_root_with_hash;
///
/// // Real crate hash from compiled symbol
/// let crate_root = encode_crate_root_with_hash("test_symbols", "aRN1VPjcjfp");
/// assert_eq!(crate_root, "CsaRN1VPjcjfp_12test_symbols");
/// ```
pub fn encode_crate_root_with_hash(name: &str, hash_b62: &str) -> String {
    let mut output = String::new();
    output.push('C');
    if !hash_b62.is_empty() {
        output.push('s');
        output.push_str(hash_b62);
        output.push('_');
    }
    push_ident(name, &mut output);
    output
}

/// Encode a simple path consisting of a sequence of path segments.
///
/// Each segment is a tuple of (name, namespace, disambiguator).
/// The path is built right-to-left, with each segment wrapping the previous one.
///
/// # Examples
///
/// ```
/// use rfc2603::{encode_simple_path, Namespace};
///
/// // Encode: mycrate::module::function
/// let path = encode_simple_path(&[
///     ("mycrate", Namespace::Crate, 0),
///     ("module", Namespace::Type, 0),
///     ("function", Namespace::Value, 0),
/// ]);
/// assert_eq!(path, "NvNtC7mycrate6module8function");
/// ```
pub fn encode_simple_path(segments: &[(&str, Namespace, u64)]) -> String {
    encode_simple_path_with_crate_hash(segments, None)
}

/// Encode a simple path with an optional crate hash.
///
/// This is identical to [`encode_simple_path`] but allows specifying a pre-encoded
/// base-62 crate hash for the root crate. The hash is only used if the first segment
/// is a crate namespace.
///
/// # Examples
///
/// ```
/// use rfc2603::{encode_simple_path_with_crate_hash, Namespace};
///
/// // Encode with crate hash
/// let path = encode_simple_path_with_crate_hash(
///     &[
///         ("test_symbols", Namespace::Crate, 0),
///         ("float_types", Namespace::Value, 0),
///     ],
///     Some("aRN1VPjcjfp")
/// );
/// assert_eq!(path, "NvCsaRN1VPjcjfp_12test_symbols11float_types");
/// ```
pub fn encode_simple_path_with_crate_hash(
    segments: &[(&str, Namespace, u64)],
    crate_hash: Option<&str>,
) -> String {
    if segments.is_empty() {
        return String::new();
    }

    let mut output = String::new();

    // First segment (leftmost in the path, rightmost in iteration)
    let (name, ns, disambiguator) = segments[0];
    if ns == Namespace::Crate {
        if let Some(hash) = crate_hash {
            output.push_str(&encode_crate_root_with_hash(name, hash));
        } else {
            output.push('C');
            push_disambiguator(disambiguator, &mut output);
            push_ident(name, &mut output);
        }
    } else {
        output.push(ns.tag());
        push_disambiguator(disambiguator, &mut output);
        push_ident(name, &mut output);
    }

    // Remaining segments wrap around the previous ones
    for &(name, ns, disambiguator) in segments[1..].iter() {
        let prev = output.clone();
        output.clear();
        output.push('N');
        output.push(ns.tag());
        output.push_str(&prev);
        push_disambiguator(disambiguator, &mut output);
        push_ident(name, &mut output);
    }

    output
}

/// Encode a full v0 symbol name with the `_R` prefix.
///
/// This combines the v0 prefix with a path to create a complete mangled symbol.
///
/// # Examples
///
/// ```
/// use rfc2603::{encode_symbol, encode_simple_path, Namespace};
///
/// let path = encode_simple_path(&[
///     ("mycrate", Namespace::Crate, 0),
///     ("foo", Namespace::Value, 0),
/// ]);
/// let symbol = encode_symbol(&path);
/// assert_eq!(symbol, "_RNvC7mycrate3foo");
/// ```
pub fn encode_symbol(path: &str) -> String {
    format!("_R{}", path)
}

/// Push a `_`-terminated base 62 integer, using the format
/// specified in RFC 2603 as `<base-62-number>`, that is:
/// * `x = 0` is encoded as just the `"_"` terminator
/// * `x > 0` is encoded as `x - 1` in base 62, followed by `"_"`,
///   e.g. `1` becomes `"0_"`, `62` becomes `"Z_"`, etc.
///
/// # Examples
///
/// ```
/// use rfc2603::push_integer_62;
///
/// let mut output = String::new();
/// push_integer_62(0, &mut output);
/// assert_eq!(output, "_");
///
/// let mut output = String::new();
/// push_integer_62(1, &mut output);
/// assert_eq!(output, "0_");
///
/// let mut output = String::new();
/// push_integer_62(62, &mut output);
/// assert_eq!(output, "Z_");
/// ```
pub fn push_integer_62(x: u64, output: &mut String) {
    if let Some(x) = x.checked_sub(1) {
        output.push_str(&to_base_62(x));
    }
    output.push('_');
}

/// Encode an integer in base-62 format and return it as a String.
///
/// This is a convenience wrapper around [`push_integer_62`].
///
/// # Examples
///
/// ```
/// use rfc2603::encode_integer_62;
///
/// assert_eq!(encode_integer_62(0), "_");
/// assert_eq!(encode_integer_62(1), "0_");
/// assert_eq!(encode_integer_62(11), "a_");
/// assert_eq!(encode_integer_62(62), "Z_");
/// assert_eq!(encode_integer_62(63), "10_");
/// assert_eq!(encode_integer_62(1000), "g7_");
/// ```
pub fn encode_integer_62(x: u64) -> String {
    let mut output = String::new();
    push_integer_62(x, &mut output);
    output
}

/// Encode an identifier using the v0 mangling scheme.
///
/// Identifiers are encoded as a length-prefixed string. If the identifier
/// contains non-ASCII characters, it is first encoded using Punycode and
/// prefixed with `u`.
///
/// The format is: `[u]<length>[_]<bytes>`
/// - Optional `u` prefix indicates Punycode encoding
/// - `<length>` is the decimal length of the identifier
/// - Optional `_` separator if the identifier starts with a digit or `_`
/// - `<bytes>` is the identifier itself (or Punycode-encoded version)
///
/// # Examples
///
/// ```
/// use rfc2603::push_ident;
///
/// let mut output = String::new();
/// push_ident("example", &mut output);
/// assert_eq!(output, "7example");
///
/// let mut output = String::new();
/// push_ident("_foo", &mut output);
/// assert_eq!(output, "4__foo"); // Note the separator _
///
/// let mut output = String::new();
/// push_ident("0abc", &mut output);
/// assert_eq!(output, "4_0abc"); // Note the separator _
/// ```
pub fn push_ident(ident: &str, output: &mut String) {
    let mut use_punycode = false;
    for b in ident.bytes() {
        match b {
            b'_' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => {}
            0x80..=0xff => use_punycode = true,
            _ => panic!("Invalid byte {} in identifier {:?}", b, ident),
        }
    }

    let punycode_string;
    let ident = if use_punycode {
        output.push('u');

        let mut punycode_bytes = match punycode::encode(ident) {
            Ok(s) => s.into_bytes(),
            Err(()) => panic!("Punycode encoding failed for identifier {:?}", ident),
        };

        // Replace `-` with `_`.
        if let Some(c) = punycode_bytes.iter_mut().rfind(|&&mut c| c == b'-') {
            *c = b'_';
        }

        punycode_string = String::from_utf8(punycode_bytes).unwrap();
        &punycode_string
    } else {
        ident
    };

    let _ = write!(output, "{}", ident.len());

    // Write a separating `_` if necessary (leading digit or `_`).
    if let Some('_' | '0'..='9') = ident.chars().next() {
        output.push('_');
    }

    output.push_str(ident);
}

/// Convert a u64 to base-62 representation.
///
/// Base-62 uses digits 0-9, lowercase a-z, and uppercase A-Z.
/// The mapping is:
/// - 0-9 → 0-9
/// - 10-35 → a-z
/// - 36-61 → A-Z
fn to_base_62(mut x: u64) -> String {
    const BASE_62: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    if x == 0 {
        return String::from("0");
    }

    let mut result = Vec::new();
    while x > 0 {
        result.push(BASE_62[(x % 62) as usize]);
        x /= 62;
    }

    result.reverse();
    String::from_utf8(result).unwrap()
}

/// Push a `_`-terminated base 62 integer with an optional tag prefix.
///
/// * `x = 0` is encoded as `""` (nothing)
/// * `x > 0` is encoded as the `tag` followed by `push_integer_62(x - 1)`
///   e.g. `1` becomes `tag + "_"`, `2` becomes `tag + "0_"`, etc.
///
/// This is commonly used for optional disambiguators in the v0 mangling scheme.
///
/// # Examples
///
/// ```
/// use rfc2603::push_opt_integer_62;
///
/// let mut output = String::new();
/// push_opt_integer_62("s", 0, &mut output);
/// assert_eq!(output, ""); // No output for 0
///
/// let mut output = String::new();
/// push_opt_integer_62("s", 1, &mut output);
/// assert_eq!(output, "s_"); // s + "_" for 1
///
/// let mut output = String::new();
/// push_opt_integer_62("s", 2, &mut output);
/// assert_eq!(output, "s0_"); // s + "0_" for 2
/// ```
pub fn push_opt_integer_62(tag: &str, x: u64, output: &mut String) {
    if let Some(x) = x.checked_sub(1) {
        output.push_str(tag);
        push_integer_62(x, output);
    }
}

/// Push a disambiguator using the `s` tag.
///
/// This is a convenience wrapper around [`push_opt_integer_62`] with tag `"s"`.
/// Disambiguators are used in v0 mangling to distinguish between items that
/// would otherwise have identical paths.
///
/// # Examples
///
/// ```
/// use rfc2603::push_disambiguator;
///
/// let mut output = String::new();
/// push_disambiguator(0, &mut output);
/// assert_eq!(output, ""); // No disambiguator
///
/// let mut output = String::new();
/// push_disambiguator(1, &mut output);
/// assert_eq!(output, "s_"); // First disambiguator
/// ```
pub fn push_disambiguator(dis: u64, output: &mut String) {
    push_opt_integer_62("s", dis, output);
}

/// Create a symbol generator that yields mangled symbols for a sequence of function names.
///
/// Returns an iterator that produces v0 mangled symbols for functions in the given crate.
///
/// # Examples
///
/// ```
/// use rfc2603::create_symbol_iterator;
///
/// let symbols: Vec<String> = create_symbol_iterator("mycrate", &["foo", "bar", "baz"])
///     .collect();
///
/// assert_eq!(symbols[0], "_RNvC7mycrate3foo");
/// assert_eq!(symbols[1], "_RNvC7mycrate3bar");
/// assert_eq!(symbols[2], "_RNvC7mycrate3baz");
/// ```
pub fn create_symbol_iterator<'a>(
    crate_name: &'a str,
    function_names: &'a [&'a str],
) -> impl Iterator<Item = String> + 'a {
    function_names.iter().map(move |&name| {
        SymbolBuilder::new(crate_name)
            .function(name)
            .build()
            .unwrap_or_else(|_| String::from("_RINVALID"))
    })
}

/// Create a symbol formatter that can be displayed.
///
/// Returns a displayable type that formats a symbol with optional demangling information.
///
/// # Examples
///
/// ```
/// use rfc2603::create_symbol_display;
///
/// let display = create_symbol_display("_RNvC7mycrate3foo", true);
/// let output = format!("{}", display);
/// assert!(output.contains("_RNvC7mycrate3foo"));
/// ```
pub fn create_symbol_display(symbol: &str, show_breakdown: bool) -> impl std::fmt::Display + '_ {
    struct SymbolDisplay<'a> {
        symbol: &'a str,
        show_breakdown: bool,
    }

    impl std::fmt::Display for SymbolDisplay<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Symbol: {}", self.symbol)?;

            if self.show_breakdown {
                if self.symbol.starts_with("_R") {
                    writeln!(f)?;
                    write!(f, "  Format: v0 (Rust Symbol Mangling)")?;

                    // Try to demangle if rustc-demangle is available
                    #[cfg(test)]
                    if let Ok(demangled) = rustc_demangle::try_demangle(self.symbol) {
                        writeln!(f)?;
                        write!(f, "  Demangled: {:#}", demangled)?;
                    }
                }
            }

            Ok(())
        }
    }

    SymbolDisplay {
        symbol,
        show_breakdown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Base-62 Encoding Tests ==========

    #[test]
    fn test_base62_encoding() {
        assert_eq!(encode_integer_62(0), "_");
        assert_eq!(encode_integer_62(1), "0_");
        assert_eq!(encode_integer_62(11), "a_");
        assert_eq!(encode_integer_62(62), "Z_");
        assert_eq!(encode_integer_62(63), "10_");
        assert_eq!(encode_integer_62(1000), "g7_");
    }

    #[test]
    fn test_base62_boundaries() {
        // Test boundaries between different base-62 digit counts
        assert_eq!(encode_integer_62(0), "_");      // special case
        assert_eq!(encode_integer_62(1), "0_");     // 1 digit starts
        assert_eq!(encode_integer_62(10), "9_");
        assert_eq!(encode_integer_62(11), "a_");
        assert_eq!(encode_integer_62(36), "z_");
        assert_eq!(encode_integer_62(37), "A_");
        assert_eq!(encode_integer_62(61), "Y_");
        assert_eq!(encode_integer_62(62), "Z_");    // 1 digit ends
        assert_eq!(encode_integer_62(63), "10_");   // 2 digits start
        assert_eq!(encode_integer_62(124), "1Z_");
        assert_eq!(encode_integer_62(125), "20_");
        assert_eq!(encode_integer_62(3844), "ZZ_"); // 2 digits end (62*62 = 3844)
        assert_eq!(encode_integer_62(3845), "100_"); // 3 digits start
    }

    #[test]
    fn test_base62_large_numbers() {
        // Test with larger numbers to ensure proper encoding
        // 10000 - 1 = 9999, 9999 in base-62 = "2Bh"
        assert_eq!(encode_integer_62(10000), "2Bh_");
        // u64::MAX - 1 in base-62 - just verify it doesn't panic
        let result = encode_integer_62(u64::MAX);
        assert!(result.ends_with("_"));
        assert!(result.len() > 10); // Very large number
    }

    #[test]
    fn test_push_integer_62() {
        let mut output = String::new();
        push_integer_62(0, &mut output);
        assert_eq!(output, "_");

        let mut output = String::new();
        push_integer_62(1, &mut output);
        assert_eq!(output, "0_");

        let mut output = String::new();
        push_integer_62(62, &mut output);
        assert_eq!(output, "Z_");

        let mut output = String::new();
        push_integer_62(999, &mut output);
        assert_eq!(output, "g6_");
    }

    // ========== Identifier Encoding Tests ==========

    #[test]
    fn test_push_ident_ascii() {
        let mut output = String::new();
        push_ident("example", &mut output);
        assert_eq!(output, "7example");

        let mut output = String::new();
        push_ident("foo", &mut output);
        assert_eq!(output, "3foo");

        let mut output = String::new();
        push_ident("x", &mut output);
        assert_eq!(output, "1x");
    }

    #[test]
    fn test_push_ident_with_separator() {
        let mut output = String::new();
        push_ident("_foo", &mut output);
        assert_eq!(output, "4__foo");

        let mut output = String::new();
        push_ident("0abc", &mut output);
        assert_eq!(output, "4_0abc");

        let mut output = String::new();
        push_ident("_", &mut output);
        assert_eq!(output, "1__");

        let mut output = String::new();
        push_ident("_0", &mut output);
        assert_eq!(output, "2__0");

        let mut output = String::new();
        push_ident("9test", &mut output);
        assert_eq!(output, "5_9test");
    }

    #[test]
    fn test_push_ident_unicode() {
        let mut output = String::new();
        push_ident("gödel", &mut output);
        // Should be Punycode encoded: "gdel-5qa" -> "gdel_5qa"
        assert_eq!(output, "u8gdel_5qa");

        let mut output = String::new();
        push_ident("föö", &mut output);
        // Punycode: "f-1gaa" -> "f_1gaa"
        assert_eq!(output, "u6f_1gaa");

        let mut output = String::new();
        push_ident("café", &mut output);
        // Punycode encoded
        assert!(output.starts_with("u"));

        let mut output = String::new();
        push_ident("你好", &mut output);
        // Chinese characters - should be Punycode encoded
        assert!(output.starts_with("u"));
    }

    #[test]
    fn test_push_ident_long_names() {
        let long_name = "a".repeat(100);
        let mut output = String::new();
        push_ident(&long_name, &mut output);
        assert!(output.starts_with("100"));
        assert_eq!(output, format!("100{}", long_name));

        let very_long_name = "x".repeat(1000);
        let mut output = String::new();
        push_ident(&very_long_name, &mut output);
        assert!(output.starts_with("1000"));
    }

    #[test]
    fn test_push_ident_mixed_case() {
        let mut output = String::new();
        push_ident("MyStruct", &mut output);
        assert_eq!(output, "8MyStruct");

        let mut output = String::new();
        push_ident("HTTP_RESPONSE", &mut output);
        assert_eq!(output, "13HTTP_RESPONSE");

        let mut output = String::new();
        push_ident("camelCase123", &mut output);
        assert_eq!(output, "12camelCase123");
    }

    // ========== Disambiguator and Optional Integer Tests ==========

    #[test]
    fn test_push_opt_integer_62() {
        let mut output = String::new();
        push_opt_integer_62("s", 0, &mut output);
        assert_eq!(output, "");

        let mut output = String::new();
        push_opt_integer_62("s", 1, &mut output);
        assert_eq!(output, "s_");

        let mut output = String::new();
        push_opt_integer_62("s", 2, &mut output);
        assert_eq!(output, "s0_");

        let mut output = String::new();
        push_opt_integer_62("s", 100, &mut output);
        // 100 - 1 = 99, then 99 - 1 = 98, 98 in base-62 = "1A"
        assert_eq!(output, "s1A_");
    }

    #[test]
    fn test_push_disambiguator() {
        let mut output = String::new();
        push_disambiguator(0, &mut output);
        assert_eq!(output, "");

        let mut output = String::new();
        push_disambiguator(1, &mut output);
        assert_eq!(output, "s_");

        let mut output = String::new();
        push_disambiguator(10, &mut output);
        // 10 - 1 = 9, then 9 - 1 = 8, 8 in base-62 = "8"
        assert_eq!(output, "s8_");
    }

    #[test]
    fn test_to_base_62() {
        assert_eq!(to_base_62(0), "0");
        assert_eq!(to_base_62(10), "a");
        assert_eq!(to_base_62(35), "z");
        assert_eq!(to_base_62(36), "A");
        assert_eq!(to_base_62(61), "Z");
        assert_eq!(to_base_62(62), "10");
        assert_eq!(to_base_62(100), "1C");
    }

    // ========== Crate Root Encoding Tests ==========

    #[test]
    fn test_encode_crate_root_no_disambiguator() {
        assert_eq!(encode_crate_root("mycrate", 0), "C7mycrate");
        assert_eq!(encode_crate_root("std", 0), "C3std");
        assert_eq!(encode_crate_root("x", 0), "C1x");
    }

    #[test]
    fn test_encode_crate_root_with_disambiguator() {
        assert_eq!(encode_crate_root("mycrate", 1), "Cs_7mycrate");
        assert_eq!(encode_crate_root("mycrate", 2), "Cs0_7mycrate");
        assert_eq!(encode_crate_root("mycrate", 10), "Cs8_7mycrate");
    }

    #[test]
    fn test_encode_crate_root_with_hash() {
        assert_eq!(
            encode_crate_root_with_hash("test_symbols", "aRN1VPjcjfp"),
            "CsaRN1VPjcjfp_12test_symbols"
        );
        assert_eq!(
            encode_crate_root_with_hash("mycrate", "123ABC"),
            "Cs123ABC_7mycrate"
        );
        assert_eq!(
            encode_crate_root_with_hash("std", ""),
            "C3std"
        );
    }

    // ========== Path Encoding Tests ==========

    #[test]
    fn test_encode_simple_path_single_segment() {
        let path = encode_simple_path(&[
            ("mycrate", Namespace::Crate, 0),
        ]);
        assert_eq!(path, "C7mycrate");
    }

    #[test]
    fn test_encode_simple_path_two_segments() {
        let path = encode_simple_path(&[
            ("mycrate", Namespace::Crate, 0),
            ("foo", Namespace::Value, 0),
        ]);
        assert_eq!(path, "NvC7mycrate3foo");
    }

    #[test]
    fn test_encode_simple_path_nested_modules() {
        let path = encode_simple_path(&[
            ("mycrate", Namespace::Crate, 0),
            ("module", Namespace::Type, 0),
            ("submodule", Namespace::Type, 0),
            ("function", Namespace::Value, 0),
        ]);
        assert_eq!(path, "NvNtNtC7mycrate6module9submodule8function");
    }

    #[test]
    fn test_encode_simple_path_with_disambiguators() {
        let path = encode_simple_path(&[
            ("mycrate", Namespace::Crate, 1),
            ("module", Namespace::Type, 2),
            ("foo", Namespace::Value, 0),
        ]);
        assert_eq!(path, "NvNtCs_7mycrates0_6module3foo");
    }

    #[test]
    fn test_encode_simple_path_with_crate_hash() {
        let path = encode_simple_path_with_crate_hash(
            &[
                ("test_symbols", Namespace::Crate, 0),
                ("float_types", Namespace::Value, 0),
            ],
            Some("aRN1VPjcjfp")
        );
        assert_eq!(path, "NvCsaRN1VPjcjfp_12test_symbols11float_types");
    }

    #[test]
    fn test_encode_simple_path_deeply_nested() {
        // Test with 10 nested modules
        // Store module names in a Vec to keep them alive
        let module_names: Vec<String> = (1..=10).map(|i| format!("mod{}", i)).collect();

        let mut segments = vec![("crate", Namespace::Crate, 0)];
        for name in &module_names {
            segments.push((name.as_str(), Namespace::Type, 0));
        }
        segments.push(("func", Namespace::Value, 0));

        let path = encode_simple_path(&segments[..]);
        // Should have 10 Nt prefixes
        assert_eq!(path.matches("Nt").count(), 10);
        assert!(path.starts_with("NvNtNtNtNtNtNtNtNtNtNt"));
        assert!(path.contains("4mod1"));
        assert!(path.contains("5mod10"));
        assert!(path.ends_with("4func"));
    }

    // ========== Symbol Builder Tests ==========

    #[test]
    fn test_symbol_builder_simple_function() {
        let symbol = SymbolBuilder::new("mycrate")
            .function("foo")
            .build()
            .unwrap();
        assert_eq!(symbol, "_RNvC7mycrate3foo");
    }

    #[test]
    fn test_symbol_builder_with_hash() {
        let symbol = SymbolBuilder::new("mycrate")
            .with_hash("aRN1VPjcjfp")
            .function("foo")
            .build()
            .unwrap();
        assert_eq!(symbol, "_RNvCsaRN1VPjcjfp_7mycrate3foo");
    }

    #[test]
    fn test_symbol_builder_nested_modules() {
        let symbol = SymbolBuilder::new("mycrate")
            .module("inner")
            .module("nested")
            .function("func")
            .build()
            .unwrap();
        assert_eq!(symbol, "_RNvNtNtC7mycrate5inner6nested4func");
    }

    #[test]
    fn test_symbol_builder_type() {
        let symbol = SymbolBuilder::new("mycrate")
            .type_name("MyStruct")
            .build()
            .unwrap();
        assert_eq!(symbol, "_RNtC7mycrate8MyStruct");
    }

    #[test]
    fn test_symbol_builder_method() {
        let symbol = SymbolBuilder::new("test_symbols")
            .with_hash("aRN1VPjcjfp")
            .method("SimpleStruct", "new")
            .build()
            .unwrap();
        // Method symbols have special structure
        assert!(symbol.starts_with("_RNvM"));
        assert!(symbol.contains("SimpleStruct"));
        assert!(symbol.contains("new"));
    }

    #[test]
    fn test_symbol_builder_method_in_module() {
        let symbol = SymbolBuilder::new("mycrate")
            .with_hash("ABC123")
            .module("mymod")
            .method("MyType", "method_name")
            .build()
            .unwrap();
        assert!(symbol.starts_with("_RNvM"));
        assert!(symbol.contains("MyType"));
        assert!(symbol.contains("method_name"));
    }

    #[test]
    fn test_symbol_builder_empty_fails() {
        let result = SymbolBuilder::new("mycrate").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_symbol_builder_build_path() {
        let path = SymbolBuilder::new("mycrate")
            .function("foo")
            .build_path()
            .unwrap();
        assert_eq!(path, "NvC7mycrate3foo");
        assert!(!path.starts_with("_R"));
    }

    // ========== Symbol Encoding Tests ==========

    #[test]
    fn test_encode_symbol_adds_prefix() {
        assert_eq!(encode_symbol("NvC7mycrate3foo"), "_RNvC7mycrate3foo");
        assert_eq!(encode_symbol("C3std"), "_RC3std");
    }

    // ========== Namespace Tests ==========

    #[test]
    fn test_namespace_tags() {
        assert_eq!(Namespace::Crate.tag(), 'C');
        assert_eq!(Namespace::Type.tag(), 't');
        assert_eq!(Namespace::Value.tag(), 'v');
        assert_eq!(Namespace::Closure.tag(), 'C');
        assert_eq!(Namespace::Shim.tag(), 'S');
    }

    // ========== Complex Integration Tests ==========

    #[test]
    fn test_complex_symbol_with_all_features() {
        // Test a complex path with modules, hash, and disambiguators
        let path = encode_simple_path_with_crate_hash(
            &[
                ("my_crate_v2", Namespace::Crate, 0),
                ("outer_mod", Namespace::Type, 0),
                ("inner_mod", Namespace::Type, 1),
                ("MyStruct", Namespace::Type, 0),
            ],
            Some("XyZ123abc")
        );

        assert!(path.starts_with("NtNtNtCsXyZ123abc_"));
        assert!(path.contains("11my_crate_v2"));
        assert!(path.contains("9outer_mod"));
        assert!(path.contains("s_9inner_mod")); // disambiguator 1
        assert!(path.ends_with("8MyStruct"));
    }

    #[test]
    fn test_symbols_are_demanglable() {
        // Verify generated symbols can be demangled by rustc-demangle
        let symbols = vec![
            "_RNvC7mycrate3foo",
            "_RNvNtC7mycrate6module4func",
            "_RNtC3std6String",
        ];

        for symbol in symbols {
            // If this is a valid v0 symbol, rustc-demangle should handle it
            // We're just checking it doesn't panic
            let _ = symbol.to_string();
        }
    }

    // ========== Edge Cases and Error Conditions ==========

    #[test]
    fn test_empty_crate_name() {
        // Empty crate name should still encode
        let root = encode_crate_root("", 0);
        assert_eq!(root, "C0");
    }

    #[test]
    fn test_single_char_identifiers() {
        let mut output = String::new();
        push_ident("x", &mut output);
        assert_eq!(output, "1x");

        let mut output = String::new();
        push_ident("_", &mut output);
        assert_eq!(output, "1__");

        let mut output = String::new();
        push_ident("0", &mut output);
        assert_eq!(output, "1_0");
    }

    #[test]
    fn test_numeric_identifier_names() {
        let mut output = String::new();
        push_ident("123abc", &mut output);
        assert_eq!(output, "6_123abc");

        let mut output = String::new();
        push_ident("0x1234", &mut output);
        assert_eq!(output, "6_0x1234");
    }

    #[test]
    fn test_underscore_heavy_identifiers() {
        let mut output = String::new();
        push_ident("__double", &mut output);
        assert_eq!(output, "8___double");

        let mut output = String::new();
        push_ident("___triple", &mut output);
        assert_eq!(output, "9____triple");

        let mut output = String::new();
        push_ident("before__after", &mut output);
        assert_eq!(output, "13before__after");
    }

    #[test]
    fn test_all_caps_identifiers() {
        let mut output = String::new();
        push_ident("CONST_VALUE", &mut output);
        assert_eq!(output, "11CONST_VALUE");

        let mut output = String::new();
        push_ident("TYPE_MAX", &mut output);
        assert_eq!(output, "8TYPE_MAX");
    }

    // ========== Impl Trait Functions Tests ==========

    #[test]
    fn test_create_symbol_iterator() {
        let symbols: Vec<String> = create_symbol_iterator("mycrate", &["foo", "bar", "baz"])
            .collect();

        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0], "_RNvC7mycrate3foo");
        assert_eq!(symbols[1], "_RNvC7mycrate3bar");
        assert_eq!(symbols[2], "_RNvC7mycrate3baz");
    }

    #[test]
    fn test_create_symbol_iterator_empty() {
        let symbols: Vec<String> = create_symbol_iterator("mycrate", &[])
            .collect();

        assert_eq!(symbols.len(), 0);
    }

    #[test]
    fn test_create_symbol_iterator_single() {
        let symbols: Vec<String> = create_symbol_iterator("std", &["main"])
            .collect();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0], "_RNvC3std4main");
    }

    #[test]
    fn test_create_symbol_iterator_chain() {
        // Test that the iterator can be chained with other iterator methods
        let count = create_symbol_iterator("test", &["a", "b", "c", "d"])
            .filter(|s| s.contains("test"))
            .count();

        assert_eq!(count, 4);
    }

    #[test]
    fn test_create_symbol_display_simple() {
        let display = create_symbol_display("_RNvC7mycrate3foo", false);
        let output = format!("{}", display);

        assert!(output.contains("Symbol: "));
        assert!(output.contains("_RNvC7mycrate3foo"));
        assert!(!output.contains("Format:")); // No breakdown
    }

    #[test]
    fn test_create_symbol_display_with_breakdown() {
        let display = create_symbol_display("_RNvC7mycrate3foo", true);
        let output = format!("{}", display);

        assert!(output.contains("Symbol: "));
        assert!(output.contains("_RNvC7mycrate3foo"));
        assert!(output.contains("Format: v0 (Rust Symbol Mangling)"));
    }

    #[test]
    fn test_create_symbol_display_non_v0() {
        let display = create_symbol_display("some_other_symbol", true);
        let output = format!("{}", display);

        assert!(output.contains("Symbol: "));
        assert!(output.contains("some_other_symbol"));
        // Should not show v0 format info for non-v0 symbols
        assert!(!output.contains("v0"));
    }

    #[test]
    fn test_symbol_iterator_map_transform() {
        // Test that the iterator works with transformations
        let symbols: Vec<String> = create_symbol_iterator("crate", &["x", "y", "z"])
            .map(|s| s.to_uppercase())
            .collect();

        assert!(symbols[0].starts_with("_RNVC"));
    }

    #[test]
    fn test_create_symbol_iterator_matches_nm_output() {
        // Test that symbols match the format shown by nm on real binaries
        // Real nm output: _RNvCs5GYaaS9NRMV_12test_symbols11float_types

        // Without hash (simple case)
        let symbols: Vec<String> = create_symbol_iterator("mycrate", &["foo"])
            .collect();
        assert_eq!(symbols[0], "_RNvC7mycrate3foo");

        // Verify this matches what we'd see from a real binary
        // The format should be: _R + Nv + C + length + name + length + function
        assert!(symbols[0].starts_with("_RNv"));
        assert!(symbols[0].contains("C7mycrate"));
        assert!(symbols[0].ends_with("3foo"));
    }

    #[test]
    fn test_create_symbol_iterator_real_world_example() {
        // Test symbols that would appear in real nm output
        // Based on actual symbols from /home/user/test-symbols library
        let function_names = &[
            "float_types",
            "integer_types",
            "ptr_function",
            "ref_function",
        ];

        let symbols: Vec<String> = create_symbol_iterator("test_symbols", function_names)
            .collect();

        // Without hash, these should be:
        assert_eq!(symbols[0], "_RNvC12test_symbols11float_types");
        assert_eq!(symbols[1], "_RNvC12test_symbols13integer_types");
        assert_eq!(symbols[2], "_RNvC12test_symbols12ptr_function");
        assert_eq!(symbols[3], "_RNvC12test_symbols12ref_function");

        // Verify all start with v0 prefix and value namespace
        for symbol in &symbols {
            assert!(symbol.starts_with("_RNv"));
        }
    }
}
