use anyhow::{Context, Result};
use proc_macro2::TokenStream;
use quote::ToTokens; // Added
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use syn::{
    visit::{self, Visit},
    visit_mut::{self, VisitMut},
    Item, ItemUse,
};

use split_decls_types::SplitDeclsConfig;

/// Represents a single extracted declaration.
pub struct ExtractedDecl {
    pub name: String,
    pub kind: String, // e.g., "fn", "struct", "enum"
    pub content: TokenStream,
}

/// Visitor to collect all top-level `use` statements.
#[derive(Default)]
pub struct UseStatementCollector {
    pub uses: Vec<ItemUse>,
}

impl<'ast> Visit<'ast> for UseStatementCollector {
    fn visit_item_use(&mut self, i: &'ast ItemUse) {
        self.uses.push(i.clone());
        visit::visit_item_use(self, i); // Corrected recursive call
    }
}

// --- Helper functions for item name/kind ---
pub fn get_item_name(item: &Item) -> Option<String> {
    match item {
        Item::Fn(item_fn) => Some(item_fn.sig.ident.to_string()),
        Item::Struct(item_struct) => Some(item_struct.ident.to_string()),
        Item::Enum(item_enum) => Some(item_enum.ident.to_string()),
        Item::Const(item_const) => Some(item_const.ident.to_string()),
        Item::Static(item_static) => Some(item_static.ident.to_string()),
        Item::Trait(item_trait) => Some(item_trait.ident.to_string()),
        Item::Type(item_type) => Some(item_type.ident.to_string()),
        Item::Union(item_union) => Some(item_union.ident.to_string()),
        Item::Impl(item_impl) => {
            // Impl names are complex; this needs refinement to be robust.
            // For now, let's try to get a name based on the self type or trait.
            if let Some((_, path, _)) = &item_impl.trait_ {
                Some(format!("impl_for_{}", path.to_token_stream().to_string().replace("::", "_")))
            } else if let syn::Type::Path(type_path) = &*item_impl.self_ty {
                type_path.path.segments.last().map(|s| format!("impl_for_{}", s.ident.to_string()))
            } else {
                None // Cannot determine a simple name for this impl
            }
        },
        _ => None,
    }
}

pub fn get_item_kind(item: &Item) -> Option<&'static str> {
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
        _ => None,
    }
}

/// Applies AST patches to a given syntax tree.
pub fn apply_patches_to_syntax_tree(
    syntax_tree: &mut syn::File,
    crate_name_sanitized: &str,
    global_config: &SplitDeclsConfig,
) -> Result<()> {
    let mut patch_items: Vec<Item> = Vec::new();

    if let Some(patches_for_crate) = global_config.patches.get(crate_name_sanitized) {
        for patch_spec in patches_for_crate {
            let patch_path = PathBuf::from(&patch_spec.path); // Assuming patch_path is absolute or relative to CWD
            let patch_content = fs::read_to_string(&patch_path)
                .context(format!("Failed to read patch file {}", patch_path.display()))?;
            let patch_ast = syn::parse_file(&patch_content)
                .context(format!("Failed to parse patch file {} AST", patch_path.display()))?;
            patch_items.extend(patch_ast.items);
            println!("Loaded patch: {}", patch_path.display());
            // TODO: Handle git_reference for conditional patch application if needed
        }
    }

    struct PatchVisitor {
        patch_items: HashMap<String, Item>,
    }

    impl<'ast> VisitMut for PatchVisitor {
        fn visit_item_mut(&mut self, i: &mut Item) {
            if let Some(s_name) = get_item_name(i) {
                if let Some(p_item) = self.patch_items.get(&s_name) {
                    if get_item_kind(i) == get_item_kind(p_item) {
                        *i = p_item.clone();
                        println!("Applied patch: Replaced item {:?}", s_name);
                    }
                }
            }
            visit_mut::visit_item_mut(self, i);
        }
    }

    let mut patch_map: HashMap<String, Item> = HashMap::new();
    for p_item in patch_items {
        if let Some(name) = get_item_name(&p_item) {
            patch_map.insert(name, p_item);
        }
    }

    let mut visitor = PatchVisitor { patch_items: patch_map };
    visit_mut::visit_file_mut(&mut visitor, syntax_tree);

    Ok(())
}
