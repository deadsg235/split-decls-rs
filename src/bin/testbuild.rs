use cargo_toml_generator_macros::define_root_cargo_toml;
use cargo_toml_generator_types::CargoToml;
use toml;

fn main() -> anyhow::Result<()> { // Change return type to Result
    let generated_cargo_toml: CargoToml = define_root_cargo_toml! {
        [package] {
            mkbuildrs!()
        }
    };
    
    let toml_string = toml::to_string_pretty(&generated_cargo_toml)
        .expect("Failed to serialize CargoToml to TOML string");
    
    println!("{}", toml_string);
    
    Ok(()) // Return Ok
}
