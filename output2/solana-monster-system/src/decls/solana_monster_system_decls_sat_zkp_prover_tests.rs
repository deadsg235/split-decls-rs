pub use decls :: * ; # [cfg (feature = "introspector_decl2_macros")] pub use introspector_decl2_macros :: * ; prelude ! { } # [decl_ solana_monster_system_decls_sat_zkp_prover_tests] # [cfg (test)] mod tests { use super :: * ; # [test] fn test_sat_zkp_prover () { let mut prover = SATZKProver :: new () ; let rust_code = r#"
            const MONSTER_ORDER: i64 = 196883;
            const TAU_COEFFICIENTS: [i64; 5] = [1, -24, 252, 4830, 534612];
            const HECKE_EIGENVALUES: [i64; 2] = [196883, -5472];
            
            struct BottPeriodicity {
                period: usize,
            }
            
            struct ZKProof {
                valid: bool,
            }
        "# ; let result = prover . prove_mathematical_properties (rust_code) ; assert ! (result . is_ok ()) ; if let Ok (proof_result) = result { assert ! (proof_result . proof_valid) ; assert ! (proof_result . mathematical_properties . monster_group_order . is_some ()) ; assert_eq ! (proof_result . mathematical_properties . monster_group_order . unwrap () , MONSTER_GROUP_REPRESENTATION_DIMENSION as i64) ; } } # [test] fn test_fixed_point_convergence () { let mut prover = SATZKProver :: new () ; let initial_code = & format ! ("const MONSTER_ORDER: i64 = {};" , MONSTER_GROUP_REPRESENTATION_DIMENSION) ; let result = prover . test_fixed_point_convergence (initial_code) ; assert ! (result . is_ok ()) ; if let Ok (fixed_point_result) = result { assert ! (fixed_point_result . converged || fixed_point_result . iterations > 0) ; } } # [test] fn test_mathematical_structure_extraction () { let prover = SATZKProver :: new () ; let code_with_monster = & format ! ("const ORDER: i64 = {};" , MONSTER_GROUP_REPRESENTATION_DIMENSION) ; let structures = prover . extract_mathematical_structures (code_with_monster) . unwrap () ; assert_eq ! (structures . monster_group_order , Some (MONSTER_GROUP_REPRESENTATION_DIMENSION as i64)) ; let code_with_tau = "let tau = [-24, 252];" ; let structures = prover . extract_mathematical_structures (code_with_tau) . unwrap () ; assert ! (! structures . tau_coefficients . is_empty ()) ; } }