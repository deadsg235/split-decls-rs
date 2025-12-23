# split-decls-rs Beginner's Guide

## What is split-decls-rs?

split-decls-rs is a Rust tool that transforms Rust crates by "splitting" their declarations into individual files. This creates a modular structure that enables precise patching and overlay systems similar to Nix flakes.

## Quick Start

### 1. Installation
```bash
git clone https://github.com/deadsg235/split-decls-rs.git
cd split-decls-rs
cargo build --release
```

### 2. Basic Commands
```bash
# Show all available commands
cargo run --bin split-decls-rs -- --help

# Process crates in current directory (safe dry-run mode)
cargo run --bin split-decls-rs -- ecosystem-scan --recursive --dry-run --verbose .

# Bootstrap the tool on itself
cargo run --bin split-decls-rs -- bootstrap
```

### 3. Available Commands
- **`ecosystem-scan`**: Scan and process Rust crates in a directory
- **`wrapped-workspace`**: Generate wrapped workspace structures  
- **`execute-goal-workflow`**: Execute complex processing workflows
- **`bootstrap`**: Self-process the split-decls-rs project

## How It Works

### Input: Original Crate Structure
```
my-crate/
├── Cargo.toml
├── src/
│   └── lib.rs          # Contains all declarations
└── build.rs (optional)
```

### Output: Split Declaration Structure
```
output2/my-crate/
├── Cargo.toml          # Generated with proper dependencies
├── src/
│   ├── lib.rs          # New minimal lib.rs with re-exports
│   ├── oldlib.rs       # Backup of original lib.rs
│   └── decls/          # Split declarations directory
│       ├── _decl_module_invocation.rs
│       ├── my_crate_decls_function_name.rs
│       ├── my_crate_decls_struct_name.rs
│       └── ...
├── build.rs            # Generated build script
└── oldbuild.rs         # Backup of original build.rs
```

## Configuration

Create a `split-decls-rs.toml` file to customize the transformation:

```toml
# String replacements applied before AST parsing
string_replacements = [
    { old = "World", new = "Universe" },
    { old = "println!", new = "eprintln!" }
]

# Patch files to apply to specific crates
[patches]
my_crate = [
    { path = "fixes/safe_function.rs" }
]

# Custom prelude injected into all generated files
custom_prelude_overlay = '''
// Debug macros for all files
macro_rules! debug_print {
    ($msg:expr) => { println!("[DEBUG] {}", $msg); };
}
'''
```

## Command Examples

### ecosystem-scan Command
```bash
# Scan current directory recursively in dry-run mode
cargo run --bin split-decls-rs -- ecosystem-scan --recursive --dry-run --verbose .

# Process a specific crate directory
cargo run --bin split-decls-rs -- ecosystem-scan ./my-crate

# Scan entire workspace recursively (live mode)
cargo run --bin split-decls-rs -- ecosystem-scan --recursive ./workspace-root
```

### wrapped-workspace Command
```bash
# Generate wrapped workspace in default location
cargo run --bin split-decls-rs -- wrapped-workspace

# Generate in custom output directory
cargo run --bin split-decls-rs -- wrapped-workspace --output-dir ./my-output

# Dry-run to see what would be generated
cargo run --bin split-decls-rs -- wrapped-workspace --dry-run --verbose
```

### bootstrap Command
```bash
# Self-process the split-decls-rs project
cargo run --bin split-decls-rs -- bootstrap
```

## Understanding Generated Files

### New `src/lib.rs`
```rust
// Re-exports the split declarations
pub mod decls;
pub use decls::*;
```

### Example Split Declaration
`src/decls/my_crate_decls_hello_world.rs`:
```rust
// Common use statements
use std::*;

// Prelude macro placeholder
prelude!{}

// Declaration attribute
#[decl_my_crate]
pub fn hello_world() {
    println!("Hello, Universe!"); // "World" replaced by "Universe"
}
```

### Generated `build.rs`
Handles the transformation process during compilation, including:
- Reading and parsing the original `oldlib.rs`
- Applying string replacements and patches
- Splitting declarations into individual files
- Generating module invocations

## Configuration Templates

### Template 1: Simple String Replacement
```toml
string_replacements = [
    { old = "World", new = "Universe" },
    { old = "TODO", new = "FIXME" }
]

patches = {}
custom_prelude_overlay = ""
```

### Template 2: Function Replacement
```toml
string_replacements = []

[patches]
my_crate = [
    { path = "fixes/safe_function.rs" }
]

custom_prelude_overlay = ""
```

### Template 3: Add Debugging
```toml
string_replacements = []
patches = {}

custom_prelude_overlay = '''
// Debug macros for all files
macro_rules! debug_print {
    ($msg:expr) => { println!("[DEBUG] {}", $msg); };
}
'''
```

## Current Status & Features

### ✅ Working Features:
1. **Modern CLI Interface**: Full clap-based CLI with subcommands and options
2. **Multi-threaded Processing**: Parallel crate processing with rayon
3. **Dry-run Mode**: Safe preview of changes without modifying files
4. **Recursive Scanning**: Process entire directory trees of crates
5. **String Replacements**: Text-based transformations work correctly
6. **Declaration Splitting**: Functions, structs, and other items split into individual files
7. **Backup System**: Creates oldlib.rs, oldbuild.rs, and oldCargo.toml backups
8. **Workspace Generation**: Proper Cargo.toml generation with dependencies

### ⚠️ Known Limitations:
1. **Patch Functionality**: Advanced patch system needs further testing
2. **Complex Projects**: Primarily tested with simple single-crate projects
3. **Dependencies**: Generated projects may need additional dependencies for compilation

### 📝 Configuration Format:
The tool works with this **validated** configuration format:

```toml
# Working configuration format
string_replacements = [
    { old = "World", new = "Universe" }
]

# Note: patches and custom_prelude_overlay are implemented but need more testing
patches = {}
custom_prelude_overlay = "// Custom prelude"
```

## Troubleshooting

### Common Issues
1. **Permission Errors**: Ensure you have write permissions to the output directory
2. **Missing Dependencies**: The generated code may require additional dependencies
3. **Build Failures**: Check that the original crate compiles before processing

### Getting Help
- Use `--help` with any command for detailed usage information
- Use `--verbose` flag for detailed logging
- Use `--dry-run` to preview changes safely
- Check the [COMMANDS_REFERENCE.md](COMMANDS_REFERENCE.md) for complete documentation

## Next Steps

1. **Start with dry-run**: Always use `--dry-run` first to preview changes
2. **Test on simple crates**: Begin with small, single-file crates
3. **Experiment with configuration**: Try different string replacements
4. **Explore the output**: Examine the generated `output2/` directory structure
5. **Read the full documentation**: See [README.md](README.md) and [COMMANDS_REFERENCE.md](COMMANDS_REFERENCE.md)