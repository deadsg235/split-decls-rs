pub use decls :: * ; # [cfg (feature = "introspector_decl2_macros")] pub use introspector_decl2_macros :: * ; prelude ! { } # [decl_ solana_monster_system_decls_minizinc_data_structures_impl_for_MonsterGroupParameters] impl MonsterGroupParameters { pub fn new () -> Self { Self { monster_order : MONSTER_GROUP_REPRESENTATION_DIMENSION as i64 , hecke_eigenvalues : HECKE_EIGENVALUES . to_vec () , ramanujan_coefficients : RAMANUJAN_TAU_COEFFICIENTS . to_vec () , elliptic_fibers : Vec :: new () , torus_points : Vec :: new () , monster_stabilizers : Vec :: new () , } } pub fn to_dzn (& self) -> String { format ! ("monster_order = {};\n\
            hecke_eigenvalues = {:?};\n\
            ramanujan_coefficients = {:?};\n\
            n_fibers = {};\n\
            n_torus_points = {};\n\
            n_stabilizers = {};\n\
            \n\
            % Elliptic fiber data\n\
            fiber_j_invariants = {:?};\n\
            fiber_monster_elements = {:?};\n\
            fiber_dimensions = {:?};\n\
            \n\
            % Torus point data\n\
            torus_x = {:?};\n\
            torus_y = {:?};\n\
            torus_monster_coords = {:?};\n\
            torus_modular_weights = {:?};\n\
            \n\
            % Monster stabilizer data\n\
            stabilizer_elements = {:?};\n\
            stabilizer_orbit_sizes = {:?};" , self . monster_order , self . hecke_eigenvalues , self . ramanujan_coefficients , self . elliptic_fibers . len () , self . torus_points . len () , self . monster_stabilizers . len () , self . elliptic_fibers . iter () . map (| f | f . j_invariant) . collect ::< Vec < _ >> () , self . elliptic_fibers . iter () . map (| f | f . monster_element as i64) . collect ::< Vec < _ >> () , self . elliptic_fibers . iter () . map (| f | f . fiber_dimension as i32) . collect ::< Vec < _ >> () , self . torus_points . iter () . map (| p | p . x) . collect ::< Vec < _ >> () , self . torus_points . iter () . map (| p | p . y) . collect ::< Vec < _ >> () , self . torus_points . iter () . map (| p | p . monster_coordinate as i64) . collect ::< Vec < _ >> () , self . torus_points . iter () . map (| p | p . modular_weight) . collect ::< Vec < _ >> () , self . monster_stabilizers . iter () . map (| s | s . element as i64) . collect ::< Vec < _ >> () , self . monster_stabilizers . iter () . map (| s | s . orbit_size as i32) . collect ::< Vec < _ >> ()) } }