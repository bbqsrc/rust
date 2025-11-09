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

    fn build_method_symbol(self) -> Result<String, &'static str> {
        // Method symbol format: _R + Nv + M + <impl-path> + Nt + <type-path> + <type-name> + <method-name>
        // For SimpleStruct::new: _RNvMCsaRN1VPjcjfp_12test_symbolsNtB2_12SimpleStruct3new
        // TODO: Implement backreferences properly
        // For now, we'll generate without backrefs and it will fail to match
        Err("Method encoding with backreferences not yet implemented")
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

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn test_push_ident_ascii() {
        let mut output = String::new();
        push_ident("example", &mut output);
        assert_eq!(output, "7example");

        let mut output = String::new();
        push_ident("foo", &mut output);
        assert_eq!(output, "3foo");
    }

    #[test]
    fn test_push_ident_with_separator() {
        let mut output = String::new();
        push_ident("_foo", &mut output);
        assert_eq!(output, "4__foo");

        let mut output = String::new();
        push_ident("0abc", &mut output);
        assert_eq!(output, "4_0abc");
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
    }

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
    }

    #[test]
    fn test_push_disambiguator() {
        let mut output = String::new();
        push_disambiguator(0, &mut output);
        assert_eq!(output, "");

        let mut output = String::new();
        push_disambiguator(1, &mut output);
        assert_eq!(output, "s_");
    }

    #[test]
    fn test_to_base_62() {
        assert_eq!(to_base_62(0), "0");
        assert_eq!(to_base_62(10), "a");
        assert_eq!(to_base_62(35), "z");
        assert_eq!(to_base_62(36), "A");
        assert_eq!(to_base_62(61), "Z");
        assert_eq!(to_base_62(62), "10");
    }
}
