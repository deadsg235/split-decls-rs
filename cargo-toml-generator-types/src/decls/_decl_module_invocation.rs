// Simple module declarations instead of decl_module macro
pub mod cargo_toml_generator_types_decls_CargoToml;
pub mod cargo_toml_generator_types_decls_Package;
pub mod cargo_toml_generator_types_decls_Workspace;
pub mod cargo_toml_generator_types_decls_Dependency;
pub mod cargo_toml_generator_types_decls_impl_for_Default;
pub mod cargo_toml_generator_types_decls_DependencyTable;
pub mod cargo_toml_generator_types_decls_PatchSection;

// Re-export all items
pub use cargo_toml_generator_types_decls_CargoToml::*;
pub use cargo_toml_generator_types_decls_Package::*;
pub use cargo_toml_generator_types_decls_Workspace::*;
pub use cargo_toml_generator_types_decls_Dependency::*;
pub use cargo_toml_generator_types_decls_impl_for_Default::*;
pub use cargo_toml_generator_types_decls_DependencyTable::*;
pub use cargo_toml_generator_types_decls_PatchSection::*;