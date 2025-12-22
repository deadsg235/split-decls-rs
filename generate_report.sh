#!/bin/bash

echo "=== Split-Decls-RS Processing Report ==="
echo

# Count workspace members from the source
echo "📊 WORKSPACE ANALYSIS:"
if [ -f "../../Cargo.toml" ]; then
    members_count=$(grep -A 100 "members = \[" ../../Cargo.toml | grep -c '\".*\"' || echo "0")
    echo "  • Total workspace members: $members_count"
fi

# Count crates in the crates directory
crates_count=$(find ../../crates -maxdepth 1 -type d | wc -l)
crates_count=$((crates_count - 1)) # subtract the crates directory itself
echo "  • Crates in /crates: $crates_count"

# Count submodules
submodules_count=$(find ../../submodules -maxdepth 1 -type d | wc -l)
submodules_count=$((submodules_count - 1))
echo "  • Submodules: $submodules_count"

echo
echo "📁 OUTPUT ANALYSIS:"
echo "  • Output directory: ./output"
echo "  • Generated files:"
find ./output -type f -name "*.toml" -o -name "*.rs" | while read file; do
    echo "    - $(basename "$file")"
done

echo
echo "🔍 DECLARATION ANALYSIS:"
# Count potential declarations in a sample crate
sample_crate="../../crates/monster_traits"
if [ -f "$sample_crate/src/lib.rs" ]; then
    echo "  • Sample crate: monster_traits"
    structs=$(grep -c "^pub struct\|^struct" "$sample_crate/src/lib.rs" 2>/dev/null || echo "0")
    enums=$(grep -c "^pub enum\|^enum" "$sample_crate/src/lib.rs" 2>/dev/null || echo "0")
    functions=$(grep -c "^pub fn\|^fn" "$sample_crate/src/lib.rs" 2>/dev/null || echo "0")
    traits=$(grep -c "^pub trait\|^trait" "$sample_crate/src/lib.rs" 2>/dev/null || echo "0")
    
    echo "    - Structs: $structs"
    echo "    - Enums: $enums" 
    echo "    - Functions: $functions"
    echo "    - Traits: $traits"
    echo "    - Total declarations: $((structs + enums + functions + traits))"
fi

echo
echo "⚠️  NOTE: Individual crate processing not yet completed."
echo "   The tool successfully processed the workspace structure but"
echo "   individual declaration splitting requires per-crate processing."
