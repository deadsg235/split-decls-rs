use std::{{{{
    collections::HashMap, // Added for HashMap in SplitDeclsConfig
    collections::HashSet,
    fs,
    path::{{{{Path, PathBuf}}}},
}}}}};

use anyhow::{{{{Context, Result}}}};
use proc_macro2::{{{{Span, TokenStream}}}};
use quote::{{{{quote, ToTokens}}}};
use syn::{{{{
    parse_quote,
    punctuated::Punctuated,
    visit::Visit,
    Ident,
    Item,
    ItemConst,
    ItemEnum,
    ItemFn,
    ItemImpl,
    ItemMacro,
    ItemMod,
    ItemStatic,
    ItemStruct,
    ItemTrait,
    ItemType,
    ItemUnion,
    ItemUse,
    Visibility,
}}}}};
use toml; // Added for config parsing
use serde::{{{{Deserialize, Serialize}}}}; // Added for config parsing

/// Defines a single patch file and an optional Git reference for context.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PatchSpec {{
    /// Path to the patch .rs file, relative to the workspace root.
    pub path: PathBuf,
    /// Optional Git reference (branch, tag, commit hash) associated with this patch.
    /// This indicates the state of the repository for which this patch is relevant.
    pub git_reference: Option<String>,
}}

/// Configuration for split-decls-rs, defined here for the generated build.rs.
#[derive(Debug, Default, Serialize, Deserialize, Clone)] // Added Clone
pub struct SplitDeclsConfig {{
    pub active_overlay_modules: Vec<String>,
    pub custom_prelude_overlay: Option<String>,
    pub rustc_source_path: Option<PathBuf>, // Added
    pub patches: HashMap<String, Vec<PatchSpec>>, // Modified to use PatchSpec
    pub string_replacements: Option<Vec<StringReplacement>>, // Added
}}

impl SplitDeclsConfig {{
    pub fn load_from_file(path: &Path) -> anyhow::Result<Self> {{
        if !path.exists() {{
            // If the config file doesn't exist, use default and don't error
            return Ok(Self::default());
        }}
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }}
}}

/// Defines a single string replacement operation. (Copied from src/config.rs)
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct StringReplacement {{
    pub old: String,
    pub new: String,
}}

// Define GetToken! macro
macro_rules! GetToken {{{{
    ($token:tt) => {{{{ syn::token::$token::new(proc_macro2::Span::call_site()) }}}};
}}}}

// Define mkImplCallVisitor! macro
macro_rules! mkImplCallVisitor {{{{
    (calls: $init_calls:expr) => {{{{ struct ImplCallVisitor {{ calls: std::collections::HashMap<String, std::collections::HashSet<String>> }} impl ImplCallVisitor {{ fn new() -> Self {{ ImplCallVisitor {{ calls: $init_calls }} }} }} impl<'ast> syn::visit::Visit<'ast> for ImplCallVisitor {{ fn visit_macro(&mut self, i: &'ast syn::Macro) {{ if let Some(path_segment) = i.path.segments.last() {{ let path_str = path_segment.ident.to_string(); if path_str.ends_with("_impl") {{ if let Some(module_ident) = i.path.segments.first() {{ let module_name = module_ident.ident.to_string(); let fn_name = path_str; self.calls.entry(module_name).or_default().insert(fn_name); }} }} }} syn::visit::visit_macro(self, i); }} }} }} }} // mkImplCallVisitor!


/// Represents a single extracted declaration.
struct ExtractedDecl {{
    name: String,
    kind: String, // e.g., "fn", "struct", "enum"
    content: TokenStream,
}}

/// Visitor to collect all top-level `use` statements.
#[derive(Default)]
struct UseStatementCollector {{
    uses: Vec<ItemUse>,
}}

impl<'ast> Visit<'ast> for UseStatementCollector {{
    fn visit_item_use(&mut self, i: &'ast ItemUse) {{
        self.uses.push(i.clone());
        syn::visit::visit_item_use(self, i);
    }}
}}


fn main() -> Result<()> {{{{
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={{0}}"); // oldlib.rs
    println!("cargo:rerun-if-changed={{1}}"); // oldbuild.rs (though unused for now)
    // Watch for changes in the generated config file
    println!("cargo:rerun-if-changed=.split-decls-config.toml");

    // Load configuration for this crate
    let config_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?)
        .join(".split-decls-config.toml");
    let config = SplitDeclsConfig::load_from_file(&config_path)
        .context(format!("Failed to load config from {}", config_path.display()))?;
    println!("Loaded config for this crate: {:?}", config);
    
    fs::create_dir_all("{{2}}").context("Failed to create decls directory")?;

    let mut old_lib_rs_content = fs::read_to_string("{{0}}")
        .context("Failed to read oldlib.rs content")?;

    if let Some(replacements) = &config.string_replacements {{
        for sr in replacements {{
            old_lib_rs_content = old_lib_rs_content.replace(&sr.old, &sr.new);
            println!("Applied string replacement: '{}' -> '{}'", sr.old, sr.new);
        }}
    }}

    let mut syntax_tree = syn::parse_file(&old_lib_rs_content)
        .context("Failed to parse oldlib.rs content")?;

    // Identify patches for the current crate and apply them
    let current_crate_name_for_patch = "{{3}}".to_string(); // Crate name from format! arg
    let mut patch_items: Vec<Item> = Vec::new();

    if let Some(patches_for_crate) = config.patches.get(&current_crate_name_for_patch) {{
        for patch_spec in patches_for_crate {{
            let patch_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?).join(&patch_spec.path);
            let patch_content = fs::read_to_string(&patch_path)
                .context(format!("Failed to read patch file {}", patch_path.display()))?;
            let patch_ast = syn::parse_file(&patch_content)
                .context(format!("Failed to parse patch file {} AST", patch_path.display()))?;
            patch_items.extend(patch_ast.items);
            println!("Loaded patch: {}", patch_path.display());
            // TODO: Handle git_reference for conditional patch application if needed
        }}
    }}

    // Apply patches to the syntax_tree
    for p_item in patch_items {{
        let mut replaced = false;
        for i in 0..syntax_tree.items.len() {{
            let s_item = &mut syntax_tree.items[i];
            // Simple name-based replacement. This needs to be more robust for real-world use.
            if let (Some(s_name), Some(p_name), Some(s_kind), Some(p_kind)) = (get_item_name(s_item), get_item_name(&p_item), get_item_kind(s_item), get_item_kind(&p_item)) {{
                if s_name == p_name && s_kind == p_kind {{
                    *s_item = p_item.clone(); // Replace
                    replaced = true;
                    println!("Applied patch: Replaced item {:?}", p_name);
                    break;
                }}
            }}
        }}
        if !replaced {{
            syntax_tree.items.push(p_item); // Add new item
            println!("Applied patch: Added new item {:?}", get_item_name(&p_item));
        }}
    }}
    
    // --- Helper functions for item name/kind (defined locally for the generated build.rs) ---
    fn get_item_name(item: &Item) -> Option<String> {{
        match item {{
            Item::Fn(item_fn) => Some(item_fn.sig.ident.to_string()),
            Item::Struct(item_struct) => Some(item_struct.ident.to_string()),
            Item::Enum(item_enum) => Some(item_enum.ident.to_string()),
            Item::Const(item_const) => Some(item_const.ident.to_string()),
            Item::Static(item_static) => Some(item_static.ident.to_string()),
            Item::Trait(item_trait) => Some(item_trait.ident.to_string()),
            Item::Type(item_type) => Some(item_type.ident.to_string()),
            Item::Union(item_union) => Some(item_union.ident.to_string()),
            _ => None,
        }}
    }}

    fn get_item_kind(item: &Item) -> Option<&'static str> {{
        match item {{
            Item::Fn(_) => Some("fn"),
            Item::Struct(_) => Some("struct"),
            Item::Enum(_) => Some("enum"),
            Item::Const(_) => Some("const"),
            Item::Static(_) => Some("static"),
            Item::Trait(_) => Some("trait"),
            Item::Impl(_) => Some("impl"), // Impl names are complex; this would need refinement
            Item::Type(_) => Some("type"),
            Item::Union(_) => Some("union"),
            _ => None,
        }}
    }}
    // ---------------------------------------------------------------------------------------------------------

    // Collect use statements
    let mut use_collector = UseStatementCollector::default();
    use_collector.visit_file(&syntax_tree); // Visit the patched syntax_tree
    let common_uses = quote! {{{{ #(#(#use_collector.uses)*)* }}}};


    let mut collected_module_names: Vec<Ident> = Vec::new();
    let mut item_count = 0;

    // Extract and split declarations
    for item in syntax_tree.items {{
        let extracted_decl = match item {{
            Item::Fn(item_fn) => Some(ExtractedDecl {{
                name: item_fn.sig.ident.to_string(),
                kind: "fn".to_string(),
                content: item_fn.into_token_stream(),
            }}),
            Item::Struct(item_struct) => Some(ExtractedDecl {{
                name: item_struct.ident.to_string(),
                kind: "struct".to_string(),
                content: item_struct.into_token_stream(),
            }}),
            Item::Enum(item_enum) => Some(ExtractedDecl {{
                name: item_enum.ident.to_string(),
                kind: "enum".to_string(),
                content: item_enum.into_token_stream(),
            }}),
            Item::Const(item_const) => Some(ExtractedDecl {{
                name: item_const.ident.to_string(),
                kind: "const".to_string(),
                content: item_const.into_token_stream(),
            }}),
            Item::Static(item_static) => Some(ExtractedDecl {{
                name: item_static.ident.to_string(),
                kind: "static".to_string(),
                content: item_static.into_token_stream(),
            }}),
            Item::Trait(item_trait) => Some(ExtractedDecl {{
                name: item_trait.ident.to_string(),
                kind: "trait".to_string(),
                content: item_trait.into_token_stream(),
            }}),
            Item::Impl(item_impl) => {{
                let name = if let Some((_, path, _)) = item_impl.trait_ {{
                    format!("impl_for_{{}}", path.to_token_stream().to_string().replace("::", "_"))
                }} else if let syn::Type::Path(type_path) = *item_impl.self_ty {{
                    if let Some(segment) = type_path.path.segments.last() {{
                        format!("impl_for_{{}}", segment.ident.to_string())
                    }} else {{
                        format!("impl_{{}}", item_count)
                    }}
                }} else {{
                    format!("impl_{{}}", item_count)
                }};
                Some(ExtractedDecl {{
                    name,
                    kind: "impl".to_string(),
                    content: item_impl.into_token_stream(),
                }})
            }},
            Item::Type(item_type) => Some(ExtractedDecl {{
                name: item_type.ident.to_string(),
                kind: "type".to_string(),
                content: item_type.into_token_stream(),
            }}),
            Item::Union(item_union) => Some(ExtractedDecl {{
                name: item_union.ident.to_string(),
                kind: "union".to_string(),
                content: item_union.into_token_stream(),
            }}),
            Item::Use(item_use) => {{
                println!("Skipping top-level use statement in splitting: {{}}", item_use.into_token_stream());
                None
            }},
            Item::Macro(item_macro) => {{
                println!("Including top-level macro invocation: {{}}", item_macro.mac.path.to_token_stream());
                None
            }},
            Item::Mod(item_mod) => {{
                println!("Skipping top-level module: {{}}", item_mod.ident);
                None
            }}
            _ => {{
                println!("Skipping unsupported item type: {{:?}}", item.to_token_stream());
                None
            }}
        }};

        if let Some(decl) = extracted_decl {{
            let module_name_str = format!("{{3}}_decls_{{}}", decl.name);
            let module_name_ident = Ident::new(&module_name_str, Span::call_site());
            collected_module_names.push(module_name_ident.clone());

            let decl_file_path = PathBuf::from("{{2}}").join(format!("{{}}.rs", module_name_str));
            let custom_prelude = if let Some(ref prelude_str) = config.custom_prelude_overlay {{
                prelude_str.parse::<TokenStream>().context("Failed to parse custom prelude overlay")?
            }} else {{
                quote!{{}}
            }};

            let file_content = quote! {{{{
                #custom_prelude
                #common_uses
                prelude! {{{{}}}} // Placeholder for the actual prelude! macro
                #[decl_ #module_name_ident] // Placeholder for the specific decl macro
                #decl.content
            }}}}};
            fs::write(&decl_file_path, file_content.to_string())
                .context(format!("Failed to write to {{}}", decl_file_path.display()))?;
            println!("Split '{{}} {{}}' to {{}}", decl.kind, decl.name, decl_file_path.display());
        }}
        item_count += 1;
    }}
    
    // Generate decl_module! invocation
    use introspector_decl2_macros::decl_module;
    let decl_module_invocation_args = Punctuated::<Ident, syn::token::Comma>::from_iter(collected_module_names.into_iter());

    // Write the decl_module! invocation to a temporary file that is then included,
    // or generate a final rust file that holds the entire processed module.
    // For now, let's just make sure the `decl_module!` invocation is correct.
    let final_decl_module_code = quote! {{{{
        decl_module!({{{{decl_module_invocation_args}}}});
    }}}}};
    fs::write(PathBuf::from("{{2}}").join("_decl_module_invocation.rs"), final_decl_module_code.to_string())
        .context("Failed to write _decl_module_invocation.rs")?;

    Ok(())
}}}}