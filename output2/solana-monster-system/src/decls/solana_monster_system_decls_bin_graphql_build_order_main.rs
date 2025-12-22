pub use decls :: * ; # [cfg (feature = "introspector_decl2_macros")] pub use introspector_decl2_macros :: * ; prelude ! { } # [decl_ solana_monster_system_decls_bin_graphql_build_order_main] fn main () -> Result < () , Box < dyn std :: error :: Error > > { let query = r#"
    query SolanaRustcBuildOrder {
      crates(orderBy: CRITICALITY_DESC, filter: {critical: true}) {
        name
        version
        branch
        repo
        path
        gitHash
        criticality
        dependencies {
          name
          relationship
        }
      }
      rustcPath {
        solanaRustc
        makefilePath
      }
    }
    "# ; println ! ("=== GraphQL Query Result ===") ; println ! ("Query: {}" , query . trim ()) ; println ! ("\nExecuting against RocksDB + Git database...") ; let result = execute_query (query) ? ; display_result (result) ; Ok (()) }