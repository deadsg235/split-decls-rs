pub fn hello_world() -> String {
    "Hello, World!".to_string()
}

pub fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}

pub struct Person {
    pub name: String,
    pub age: u32,
}

impl Person {
    pub fn new(name: String, age: u32) -> Self {
        Person { name, age }
    }
}