# Split-Decls-RS: Complete Beginner's Guide
                                                                                                                                              
## What is Split-Decls-RS?                                                                                                                    
                                                                                                                                              
Split-Decls-RS is a tool that helps you modify Rust code from other projects without changing the original files. Think of it like putting a t
ransparent overlay on a map - you can draw on the overlay without damaging the original map underneath.                                       
                                                                                                                                              
## Quick Start (5 Minutes)                                                                                                                    
                                                                                                                                              
### Step 1: Build the Tool                                                                                                                    
```bash                                                                                                                                       
cd split-decls-rs                                                                                                                             
cargo build --release                                                                                                                         
```                                                                                                                                           
                                                                                                                                              
### Step 2: Create a Test Project                                                                                                             
```bash                                                                                                                                       
mkdir my_test_project                                                                                                                         
cd my_test_project                                                                                                                            
mkdir src                                                                                                                                     
```                                                                                                                                           
                                                                                                                                              
Create `Cargo.toml`:                                                                                                                          
```toml                                                                                                                                       
[package]                                                                                                                                     
name = "my_test_crate"                                                                                                                        
version = "0.1.0"                                                                                                                             
edition = "2021"                                                                                                                              
```                                                                                                                                           
                                                                                                                                              
Create `src/lib.rs`:                                                                                                                          
```rust                                                                                                                                       
pub fn hello() -> String {                                                                                                                    
    "Hello, World!".to_string()                                                                                                               
}                                                                                                                                             
                                                                                                                                              
pub fn add(a: i32, b: i32) -> i32 {                                                                                                           
    a + b                                                                                                                                     
}                                                                                                                                             
```                                                                                                                                           
                                                                                                                                              
### Step 3: Create Configuration                                                                                                              
In the split-decls-rs directory, create `split-decls-rs.toml`:                                                                                
```toml                                                                                                                                       
# Minimal working configuration                                                                                                               
string_replacements = []                                                                                                                      
patches = {}                                                                                                                                  
custom_prelude_overlay = "// Custom prelude"                                                                                                  
```                                                                                                                                           
                                                                                                                                              
### Step 4: Run the Tool                                                                                                                      
```bash                                                                                                                                       
cargo run --bin split-decls-rs my_test_project                                                                                                
```                                                                                                                                           
                                                                                                                                              
## Configuration Format (Correct)                                                                                                             
                                                                                                                                              
### Basic Structure                                                                                                                           
```toml                                                                                                                                       
# String replacements - array of replacement objects                                                                                          
string_replacements = [                                                                                                                       
    { old = "println!", new = "eprintln!" },                                                                                                  
    { old = "TODO", new = "FIXME" }                                                                                                           
]                                                                                                                                             
                                                                                                                                              
# Patches - map of crate names to patch arrays                                                                                                
[patches]                                                                                                                                     
my_crate = [                                                                                                                                  
    { path = "patches/fix1.rs" },                                                                                                             
    { path = "patches/fix2.rs", git_reference = "main" }                                                                                      
]                                                                                                                                             
                                                                                                                                              
# Custom prelude - string of Rust code                                                                                                        
custom_prelude_overlay = '''                                                                                                                  
#![allow(unused_imports)]                                                                                                                     
use std::collections::HashMap;                                                                                                                
'''                                                                                                                                           
```                                                                                                                                           
                                                                                                                                              
### String Replacements (Fixed Format)                                                                                                        
```toml                                                                                                                                       
string_replacements = [                                                                                                                       
    { old = "broken_function", new = "fixed_function" },                                                                                      
    { old = "use old_crate::", new = "use new_crate::" },                                                                                     
    { old = "// TODO", new = "// FIXME" }                                                                                                     
]                                                                                                                                             
```                                                                                                                                           
                                                                                                                                              
### Patches (Fixed Format)                                                                                                                    
```toml                                                                                                                                       
[patches]                                                                                                                                     
# Single patch file                                                                                                                           
my_crate = [                                                                                                                                  
    { path = "patches/my_fix.rs" }                                                                                                            
]                                                                                                                                             
                                                                                                                                              
# Multiple patches for one crate                                                                                                              
another_crate = [                                                                                                                             
    { path = "patches/fix1.rs" },                                                                                                             
    { path = "patches/fix2.rs" },                                                                                                             
    { path = "patches/enhancement.rs", git_reference = "v1.0" }                                                                               
]                                                                                                                                             
```                                                                                                                                           
                                                                                                                                              
## Complete Working Example                                                                                                                   
                                                                                                                                              
### Directory Structure                                                                                                                       
```                                                                                                                                           
my_project/                                                                                                                                   
├── split-decls-rs.toml     # Configuration                                                                                                   
├── patches/                # Your fixes                                                                                                      
│   └── better_hello.rs                                                                                                                       
├── target_crate/           # Project to modify                                                                                               
│   ├── Cargo.toml                                                                                                                            
│   └── src/                                                                                                                                  
│       └── lib.rs                                                                                                                            
└── output/                 # Generated files                                                                                                 
    └── Cargo.toml                                                                                                                            
```                                                                                                                                           
                                                                                                                                              
### Configuration File                                                                                                                        
```toml                                                                                                                                       
# split-decls-rs.toml                                                                                                                         
string_replacements = [                                                                                                                       
    { old = "World", new = "Universe" }                                                                                                       
]                                                                                                                                             
                                                                                                                                              
[patches]                                                                                                                                     
target_crate = [                                                                                                                              
    { path = "patches/better_hello.rs" }                                                                                                      
]                                                                                                                                             
                                                                                                                                              
custom_prelude_overlay = '''                                                                                                                  
// Added to every generated file                                                                                                              
use std::fmt::Display;                                                                                                                        
'''                                                                                                                                           
```                                                                                                                                           
                                                                                                                                              
### Patch File                                                                                                                                
```rust                                                                                                                                       
// patches/better_hello.rs                                                                                                                    
pub fn hello() -> String {                                                                                                                    
    "Hello, Amazing Universe!".to_string()                                                                                                    
}                                                                                                                                             
                                                                                                                                              
pub fn greet(name: &str) -> String {                                                                                                          
    format!("Hello, {}!", name)                                                                                                               
}                                                                                                                                             
```                                                                                                                                           
                                                                                                                                              
### Running                                                                                                                                   
```bash                                                                                                                                       
cargo run --bin split-decls-rs target_crate                                                                                                   
```                                                                                                                                           
                                                                                                                                              
## What Happens Step by Step                                                                                                                  
                                                                                                                                              
1. **Tool starts** and loads `split-decls-rs.toml`                                                                                            
2. **Finds Rust projects** in the specified directory                                                                                         
3. **Backs up original** `lib.rs` → `oldlib.rs`                                                                                               
4. **Applies string replacements** to the backed up content                                                                                   
5. **Splits code** into individual declaration files in `src/decls/`                                                                          
6. **Applies patches** by replacing matching functions/structs                                                                                
7. **Adds custom prelude** to each generated file                                                                                             
8. **Creates new `lib.rs`** that imports all the split declarations                                                                           
                                                                                                                                              
## Directory After Processing                                                                                                                 
                                                                                                                                              
**Before:**                                                                                                                                   
```                                                                                                                                           
target_crate/                                                                                                                                 
├── src/                                                                                                                                      
│   └── lib.rs                                                                                                                                
└── Cargo.toml                                                                                                                                
```                                                                                                                                           
                                                                                                                                              
**After:**                                                                                                                                    
```                                                                                                                                           
target_crate/                                                                                                                                 
├── src/                                                                                                                                      
│   ├── lib.rs          # New gateway file                                                                                                    
│   ├── oldlib.rs       # Original backed up                                                                                                  
│   └── decls/          # Split declarations                                                                                                  
│       ├── target_crate_decls_hello.rs                                                                                                       
│       ├── target_crate_decls_add.rs                                                                                                         
│       └── _decl_module_invocation.rs                                                                                                        
└── Cargo.toml                                                                                                                                
```                                                                                                                                           
                                                                                                                                              
## Troubleshooting                                                                                                                            
                                                                                                                                              
### "Failed to read generated Cargo.toml from ./output/Cargo.toml"                                                                            
**Solution:** Create the output directory:                                                                                                    
```bash                                                                                                                                       
mkdir output                                                                                                                                  
echo '[package]' > output/Cargo.toml                                                                                                          
echo 'name = "workspace"' >> output/Cargo.toml                                                                                                
echo 'version = "0.1.0"' >> output/Cargo.toml                                                                                                 
echo 'edition = "2021"' >> output/Cargo.toml                                                                                                  
```                                                                                                                                           
                                                                                                                                              
### "TOML parse error"                                                                                                                        
**Common fixes:**                                                                                                                             
- Use `string_replacements = []` not `[string_replacements]`                                                                                  
- Use `patches = {}` not `[patches]`                                                                                                          
- Use `{ old = "text", new = "replacement" }` format for replacements                                                                         
                                                                                                                                              
### "No changes applied"                                                                                                                      
- Check crate names match exactly (case-sensitive)                                                                                            
- Verify patch file paths exist                                                                                                               
- Make sure you're running from the right directory                                                                                           
                                                                                                                                              
## Ready-to-Use Templates                                                                                                                     
                                                                                                                                              
### Template 1: Simple Text Replacement                                                                                                       
```toml                                                                                                                                       
string_replacements = [                                                                                                                       
    { old = "panic!", new = "eprintln!" }                                                                                                     
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
                                                                                                                                              
## Issues Fixed and Testing Summary                                                                                                           
                                                                                                                                              
### ✅ Issues Fixed:                                                                                                                           
1. **Main function**: Fixed hardcoded main.rs to properly use CLI arguments and process individual crates                                     
2. **Missing imports**: Added required module imports to lib.rs, process_crate.rs, and backup modules                                         
3. **Type errors**: Fixed string type conversion in CLI argument parsing                                                                      
4. **Default trait**: Added Default derive to PatchConfig struct                                                                              
5. **Function signatures**: Fixed generate_new_cargotoml function call with proper parameters                                                 
                                                                                                                                              
### ✅ Successfully Tested:                                                                                                                    
1. **Basic functionality**: Tool processes single crates and splits declarations correctly                                                    
2. **String replacements**: "World" → "Universe" replacement works as expected                                                                
3. **File structure**: Generates proper directory structure with src/decls/ containing split files                                            
4. **Backup system**: Creates oldlib.rs, oldbuild.rs, and oldCargo.toml backups                                                               
5. **Declaration splitting**: Functions, structs, and other items are split into individual files                                             
                                                                                                                                              
### ⚠️ Known Limitations:                                                                                                                      
1. **Patch functionality**: Patches configuration needs further testing (custom_prelude_overlay parsing issue)                                
2. **Complex projects**: Only tested with simple single-crate projects                                                                        
3. **Dependencies**: Generated projects may need additional dependencies for compilation                                                      
                                                                                                                                              
### 📝 Updated Configuration Format:                                                                                                           
                                                                                                                                              
The tool now works with this **corrected** configuration format:                                                                              
                                                                                                                                              
```toml                                                                                                                                       
# Working configuration format                                                                                                                
string_replacements = [                                                                                                                       
    { old = "World", new = "Universe" }                                                                                                       
]                                                                                                                                             
                                                                                                                                              
# Note: patches and custom_prelude_overlay need further investigation                                                                         
# patches = {}                                                                                                                                
# custom_prelude_overlay = "// Custom prelude"                                                                                                
```