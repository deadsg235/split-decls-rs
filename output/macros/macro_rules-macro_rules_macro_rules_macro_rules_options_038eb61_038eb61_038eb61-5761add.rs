macro_rules ! tmod_enum { ($ tmod_enum_name : ident , $ prefix : expr , $ ({ $ ($ optinfo : tt) * }) ,* $ (,) *) => { tmod_enum ! { $ tmod_enum_name , $ prefix , @ parse {}
, (user_value) {}
; $ ($ ($ optinfo) *|) *}
} ; ($ tmod_enum_name : ident , $ prefix : expr , @ parse { $ ($ eout : tt) *}
, ($ user_value : ident) { $ ($ pout : tt) *}
;) => { #[allow (non_camel_case_types)] #[derive (PartialEq , Eq , PartialOrd , Ord , Debug , Copy , Clone , Encodable , Decodable)] pub enum $ tmod_enum_name { $ ($ eout) ,*}
impl $ tmod_enum_name { #[allow (unused_variables)] pub fn reparse (& self , $ user_value : & str) -> ExtendedTargetModifierInfo { #[allow (unreachable_patterns)] match self { $ ($ pout) * _ => panic ! ("unknown target modifier option: {:?}" , * self)}
} pub fn is_target_modifier (flag_name : & str) -> bool { match flag_name . replace ('-' , "_") . as_str () { $ (stringify ! ($ eout) => true ,) * _ => false ,}
}}
} ; ($ tmod_enum_name : ident , $ prefix : expr , @ parse { $ ($ eout : tt) *}
, ($ puser_value : ident) { $ ($ pout : tt) *}
; $ opt : ident , $ parse : ident , $ t : ty , [TARGET_MODIFIER] | $ ($ tail : tt) *) => { tmod_enum ! { $ tmod_enum_name , $ prefix , @ parse { $ ($ eout) * $ opt}
, ($ puser_value) { $ ($ pout) * Self ::$ opt => { let mut parsed : $ t = Default :: default () ; let val = if $ puser_value . is_empty () { None}
else { Some ($ puser_value)}
; parse ::$ parse (& mut parsed , val) ; ExtendedTargetModifierInfo { prefix : $ prefix . to_string () , name : stringify ! ($ opt) . to_string () . replace ('_' , "-") , tech_value : format ! ("{:?}" , parsed) ,}
} ,}
; $ ($ tail) *}
} ; ($ tmod_enum_name : ident , $ prefix : expr , @ parse { $ ($ eout : tt) *}
, ($ puser_value : ident) { $ ($ pout : tt) *}
; $ opt : ident , $ parse : ident , $ t : ty , [] | $ ($ tail : tt) *) => { tmod_enum ! { $ tmod_enum_name , $ prefix , @ parse { $ ($ eout) *}
, ($ puser_value) { $ ($ pout) *}
; $ ($ tail) *}
} ; }