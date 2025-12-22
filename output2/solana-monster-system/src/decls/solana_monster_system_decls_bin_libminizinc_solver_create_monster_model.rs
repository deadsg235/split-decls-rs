pub use decls :: * ; # [cfg (feature = "introspector_decl2_macros")] pub use introspector_decl2_macros :: * ; prelude ! { } # [decl_ solana_monster_system_decls_bin_libminizinc_solver_create_monster_model] fn create_monster_model () -> String { r#"
% Real Monster Group Constraint Model
int: MONSTER_ORDER = 196883;

% Input variables
var 0..1000000: solana_blocks;
var 0..100000: code_lines;
var 0..50000: meme_power;
var 0..500000: social_score;

% Monster variables
var 1..MONSTER_ORDER: monster_element;
var 0..30: binary_exp;
var 0..15: ternary_exp;

% Constraints
constraint binary_exp = solana_blocks div 33333;
constraint ternary_exp = meme_power div 3333;
constraint monster_element = (pow(2, binary_exp) + pow(3, ternary_exp)) mod MONSTER_ORDER;

% Bounds
constraint solana_blocks >= 100000;
constraint code_lines >= 10000;
constraint meme_power >= 5000;
constraint social_score >= 50000;

solve maximize monster_element;

output [
  "Monster element: ", show(monster_element), "\n",
  "Binary: 2^", show(binary_exp), " = ", show(pow(2, binary_exp)), "\n",
  "Ternary: 3^", show(ternary_exp), " = ", show(pow(3, ternary_exp)), "\n"
];
"# . to_string () }