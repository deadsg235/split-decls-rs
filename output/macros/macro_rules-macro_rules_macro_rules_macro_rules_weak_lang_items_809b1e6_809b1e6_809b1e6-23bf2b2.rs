macro_rules ! weak_lang_items { ($ ($ item : ident , $ sym : ident ;) *) => { pub static WEAK_LANG_ITEMS : & [LangItem] = & [$ (LangItem ::$ item ,) *] ; impl LangItem { pub fn is_weak (self) -> bool { matches ! (self , $ (LangItem ::$ item) |*)}
pub fn link_name (self) -> Option < Symbol > { match self { $ (LangItem ::$ item => Some (sym ::$ sym) ,) * _ => None ,}
}}
} }