//! Basic usage examples for the rfc2603 crate

use rfc2603::{push_disambiguator, push_ident, push_integer_62, push_opt_integer_62};

fn main() {
    println!("RFC 2603 - Rust Symbol Name Mangling v0 Examples\n");

    // Example 1: Base-62 encoding
    println!("=== Base-62 Encoding ===");
    let mut output = String::new();
    push_integer_62(0, &mut output);
    println!("encode_integer_62(0)    = {}", output);

    let mut output = String::new();
    push_integer_62(1, &mut output);
    println!("encode_integer_62(1)    = {}", output);

    let mut output = String::new();
    push_integer_62(62, &mut output);
    println!("encode_integer_62(62)   = {}", output);

    let mut output = String::new();
    push_integer_62(1000, &mut output);
    println!("encode_integer_62(1000) = {}", output);

    // Example 2: Identifier encoding
    println!("\n=== Identifier Encoding ===");
    let mut output = String::new();
    push_ident("example", &mut output);
    println!("push_ident(\"example\")  = {}", output);

    let mut output = String::new();
    push_ident("_private", &mut output);
    println!("push_ident(\"_private\") = {}", output);

    let mut output = String::new();
    push_ident("0abc", &mut output);
    println!("push_ident(\"0abc\")     = {}", output);

    // Example 3: Unicode identifiers
    println!("\n=== Unicode Identifiers (Punycode) ===");
    let mut output = String::new();
    push_ident("gödel", &mut output);
    println!("push_ident(\"gödel\")    = {}", output);

    let mut output = String::new();
    push_ident("föö", &mut output);
    println!("push_ident(\"föö\")      = {}", output);

    let mut output = String::new();
    push_ident("铁锈", &mut output);
    println!("push_ident(\"铁锈\")      = {}", output);

    // Example 4: Optional integers and disambiguators
    println!("\n=== Optional Integers ===");
    let mut output = String::new();
    push_opt_integer_62("s", 0, &mut output);
    println!("push_opt_integer_62(\"s\", 0) = \"{}\"", output);

    let mut output = String::new();
    push_opt_integer_62("s", 1, &mut output);
    println!("push_opt_integer_62(\"s\", 1) = \"{}\"", output);

    let mut output = String::new();
    push_opt_integer_62("s", 2, &mut output);
    println!("push_opt_integer_62(\"s\", 2) = \"{}\"", output);

    // Example 5: Disambiguators
    println!("\n=== Disambiguators ===");
    let mut output = String::new();
    push_disambiguator(0, &mut output);
    println!("push_disambiguator(0) = \"{}\" (no disambiguator)", output);

    let mut output = String::new();
    push_disambiguator(1, &mut output);
    println!("push_disambiguator(1) = \"{}\"", output);

    let mut output = String::new();
    push_disambiguator(100, &mut output);
    println!("push_disambiguator(100) = \"{}\"", output);

    // Example 6: Building a simple mangled path
    println!("\n=== Building a Simple Mangled Path ===");
    let mut symbol = String::from("_R"); // v0 prefix
    symbol.push('C'); // Crate root
    push_disambiguator(0xca63f166dbe9293, &mut symbol);
    push_ident("mycrate", &mut symbol);
    symbol.push('N'); // Nested path
    symbol.push('v'); // Value namespace
    symbol.push('C'); // Crate root reference
    push_disambiguator(0xca63f166dbe9293, &mut symbol);
    push_ident("mycrate", &mut symbol);
    push_ident("example", &mut symbol);
    println!("Mangled symbol: {}", symbol);
    println!("This represents: mycrate::example");
}
