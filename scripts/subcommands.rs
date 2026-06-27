#!/usr/bin/env -S cargo +nightly -Zscript
---
[dependencies]
syn = { version = "2", features = ["full"] }
---
use syn::{File, Item};
fn main() {
    let src = std::fs::read_to_string("src/main.rs").unwrap();
    let ast: File = syn::parse_str(&src).unwrap();
    let names: Vec<String> = ast
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Enum(e) if e.ident == "Commands" => Some(e),
            _ => None,
        })
        .flat_map(|e| e.variants.iter())
        .map(|v| v.ident.to_string().to_lowercase())
        .collect();
    println!(
        "[{}]",
        names
            .iter()
            .map(|n| format!("\"{n}\""))
            .collect::<Vec<_>>()
            .join(",")
    );
}
