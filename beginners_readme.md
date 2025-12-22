# Split-Decls-RS: Complete Beginner's Guide

## What is Split-Decls-RS?

Split-Decls-RS is a tool that helps you modify Rust code from other projects without changing the original files. Think of it like putting a transparent overlay on a map - you can draw on the overlay without damaging the original map underneath.

## Why Would You Use This?

Imagine you want to:
- Fix a bug in someone else's Rust library
- Add features to existing code
- Experiment with changes without breaking the original
- Keep your modifications separate and organized

Split-Decls-RS lets you do all this while keeping the original code untouched.

## How It Works (Simple Explanation)

1. **Takes apart** a Rust library file (`lib.rs`) into individual pieces
2. **Applies your changes** to specific pieces
3. **Puts everything back together** with your modifications included
4. **Keeps originals safe** - never touches the source files

## Installation & Setup

### Step 1: Build the Tool
```bash
cd split-decls-rs
cargo build --release
```

### Step 2: Create Your First Configuration
Create a file called `split-decls-rs.toml`:

```toml
# Your modification rules go here
[patches]
# Empty for now - we'll add patches later

[string_replacements]
# Simple text replacements
# "old_text" = "new_text"

# Code that gets added to every file
custom_prelude_overlay = '''
// This comment will appear in every generated file
use std::collections::HashMap;
'''
```

## Basic Usage

### Running the Tool
```bash
# Process all Rust crates in current directory
cargo run --bin split-decls-rs

# Process a specific directory
cargo run --bin split-decls-rs /path/to/rust/project
```

### What Happens When You Run It

1. **Finds Rust projects** in the specified directory
2. **Backs up original files** (renames `lib.rs` to `oldlib.rs`)
3. **Splits code** into individual declaration files
4. **Applies your patches** from the config file
5. **Creates new structure** that Rust can compile

## Configuration File Explained

### Basic Structure
```toml
# split-decls-rs.toml

# Simple text replacements (applied before parsing)
[string_replacements]
"println!" = "eprintln!"  # Change all println! to eprintln!
"TODO" = "FIXME"          # Change all TODO comments to FIXME

# Code patches (replace entire functions/structs)
[patches]
"my_crate" = "patches/my_fixes.rs"  # Apply patches/my_fixes.rs to my_crate

# Code added to every generated file
custom_prelude_overlay = '''
#![allow(unused_imports)]
use std::collections::*;
'''
```

### String Replacements
Replace any text in the code before it gets processed:

```toml
[string_replacements]
# Fix a typo in all files
"recieve" = "receive"

# Change function names
"old_function_name" = "new_function_name"

# Update imports
"use old_crate::" = "use new_crate::"
```

### Patches
Replace entire functions, structs, or other code blocks:

```toml
[patches]
"target_crate_name" = "path/to/patch/file.rs"
```

Create `path/to/patch/file.rs`:
```rust
// This will replace the original function
pub fn my_function() -> String {
    "This is my improved version!".to_string()
}

// This will replace the original struct
pub struct MyStruct {
    pub new_field: i32,
    pub improved_field: String,
}
```

## Step-by-Step Example

### Example 1: Fix a Simple Function

**Original code** (in some library's `lib.rs`):
```rust
pub fn broken_function() -> i32 {
    42 / 0  // This will panic!
}
```

**Step 1:** Create `split-decls-rs.toml`:
```toml
[patches]
"example_crate" = "fixes/safe_function.rs"

custom_prelude_overlay = ""
```

**Step 2:** Create `fixes/safe_function.rs`:
```rust
pub fn broken_function() -> i32 {
    42  // Fixed: no more division by zero
}
```

**Step 3:** Run the tool:
```bash
cargo run --bin split-decls-rs
```

**Result:** The library now uses your safe version instead of the broken one!

### Example 2: Add Debug Logging

**Goal:** Add logging to all functions without modifying original code.

**Configuration:**
```toml
[string_replacements]
"fn " = "fn "  # We'll use prelude instead

custom_prelude_overlay = '''
// Add logging to every file
use log::{info, debug, warn, error};

macro_rules! log_entry {
    ($func_name:expr) => {
        debug!("Entering function: {}", $func_name);
    };
}
'''
```

## Directory Structure After Processing

**Before:**
```
my_project/
├── src/
│   └── lib.rs          # Original code
└── Cargo.toml
```

**After:**
```
my_project/
├── src/
│   ├── lib.rs          # New gateway file
│   ├── oldlib.rs       # Original backed up
│   └── decls/          # Split declarations
│       ├── my_project_decls_function1.rs
│       ├── my_project_decls_struct1.rs
│       └── _decl_module_invocation.rs
└── Cargo.toml
```

## Common Use Cases

### 1. Bug Fixes
```toml
[patches]
"buggy_crate" = "fixes/security_patch.rs"
```

### 2. Performance Improvements
```toml
[patches]
"slow_crate" = "optimizations/faster_algorithm.rs"
```

### 3. Adding Features
```toml
[patches]
"basic_crate" = "enhancements/new_methods.rs"

custom_prelude_overlay = '''
// Add new traits to all files
use serde::{Serialize, Deserialize};
'''
```

### 4. Debugging
```toml
[string_replacements]
"return " = "{ println!(\"Returning from function\"); return "

custom_prelude_overlay = '''
// Add debug macros everywhere
macro_rules! dbg_print {
    ($msg:expr) => { println!("[DEBUG] {}", $msg); };
}
'''
```

## Troubleshooting

### "Failed to read generated Cargo.toml"
- Make sure you're running the tool in a directory with Rust projects
- Check that `split-decls-rs.toml` exists in the current directory

### "Build errors after processing"
- Check your patch files for syntax errors
- Make sure replacement strings don't break Rust syntax
- Verify that your `custom_prelude_overlay` is valid Rust code

### "No changes applied"
- Check that crate names in `[patches]` match actual crate names
- Verify patch file paths are correct
- Make sure string replacements match exactly (case-sensitive)

## Advanced Tips

### 1. Test Your Changes
Always test in a copy of the project first:
```bash
cp -r original_project test_project
cd test_project
# Run split-decls-rs here
cargo test  # Make sure everything still works
```

### 2. Incremental Changes
Start small and build up:
```toml
# First, just add some imports
custom_prelude_overlay = '''
use std::collections::HashMap;
'''

# Later, add more complex changes
[patches]
"my_crate" = "small_fix.rs"
```

### 3. Version Control
Keep your patches and config in version control:
```
my_overlay_project/
├── split-decls-rs.toml
├── patches/
│   ├── fix1.rs
│   └── enhancement1.rs
└── target_projects/
    └── (projects you're modifying)
```

## What's Next?

1. **Start simple** - try string replacements first
2. **Learn by doing** - experiment with small patches
3. **Read the main README** for advanced features
4. **Check examples** in the project repository

Remember: Split-Decls-RS is like having a magic overlay system for Rust code. You can modify anything without fear of breaking the original!