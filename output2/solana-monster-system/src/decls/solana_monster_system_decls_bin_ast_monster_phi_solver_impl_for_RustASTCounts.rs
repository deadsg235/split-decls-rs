pub use decls :: * ; # [cfg (feature = "introspector_decl2_macros")] pub use introspector_decl2_macros :: * ; prelude ! { } # [decl_ solana_monster_system_decls_bin_ast_monster_phi_solver_impl_for_RustASTCounts] impl RustASTCounts { fn analyze_rust_code (code : & str) -> Self { Self { struct_count : code . matches ("struct ") . count () as u32 , enum_count : code . matches ("enum ") . count () as u32 , fn_count : code . matches ("fn ") . count () as u32 , impl_count : code . matches ("impl ") . count () as u32 , trait_count : code . matches ("trait ") . count () as u32 , macro_count : code . matches ("macro_rules!") . count () as u32 , mod_count : code . matches ("mod ") . count () as u32 , use_count : code . matches ("use ") . count () as u32 , const_count : code . matches ("const ") . count () as u32 , static_count : code . matches ("static ") . count () as u32 , } } fn generate_minizinc_data (& self) -> String { format ! (r#"
% Rust AST data for Monster Group Phi mapping
struct_count = {};
enum_count = {};
fn_count = {};
impl_count = {};
trait_count = {};
macro_count = {};
mod_count = {};
use_count = {};
const_count = {};
static_count = {};
"# , self . struct_count , self . enum_count , self . fn_count , self . impl_count , self . trait_count , self . macro_count , self . mod_count , self . use_count , self . const_count , self . static_count) } }