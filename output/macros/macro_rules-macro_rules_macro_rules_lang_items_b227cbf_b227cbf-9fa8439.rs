macro_rules ! language_item_table { ($ ($ (#[$ attr : meta]) * $ variant : ident , $ module : ident :: $ name : ident , $ method : ident , $ target : expr , $ generics : expr ;) *) => { #[doc = " A representation of all the valid lang items in Rust."] #[derive (Debug , Copy , Clone , PartialEq , Eq , Hash , Encodable , Decodable)] pub enum LangItem { $ (#[doc = concat ! ("The `" , stringify ! ($ name) , "` lang item.")] $ (#[$ attr]) * $ variant ,) *}
impl LangItem { fn from_u32 (u : u32) -> Option < LangItem > { $ (if u == LangItem ::$ variant as u32 { return Some (LangItem ::$ variant) }) * None}
#[doc = " Returns the `name` symbol in `#[lang = \"$name\"]`."] #[doc = " For example, [`LangItem::PartialEq`]`.name()`"] #[doc = " would result in [`sym::eq`] since it is `#[lang = \"eq\"]`."] pub fn name (self) -> Symbol { match self { $ (LangItem ::$ variant => $ module ::$ name ,) *}
} #[doc = " Opposite of [`LangItem::name`]"] pub fn from_name (name : Symbol) -> Option < Self > { match name { $ ($ module ::$ name => Some (LangItem ::$ variant) ,) * _ => None ,}
} #[doc = " Returns the name of the `LangItem` enum variant."] pub fn variant_name (self) -> &'static str { match self { $ (LangItem ::$ variant => stringify ! ($ variant) ,) *}
} pub fn target (self) -> Target { match self { $ (LangItem ::$ variant => $ target ,) *}
} pub fn required_generics (& self) -> GenericRequirement { match self { $ (LangItem ::$ variant => $ generics ,) *}
}}
impl LanguageItems { $ (#[doc = concat ! ("Returns the [`DefId`] of the `" , stringify ! ($ name) , "` lang item if it is defined.")] pub fn $ method (& self) -> Option < DefId > { self . items [LangItem ::$ variant as usize] }) *}
} }