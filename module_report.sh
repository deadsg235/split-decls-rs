#!/bin/bash

echo "📊 DETAILED MODULE-BY-MODULE DECLARATION COUNT REPORT"
echo "======================================================"
echo

total_decls=0
total_crates=0

for crate_dir in output2/*/; do
    if [ -d "$crate_dir" ]; then
        crate_name=$(basename "$crate_dir")
        decl_count=$(find "$crate_dir" -name "*_decls_*.rs" 2>/dev/null | wc -l)
        
        if [ "$decl_count" -gt 0 ]; then
            echo "📦 $crate_name: $decl_count declarations"
            total_decls=$((total_decls + decl_count))
            total_crates=$((total_crates + 1))
        fi
    fi
done

echo
echo "📈 SUMMARY:"
echo "• Total crates processed: $total_crates"
echo "• Total declarations generated: $total_decls"
echo "• Average declarations per crate: $((total_decls / total_crates))"

echo
echo "🏆 TOP 10 CRATES BY DECLARATION COUNT:"
for crate_dir in output2/*/; do
    if [ -d "$crate_dir" ]; then
        crate_name=$(basename "$crate_dir")
        decl_count=$(find "$crate_dir" -name "*_decls_*.rs" 2>/dev/null | wc -l)
        if [ "$decl_count" -gt 0 ]; then
            echo "$decl_count $crate_name"
        fi
    fi
done | sort -nr | head -10 | while read count name; do
    echo "  $name: $count declarations"
done
