use quote::ToTokens;
use syn::{self, Item};

//use crate::buildrs_ast_utils::ExtractedDecl;
use crate::ExtractedDecl;
/// Extracts a single declaration from a `syn::Item`.
/// Returns `Some(ExtractedDecl)` if the item is a supported declaration type, `None` otherwise.
pub fn extract_single_declaration(item: &Item, item_count: usize) -> Option<ExtractedDecl> {
    match item {
        Item::Fn(item_fn) => Some(ExtractedDecl {
            name: item_fn.sig.ident.to_string(),
            kind: "fn".to_string(),
            content: item_fn.to_token_stream(),
        }),
        Item::Struct(item_struct) => Some(ExtractedDecl {
            name: item_struct.ident.to_string(),
            kind: "struct".to_string(),
            content: item_struct.to_token_stream(),
        }),
        Item::Enum(item_enum) => Some(ExtractedDecl {
            name: item_enum.ident.to_string(),
            kind: "enum".to_string(),
            content: item_enum.to_token_stream(),
        }),
        Item::Const(item_const) => Some(ExtractedDecl {
            name: item_const.ident.to_string(),
            kind: "const".to_string(),
            content: item_const.to_token_stream(),
        }),
        Item::Static(item_static) => Some(ExtractedDecl {
            name: item_static.ident.to_string(),
            kind: "static".to_string(),
            content: item_static.to_token_stream(),
        }),
        Item::Trait(item_trait) => Some(ExtractedDecl {
            name: item_trait.ident.to_string(),
            kind: "trait".to_string(),
            content: item_trait.to_token_stream(),
        }),
        Item::Impl(item_impl) => {
            let name = if let Some((_, path, _)) = &item_impl.trait_ {
                let trait_name = path.to_token_stream().to_string()
                    .replace("::", "_")
                    .replace(" ", "")
                    .replace("<", "_")
                    .replace(">", "_")
                    .replace("(", "_")
                    .replace(")", "_")
                    .replace(",", "_")
                    .replace("'", "_")
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == '_')
                    .collect::<String>();
                format!("impl_for_{}", trait_name)
            } else if let syn::Type::Path(type_path) = &*item_impl.self_ty {
                if let Some(segment) = type_path.path.segments.last() {
                    format!("impl_for_{}", segment.ident.to_string())
                } else {
                    format!("impl_{}", item_count)
                }
            } else {
                format!("impl_{}", item_count)
            };
            Some(ExtractedDecl {
                name,
                kind: "impl".to_string(),
                content: item_impl.to_token_stream(),
            })
        },
        Item::Type(item_type) => Some(ExtractedDecl {
            name: item_type.ident.to_string(),
            kind: "type".to_string(),
            content: item_type.to_token_stream(),
        }),
        Item::Union(item_union) => Some(ExtractedDecl {
            name: item_union.ident.to_string(),
            kind: "union".to_string(),
            content: item_union.to_token_stream(),
        }),
        Item::Use(item_use) => {
            println!("Skipping top-level use statement in splitting: {}", item_use.to_token_stream());
            None
        },
        Item::Macro(item_macro) => {
            let macro_name = item_macro.mac.path.segments.last()
                .map_or("unknown_macro".to_string(), |s| s.ident.to_string());
            Some(ExtractedDecl {
                name: macro_name,
                kind: "macro".to_string(),
                content: item_macro.to_token_stream(),
            })
        },
        Item::Mod(item_mod) => {
            let mod_name = item_mod.ident.to_string();
            println!("Processing module: {}", mod_name);
            
            // Return a placeholder declaration for the module itself
            Some(ExtractedDecl {
                name: mod_name,
                content: item.to_token_stream(),
                kind: "module".to_string(),
            })
        }
        _ => {
            println!("Skipping unsupported item type: {}", item.to_token_stream());
            None
        }
    }
}
