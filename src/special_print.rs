use std::any::Any;
pub fn specialprint(macro_name: &str, local_score: f64) {
    print!(", {}: {:.4}", macro_name, local_score);
}
