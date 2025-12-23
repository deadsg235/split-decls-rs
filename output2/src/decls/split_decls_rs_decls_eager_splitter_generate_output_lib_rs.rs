 use serde :: { Deserialize , Serialize } ; use std :: collections :: HashMap ; # [doc = " Generate new lib.rs that re-exports the split declarations in the output directory"] fn generate_output_lib_rs (paths : & CratePaths) -> Result < () > { let output_lib_path = paths . decls_output_dir . parent () . unwrap () . join ("lib.rs") ; let new_lib_content = format ! (r#"
// Re-exports all split declarations

pub mod decls {{
    include!("decls/_decl_module_invocation.rs");
}}
pub use decls::*;

// Re-export prelude macros if available

pub use introspector_decl2_macros::*;
"#) ; fs :: create_dir_all (output_lib_path . parent () . unwrap ()) . context ("Failed to create output src directory") ? ; add_generated_rust_header ! (& output_lib_path , new_lib_content . as_str () , file ! () , line ! ()) ? ; println ! ("Generated output lib.rs at {}" , output_lib_path . display ()) ; Ok (()) }