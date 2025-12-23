pub fn hello() -> String {
    "Hello, Amazing Universe!".to_string()
}

pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}