use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use syn::{self, visit::Visit, visit_mut::VisitMut, Item, ItemUse};
use proc_macro2::TokenStream;
use quote::quote; // Required for quote! macro
use std::collections::{HashMap, HashSet};

// Helper to get the name of a syn::Item
fn get_item_name(item: &Item) -> Option<String> {
    match item {
        Item::Fn(item_fn) => Some(item_fn.sig.ident.to_string()),
        Item::Struct(item_struct) => Some(item_struct.ident.to_string()),
        Item::Enum(item_enum) => Some(item_enum.ident.to_string()),
        Item::Const(item_const) => Some(item_const.ident.to_string()),
        Item::Static(item_static) => Some(item_static.ident.to_string()),
        Item::Trait(item_trait) => Some(item_trait.ident.to_string()),
        Item::Type(item_type) => Some(item_type.ident.to_string()),
        Item::Union(item_union) => Some(item_union.ident.to_string()),
        Item::Macro(item_macro) => item_macro.ident.as_ref().map(|id| id.to_string()),
        // Item::Impl does not have a simple name. It implements a trait for a type, or is for a type.
        // We'll need a more sophisticated approach if impl blocks need unique file names.
        Item::Impl(_) => None, 
        _ => None,
    }
}

// Helper to get the kind of a syn::Item
fn get_item_kind(item: &Item) -> Option<&'static str> {
    match item {
        Item::Fn(_) => Some("fn"),
        Item::Struct(_) => Some("struct"),
        Item::Enum(_) => Some("enum"),
        Item::Const(_) => Some("const"),
        Item::Static(_) => Some("static"),
        Item::Trait(_) => Some("trait"),
        Item::Impl(_) => Some("impl"),
        Item::Type(_) => Some("type"),
        Item::Union(_) => Some("union"),
        Item::Macro(_) => Some("macro"),
        _ => None,
    }
}

// Visitor to collect all top-level `use` statements.
#[derive(Default)]
struct UseStatementCollector {
    uses: Vec<ItemUse>,
}

impl<'ast> Visit<'ast> for UseStatementCollector {
    fn visit_item_use(&mut self, i: &'ast ItemUse) {
        // Only collect top-level use statements (not within blocks)
        self.uses.push(i.clone());
        syn::visit::visit_item_use(self, i);
    }
}

pub fn split_lib_rs(
    lib_rs_path: &Path,
    decls_output_dir: &Path,
    crate_name: &str,
    global_config: &SplitDeclsConfig, // New parameter
    dry_run: bool,
) -> Result<()> {
    println!("Splitting {} for crate {}...", lib_rs_path.display(), crate_name);
    if dry_run {
        println!("Dry-run: Splitting operations would be performed.");
        return Ok(());
    }

    // Ensure decls directory exists
    fs::create_dir_all(decls_output_dir)
        .context(format!("Failed to create directory {}", decls_output_dir.display()))?;
    println!("Created directory: {}", decls_output_dir.display());

    let original_lib_rs_content = fs::read_to_string(lib_rs_path)
        .context(format!("Failed to read {}", lib_rs_path.display()))?;

    let mut syntax_tree = syn::parse_file(&original_lib_rs_content)
        .context(format!("Failed to parse {} content", lib_rs_path.display()))?;

    let mut collected_uses = UseStatementCollector::default();
    collected_uses.visit_file(&syntax_tree);

    let mut current_lib_rs_mods: Vec<Item> = Vec::new();
    let mut generated_decls_modules: Vec<String> = Vec::new();

    // Iterate through items and split them
    let mut new_items: Vec<Item> = Vec::new(); // Items that will remain in the main lib.rs or are mod declarations

    for item in syntax_tree.items {
        match item {
            Item::Use(_) => {
                // Top-level use statements are collected and will be added to each decl file
                // and potentially to the new lib.rs, but are removed from the original item flow.
            },
            Item::Mod(item_mod) => {
                // Keep mod declarations in the main lib.rs, but without their content
                if item_mod.content.is_some() {
                    // This is a module with inline content, we should ideally split its contents too
                    // For now, let's keep it as is, or error if we want strict top-level decls
                    eprintln!("Warning: Inline module '{}' found in {}. Its contents will not be split.", item_mod.ident, lib_rs_path.display());
                    new_items.push(Item::Mod(item_mod));
                } else {
                    current_lib_rs_mods.push(Item::Mod(item_mod));
                }
            },
            Item::Fn(_) | Item::Struct(_) | Item::Enum(_) | Item::Const(_) |
            Item::Static(_) | Item::Trait(_) | Item::Type(_) | Item::Union(_) | Item::Macro(_) => {
                if let (Some(name), Some(kind)) = (get_item_name(&item), get_item_kind(&item)) {
                    let file_name = format!("{}_{}.rs", kind, name.to_ascii_lowercase());
                    let output_file_path = decls_output_dir.join(&file_name);

                    // Add common uses to the top of the decl file
                    let mut decl_file_content = String::new();
                    for u in &collected_uses.uses {
                        decl_file_content.push_str(&quote!{#u}.to_string());
                        decl_file_content.push('\n');
                    }
                    decl_file_content.push_str("\nuse introspector_decl2_macros::introspect;\n\n"); // Ensure introspect is available

                    let wrapped_decl = quote! {
                        introspect! {
                            #item
                        }
                    }.to_string();
                    decl_file_content.push_str(&wrapped_decl);

                    fs::write(&output_file_path, decl_file_content)
                        .context(format!("Failed to write declaration to {}", output_file_path.display()))?;
                    println!("Extracted {} {} to {}", kind, name, output_file_path.display());
                    generated_decls_modules.push(name);
                } else {
                    eprintln!("Skipping unnamed or unhandled item type: {:?}", item);
                }
            },
            _ => {
                // Other items like extern blocks, etc., are kept in the main lib.rs
                new_items.push(item);
            }
        }
    }

    // Construct the new, minimalist lib.rs content
    let mut new_lib_rs_content = String::new();

    // Preserve initial comments/attributes if any, though parsing typically discards them
    // For now, let's just add the essential uses and mods.
    new_lib_rs_content.push_str("//! This file is generated by split-decls-rs. DO NOT EDIT.\n\n");
    new_lib_rs_content.push_str("use anyhow::{Context, Result};\n"); // Common dependency for generated build.rs, might be needed.
    new_lib_rs_content.push_str("use introspector_decl2_macros::prelude::*;

"); // Re-export prelude macros

    for item_mod in current_lib_rs_mods {
        new_lib_rs_content.push_str(&quote!{#item_mod}.to_string());
        new_lib_rs_content.push('\n');
    }

    // Declare the decls module
    new_lib_rs_content.push_str("pub mod decls;\n");
    new_lib_rs_content.push_str("pub use decls::*;
");

    fs::write(lib_rs_path, new_lib_rs_content)
        .context(format!("Failed to write new lib.rs to {}", lib_rs_path.display()))?;
    println!("Rewrote {} to a minimalist version.", lib_rs_path.display());

    Ok(())
}