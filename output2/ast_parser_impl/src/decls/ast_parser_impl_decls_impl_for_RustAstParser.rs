use anyhow :: { Context , Result } ; use std :: collections :: HashSet ; use std :: fs ; use std :: io :: Write ; use std :: path :: { Path , PathBuf } ; use lazy_static :: lazy_static ; use monster_math_traits :: { Declaration , RustAstParser } ; use once_cell :: sync :: Lazy ; use prelude_generator :: use_extractor :: expand_macros_and_parse ; use prelude_generator :: use_extractor :: rustc_info :: get_rustc_info ; use quote :: ToTokens ; use regex :: Regex ; use split_expanded_lib :: { ErrorSample , RustcInfo as SplitExpandedRustcInfo } ; use syn :: { self , visit :: Visit , ItemConst , ItemEnum , ItemFn , ItemMod , ItemStatic , ItemStruct , ItemTrait , ItemType , ItemUnion , } ; use tokio :: runtime :: Runtime ; prelude ! { } # [decl_ ast_parser_impl_decls_impl_for_RustAstParser] impl RustAstParser for RealRustAstParser { fn parse_rust_code (& self , code : & str) -> Vec < Declaration > { TOKIO_RUNTIME . block_on (async { let temp_dir = tempfile :: tempdir () . expect ("Failed to create temporary directory") ; let temp_crate_path = temp_dir . path () ; let temp_src_dir = temp_crate_path . join ("src") ; fs :: create_dir_all (& temp_src_dir) . expect ("Failed to create temporary src directory") ; let lib_rs_path = temp_src_dir . join ("lib.rs") ; fs :: write (& lib_rs_path , code) . expect ("Failed to write code to temporary lib.rs") ; let cargo_toml_path = temp_crate_path . join ("Cargo.toml") ; fs :: write (& cargo_toml_path , r#"[package]
name = "temp_crate"
version = "0.1.0"
edition = "2021"

[lib]
path = "src/lib.rs"

[dependencies]
anyhow = "1.0"
tokio = { version = "1", features = ["full"] }
syn = { version = "2.0", features = ["full", "extra-traits"] }
quote = "1.0"
proc-macro2 = { version = "1.0", features = ["span-locations"] }
serde = { version = "1.0", features = ["derive"] }
lazy_static = "1.4.0"
once_cell = "1.19.0"
regex = "1"
split-expanded-lib = { path = "../../tools/rust-bootstrap-nix/split-expanded-lib" }
prelude-generator = { path = "../../tools/rust-bootstrap-nix/prelude-generator" }
"# ,) . expect ("Failed to write Cargo.toml") ; let rustc_info = get_rustc_info () . expect ("Failed to get rustc info") ; let cache_dir = temp_crate_path . join (".prelude_cache") ; fs :: create_dir_all (& cache_dir) . expect ("Failed to create cache directory") ; let mut writer = Vec :: new () ; let (syn_file , error_sample) = expand_macros_and_parse (& mut writer , & lib_rs_path , temp_crate_path , & cargo_toml_path , & rustc_info , & cache_dir ,) . await . expect ("Failed to expand macros and parse code") ; if let Some (error) = error_sample { eprintln ! ("Error during macro expansion: {:?}" , error) ; return Vec :: new () ; } RealRustAstParser :: extract_declarations_from_syn_file (& syn_file , & lib_rs_path) }) } }