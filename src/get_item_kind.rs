use syn::Item;

pub fn get_item_kind(item: &Item) -> Option<&'static str> {
    match item {
        Item::Fn(_) => Some("fn"),
        Item::Struct(_) => Some("struct"),
        Item::Enum(_) => Some("enum"),
        Item::Const(_) => Some("const"),
        Item::Static(_) => Some("static"),
        Item::Trait(_) => Some("trait"),
        Item::Impl(_) => Some("impl"),
        Item::Type(_) => Some("type"),
        Item::Union(_) => Some("union"),
        _ => None,
    }
}
