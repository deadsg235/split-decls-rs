#[doc = " Macro for generating OptionsTargetsModifiers top-level enum with impl."] #[doc = " Will generate something like:"] #[doc = " ```rust,ignore (illustrative)"] #[doc = " pub enum OptionsTargetModifiers {"] #[doc = "     CodegenOptions(CodegenOptionsTargetModifiers),"] #[doc = "     UnstableOptions(UnstableOptionsTargetModifiers),"] #[doc = " }"] #[doc = " impl OptionsTargetModifiers {"] #[doc = "     pub fn reparse(&self, user_value: &str) -> ExtendedTargetModifierInfo {"] #[doc = "         match self {"] #[doc = "             Self::CodegenOptions(v) => v.reparse(user_value),"] #[doc = "             Self::UnstableOptions(v) => v.reparse(user_value),"] #[doc = "         }"] #[doc = "     }"] #[doc = "     pub fn is_target_modifier(flag_name: &str) -> bool {"] #[doc = "         CodegenOptionsTargetModifiers::is_target_modifier(flag_name) ||"] #[doc = "         UnstableOptionsTargetModifiers::is_target_modifier(flag_name)"] #[doc = "     }"] #[doc = " }"] #[doc = " ```"] macro_rules ! top_level_tmod_enum { ($ ({ $ ($ optinfo : tt) * }) ,* $ (,) *) => { top_level_tmod_enum ! { @ parse {}
, (user_value) {}
; $ ($ ($ optinfo) *|) *}
} ; (@ parse { $ ($ variant : tt ($ substruct_enum : tt)) *}
, ($ user_value : ident) { $ ($ pout : tt) *}
;) => { #[allow (non_camel_case_types)] #[derive (PartialEq , Eq , PartialOrd , Ord , Debug , Copy , Clone , Encodable , Decodable)] pub enum OptionsTargetModifiers { $ ($ variant ($ substruct_enum)) ,*}
impl OptionsTargetModifiers { #[allow (unused_variables)] pub fn reparse (& self , $ user_value : & str) -> ExtendedTargetModifierInfo { #[allow (unreachable_patterns)] match self { $ ($ pout) * _ => panic ! ("unknown target modifier option: {:?}" , * self)}
} pub fn is_target_modifier (flag_name : & str) -> bool { $ ($ substruct_enum :: is_target_modifier (flag_name)) ||*}
}}
; (@ parse { $ ($ eout : tt) *}
, ($ puser_value : ident) { $ ($ pout : tt) *}
; [SUBSTRUCT $ substruct_enum : ident $ variant : ident] | $ ($ tail : tt) *) => { top_level_tmod_enum ! { @ parse { $ ($ eout) * $ variant ($ substruct_enum)}
, ($ puser_value) { $ ($ pout) * Self ::$ variant (v) => v . reparse ($ puser_value) ,}
; $ ($ tail) *}
} ; (@ parse { $ ($ eout : tt) *}
, ($ puser_value : ident) { $ ($ pout : tt) *}
; [$ non_substruct : ident] | $ ($ tail : tt) *) => { top_level_tmod_enum ! { @ parse { $ ($ eout) *}
, ($ puser_value) { $ ($ pout) *}
; $ ($ tail) *}
} ; }