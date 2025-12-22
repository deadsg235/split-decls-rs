use std::any::Any;
pub fn specialprint(macro_name: Any, local_score: Any) {
    print!(", {}: {:.4}", macro_name, local_score);
}
