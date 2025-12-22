pub use decls :: * ; # [cfg (feature = "introspector_decl2_macros")] pub use introspector_decl2_macros :: * ; prelude ! { } # [decl_ solana_monster_system_decls_architectural_journey_tests] # [cfg (test)] mod tests { use super :: * ; # [test] fn test_complete_architectural_journey () { let mut journey = ArchitecturalJourney :: new () ; let source_code = r#"
            use std::collections::HashMap;
            
            fn main() {
                let mut map = HashMap::new();
                map.insert("monster", 196883);
                map.insert("hecke", -5472);
                println!("Monster Group order: {}", map["monster"]);
            }
        "# ; let build_config = r#"
            [package]
            name = "monster-compiler"
            version = "1.0.0"
            edition = "2021"
        "# ; let result = journey . execute_complete_journey (source_code , build_config) ; assert ! (result . is_ok ()) ; if let Ok (realization) = result { assert ! (realization . journey_complete) ; assert ! (realization . system_synthesized) ; assert ! (realization . computational_system_realized) ; assert ! (matches ! (realization . realization_quality , RealizationQuality :: Complete)) ; } } # [test] fn test_foundational_axiom () { let mut axiom = FoundationalAxiom :: establish () ; let source = "fn test() { println!(\"Monster Group\"); }" ; let equivalence_verified = axiom . verify_equivalence (source) ; assert ! (equivalence_verified) ; let correspondence = axiom . establish_monster_correspondence (source) ; assert ! (correspondence) ; let consistency = axiom . validate_consistency () ; assert ! (consistency) ; } }