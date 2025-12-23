# split-decls-rs Commands Reference

## Overview
The `split-decls-rs` tool provides a modern CLI interface with multiple commands for processing Rust codebases and creating overlay systems.

## Installation & Build
```bash
# Clone and build
git clone https://github.com/deadsg235/split-decls-rs.git
cd split-decls-rs
cargo build --release

# Or run directly
cargo run --bin split-decls-rs -- --help
```

## Global Options
- `-v, --verbose`: Enable verbose output for detailed logging
- `-h, --help`: Show help information
- `-V, --version`: Show version information

## Commands

### 1. `ecosystem-scan` - Scan and Process Crates
Scans a directory tree for Rust crates and processes them with the split-decls transformation.

```bash
# Basic usage
cargo run --bin split-decls-rs -- ecosystem-scan <PATH>

# With options
cargo run --bin split-decls-rs -- ecosystem-scan --recursive --dry-run --verbose ./my-project
```

**Arguments:**
- `<PATH>`: Base path to scan for crates

**Options:**
- `-r, --recursive`: Recursively scan subdirectories for Cargo.toml files
- `-d, --dry-run`: Run in dry-run mode, no files will be modified
- `-h, --help`: Show help for this command

**Examples:**
```bash
# Scan current directory recursively in dry-run mode
cargo run --bin split-decls-rs -- ecosystem-scan -r -d -v .

# Process a specific crate directory
cargo run --bin split-decls-rs -- ecosystem-scan ./my-crate

# Scan entire workspace recursively
cargo run --bin split-decls-rs -- ecosystem-scan -r ./workspace-root
```

### 2. `wrapped-workspace` - Generate Wrapped Workspace
Creates a wrapped workspace structure for managing multiple crates with overlay transformations.

```bash
# Basic usage
cargo run --bin split-decls-rs -- wrapped-workspace

# With options
cargo run --bin split-decls-rs -- wrapped-workspace --output-dir ./output --dry-run
```

**Options:**
- `-o, --output-dir <DIR>`: Directory to output the wrapped workspace (optional)
- `-d, --dry-run`: Run in dry-run mode, no files will be modified
- `-h, --help`: Show help for this command

**Examples:**
```bash
# Generate wrapped workspace in default location
cargo run --bin split-decls-rs -- wrapped-workspace

# Generate in custom output directory
cargo run --bin split-decls-rs -- wrapped-workspace -o ./my-output

# Dry-run to see what would be generated
cargo run --bin split-decls-rs -- wrapped-workspace -d -v
```

### 3. `execute-goal-workflow` - Execute Workflow
Executes a workflow defined in a goal.toml file for complex processing pipelines.

```bash
cargo run --bin split-decls-rs -- execute-goal-workflow
```

**Options:**
- `-h, --help`: Show help for this command

### 4. `bootstrap` - Self-Bootstrap
Bootstrap command that scans the project itself into output2, builds the generated code, and reports errors.

```bash
cargo run --bin split-decls-rs -- bootstrap
```

**Options:**
- `-h, --help`: Show help for this command

**What it does:**
- Processes the split-decls-rs project itself
- Generates output in `./output2/` directory
- Builds the generated code
- Reports any compilation errors or issues

## Configuration File Format

The tool uses `split-decls-rs.toml` configuration files:

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

## Build Features

### Core Functionality
- **AST-based Processing**: Uses `syn` crate for robust Rust code parsing
- **Declaration Splitting**: Splits `lib.rs` into individual declaration files
- **Backup System**: Creates `oldlib.rs`, `oldbuild.rs`, `oldCargo.toml` backups
- **Workspace Management**: Handles complex workspace configurations
- **Parallel Processing**: Uses `rayon` for multi-threaded crate processing

### Output Structure
```
output2/
├── Cargo.toml              # Generated workspace manifest
├── my-crate/
│   ├── Cargo.toml          # Generated crate manifest
│   ├── src/
│   │   ├── lib.rs          # New minimal lib.rs with re-exports
│   │   ├── oldlib.rs       # Backup of original lib.rs
│   │   └── decls/          # Split declarations directory
│   │       ├── _decl_module_invocation.rs
│   │       ├── my_crate_decls_function_name.rs
│   │       ├── my_crate_decls_struct_name.rs
│   │       └── ...
│   ├── build.rs            # Generated build script
│   └── oldbuild.rs         # Backup of original build.rs
```

### Supported Declaration Types
- Functions (`fn`)
- Structs (`struct`)
- Enums (`enum`)
- Constants (`const`)
- Static variables (`static`)
- Traits (`trait`)
- Implementations (`impl`)
- Type aliases (`type`)
- Unions (`union`)

### Advanced Features
- **Git Integration**: Can work with git repositories and submodules
- **Dependency Management**: Automatically configures workspace dependencies
- **Build Script Generation**: Creates custom build.rs for each processed crate
- **Patch Application**: Applies code patches during build process
- **Dry-Run Mode**: Preview changes without modifying files
- **Verbose Logging**: Detailed output for debugging and monitoring

## Binary Tools

The project includes several additional binary tools:

- `rustmacrodoc`: Rust macro documentation generator
- `testbuild`: Build testing utility
- `macro_analyzer`: Macro analysis tool
- `test_splitter`: Declaration splitting tester
- `submodule_decl_report`: Submodule declaration reporter
- `eigenmatrix_analyzer`: Code analysis tool

## Error Handling

The tool provides comprehensive error handling:
- Compilation errors are reported with context
- File system operations are validated
- AST parsing errors include helpful messages
- Dry-run mode prevents accidental modifications

## Performance

- **Multi-threaded**: Uses `rayon` for parallel crate processing
- **Incremental**: Only processes changed files when possible
- **Memory Efficient**: Streams large codebases without loading everything into memory
- **Fast AST Processing**: Optimized `syn` usage for quick parsing

## Integration

The tool is designed to integrate with:
- **Nix**: Can be used in Nix flake overlays
- **Cargo**: Works with standard Cargo workspaces
- **CI/CD**: Suitable for automated build pipelines
- **IDEs**: Generated code works with Rust language servers