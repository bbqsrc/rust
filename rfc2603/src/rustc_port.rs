//! Direct port of rustc's v0 symbol mangling implementation
//!
//! This is copied from compiler/rustc_symbol_mangling/src/v0.rs
//! with rustc internal types replaced by facet Shape.

use std::collections::HashMap;
use std::ops::Range;
use std::fmt;
use facet::Shape;

/// V0 Symbol Mangler - 1:1 port from rustc
///
/// This is the main mangler struct copied from rustc_symbol_mangling/src/v0.rs
/// Original struct: `struct V0SymbolMangler<'tcx>`
///
/// Key changes from rustc:
/// - `TyCtxt<'tcx>` → removed (facet Shape is self-contained)
/// - `Ty<'tcx>` → `&'static Shape`
/// - `DefId` → `DefId` (custom type below)
/// - `GenericArg<'tcx>` → `GenericArg` (custom type below)
pub struct V0SymbolMangler {
    /// Binder level tracking for lifetimes
    /// Copied from rustc's BinderLevel
    binders: Vec<BinderLevel>,

    /// Output string being built
    pub out: String,

    /// Whether this symbol is exportable (affects disambiguators)
    is_exportable: bool,

    /// The length of the prefix in `out` (e.g. 2 for `_R`).
    start_offset: usize,

    /// Cache of (DefId, generic args) -> position for backreferences
    /// Maps to byte positions in `out`
    paths: HashMap<(DefId, Vec<GenericArg>), usize>,

    /// Cache of shapes -> position for backreferences
    types: HashMap<ShapeKey, usize>,

    /// Cache of consts -> position for backreferences
    consts: HashMap<ConstValue, usize>,
}

/// Binder level tracking for lifetimes
/// Copied directly from rustc
struct BinderLevel {
    /// The range of distances from the root of what's
    /// being printed, to the lifetimes in a binder.
    lifetime_depths: Range<u32>,
}

/// Definition ID - identifies a unique item in the program
/// Replaces rustc's DefId
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DefId {
    pub krate: u32,
    pub index: u32,
}

/// Generic argument - can be a type, const, or lifetime
/// Replaces rustc's GenericArg<'tcx>
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum GenericArg {
    Type(&'static Shape),
    Const(ConstValue),
    Lifetime(Lifetime),
}

/// Constant value
/// Replaces rustc's ty::Const<'tcx>
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ConstValue {
    // Simplified - rustc has complex const evaluation
    pub value: u64,
}

/// Lifetime/region
/// Replaces rustc's ty::Region<'tcx>
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lifetime {
    Erased,
    Bound { debruijn: usize, var: u32 },
}

/// Shape key for hashmap lookups
/// Since Shape is complex, we use the type ID as key
#[derive(Clone, PartialEq, Eq, Hash)]
struct ShapeKey {
    id: facet::ConstTypeId,
}

impl From<&'static Shape> for ShapeKey {
    fn from(shape: &'static Shape) -> Self {
        ShapeKey { id: shape.id }
    }
}

/// Print error type
/// Replaces rustc's PrintError
#[derive(Debug)]
pub struct PrintError;

impl fmt::Display for PrintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Print error")
    }
}

impl std::error::Error for PrintError {}

impl V0SymbolMangler {
    /// Create a new V0 symbol mangler
    /// Copied from rustc
    pub fn new() -> Self {
        let prefix = "_R";
        Self {
            binders: Vec::new(),
            out: String::from(prefix),
            is_exportable: false,
            start_offset: prefix.len(),
            paths: HashMap::new(),
            types: HashMap::new(),
            consts: HashMap::new(),
        }
    }

    /// Push a string to output
    /// Copied from rustc
    fn push(&mut self, s: &str) {
        self.out.push_str(s);
    }

    /// Push a `_`-terminated base 62 integer
    /// Copied from rustc
    fn push_integer_62(&mut self, x: u64) {
        crate::push_integer_62(x, &mut self.out)
    }

    /// Push a `tag`-prefixed base 62 integer
    /// Copied from rustc
    fn push_opt_integer_62(&mut self, tag: &str, x: u64) {
        if let Some(x) = x.checked_sub(1) {
            self.push(tag);
            self.push_integer_62(x);
        }
    }

    /// Push a disambiguator using the `s` tag
    /// Copied from rustc
    fn push_disambiguator(&mut self, dis: u64) {
        self.push_opt_integer_62("s", dis);
    }

    /// Push an identifier
    /// Copied from rustc
    fn push_ident(&mut self, ident: &str) {
        crate::push_ident(ident, &mut self.out)
    }

    /// Append a path component with namespace
    /// Copied from rustc's path_append_ns
    fn path_append_ns(
        &mut self,
        print_prefix: impl FnOnce(&mut Self) -> Result<(), PrintError>,
        ns: char,
        disambiguator: u64,
        name: &str,
    ) -> Result<(), PrintError> {
        self.push("N");
        self.out.push(ns);
        print_prefix(self)?;
        self.push_disambiguator(disambiguator);
        self.push_ident(name);
        Ok(())
    }

    /// Print a backref
    /// Copied from rustc
    fn print_backref(&mut self, i: usize) -> Result<(), PrintError> {
        self.push("B");
        self.push_integer_62((i - self.start_offset) as u64);
        Ok(())
    }

    /// Print a definition path with generic arguments
    /// Copied from rustc's Printer::print_def_path
    pub fn print_def_path(&mut self, def_id: DefId, args: &[GenericArg]) -> Result<(), PrintError> {
        // Check backref cache
        let key = (def_id, args.to_vec());
        if let Some(&i) = self.paths.get(&key) {
            return self.print_backref(i);
        }
        let start = self.out.len();

        // Default path printing (simplified - rustc has complex logic here)
        self.default_print_def_path(def_id, args)?;

        // Cache the path
        self.paths.insert(key, start);
        Ok(())
    }

    /// Default path printing logic
    /// Simplified version of rustc's default_print_def_path
    fn default_print_def_path(&mut self, _def_id: DefId, _args: &[GenericArg]) -> Result<(), PrintError> {
        // TODO: implement based on DefPath data structure
        // In rustc, this walks the def_path and prints each component
        // For facet, we'd need to store path information in a separate registry
        Ok(())
    }

    /// Print a type using facet Shape
    /// Copied from rustc's print_type, adapted for facet
    pub fn print_type(&mut self, shape: &'static Shape) -> Result<(), PrintError> {
        use facet::{Type, PrimitiveType, NumericType, TextualType, SequenceType, UserType, PointerType};

        // Get the size from layout if available
        let size = shape.layout.sized_layout().ok().map(|l| l.size());

        // Basic types, never cached (single-character).
        let basic_type = match shape.ty {
            Type::Primitive(PrimitiveType::Boolean) => "b",
            Type::Primitive(PrimitiveType::Textual(TextualType::Char)) => "c",
            Type::Primitive(PrimitiveType::Textual(TextualType::Str)) => "e",
            Type::Primitive(PrimitiveType::Numeric(NumericType::Integer { signed: true })) => {
                // Determine size from layout
                match size {
                    Some(1) => "a",   // i8
                    Some(2) => "s",   // i16
                    Some(4) => "l",   // i32
                    Some(8) => "x",   // i64
                    Some(16) => "n",  // i128
                    _ => "i",         // isize (pointer-sized)
                }
            }
            Type::Primitive(PrimitiveType::Numeric(NumericType::Integer { signed: false })) => {
                // Determine size from layout
                match size {
                    Some(1) => "h",   // u8
                    Some(2) => "t",   // u16
                    Some(4) => "m",   // u32
                    Some(8) => "y",   // u64
                    Some(16) => "o",  // u128
                    _ => "j",         // usize (pointer-sized)
                }
            }
            Type::Primitive(PrimitiveType::Numeric(NumericType::Float)) => {
                match size {
                    Some(4) => "f",   // f32
                    Some(8) => "d",   // f64
                    _ => "",          // f16/f128 not yet handled
                }
            }
            Type::Primitive(PrimitiveType::Never) => "z",
            // Unit type (empty tuple)
            Type::User(UserType::Struct(struct_type)) if struct_type.fields.is_empty() => "u",
            _ => "",
        };

        if !basic_type.is_empty() {
            self.push(basic_type);
            return Ok(());
        }

        // Check type cache for backrefs
        let key = ShapeKey::from(shape);
        if let Some(&i) = self.types.get(&key) {
            return self.print_backref(i);
        }

        let start = self.out.len();

        // Complex types
        match shape.ty {
            Type::Pointer(PointerType::Reference(ref_type)) => {
                self.push(if ref_type.mutable { "Q" } else { "R" });
                // Lifetime (simplified - facet doesn't track lifetimes in the same way)
                // We'd need additional metadata for full lifetime support
                // For now, assume erased lifetime
                self.print_type(ref_type.target)?;
            }

            Type::Pointer(PointerType::Raw(ptr_type)) => {
                self.push(if ptr_type.mutable { "O" } else { "P" });
                self.print_type(ptr_type.target)?;
            }

            Type::Sequence(SequenceType::Array(array_type)) => {
                self.push("A");
                self.print_type(array_type.t)?;
                // Array length as const
                self.print_const(&ConstValue { value: array_type.n as u64 })?;
            }

            Type::Sequence(SequenceType::Slice(slice_type)) => {
                self.push("S");
                self.print_type(slice_type.t)?;
            }

            Type::User(UserType::Struct(struct_type)) if matches!(struct_type.kind, facet::StructKind::Tuple) => {
                self.push("T");
                for field in struct_type.fields {
                    self.print_type(field.shape())?;
                }
                self.push("E");
            }

            // Nominal types (ADTs, functions, etc.) would use print_def_path
            // But facet doesn't directly provide DefId - we'd need to build that separately
            Type::User(_) => {
                // For user types, we'd need to construct a DefId from the type_identifier
                // and call print_def_path. This requires additional infrastructure.
                // For now, this is a placeholder.
                self.push(shape.type_identifier);
            }

            _ => {
                // Other types not yet implemented
            }
        }

        // Cache this type
        self.types.insert(key, start);

        Ok(())
    }

    /// Print a lifetime
    /// Copied from rustc's print_region
    fn print_lifetime(&mut self, lifetime: Lifetime) -> Result<(), PrintError> {
        let i = match lifetime {
            Lifetime::Erased => 0,
            Lifetime::Bound { debruijn, var } => {
                let binder = &self.binders[self.binders.len() - 1 - debruijn];
                let depth = binder.lifetime_depths.start + var;
                1 + (self.binders.last().unwrap().lifetime_depths.end - 1 - depth)
            }
        };
        self.push("L");
        self.push_integer_62(i as u64);
        Ok(())
    }

    /// Print a const value
    /// Simplified from rustc's print_const
    fn print_const(&mut self, const_val: &ConstValue) -> Result<(), PrintError> {
        // Check const cache
        if let Some(&i) = self.consts.get(const_val) {
            return self.print_backref(i);
        }

        let start = self.out.len();

        // Simplified - rustc has complex const value printing
        self.push("K");
        self.push_integer_62(const_val.value);

        self.consts.insert(const_val.clone(), start);
        Ok(())
    }
}

impl Default for V0SymbolMangler {
    fn default() -> Self {
        Self::new()
    }
}
