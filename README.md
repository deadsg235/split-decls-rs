# Rust Patching & Code Reuse System: Building "Rust Overlays" with Procedural Macros

## Problem Statement

In software development, particularly within complex ecosystems or when integrating external dependencies, the need often arises to modify, extend, or "patch" existing code. Direct modification of upstream codebases can lead to maintenance headaches, difficulty in upgrading dependencies, and challenges in managing custom changes across multiple projects. The goal is to avoid manual, mass editing of large volumes of code that require custom fixes or transformations.

## The "Rust Overlay" Solution

This project proposes a novel approach to address these challenges within the Rust ecosystem: building a system analogous to Nix flake overlays, but implemented entirely in pure Rust using `build.rs` scripts, procedural macros, and `syn` reflection.

The core idea is to establish a centralized mechanism for injecting patches and transformations into a codebase without directly altering the original source files. This allows for:

*   **Centralized Management:** All custom modifications are managed from a single, controlled point within the project's build system.
*   **Decoupled Modifications:** Patches are applied dynamically during the build process, leaving the original source code pristine and simplifying dependency updates.
*   **Semantic Awareness:** Leveraging advanced code introspection and analysis to apply intelligent, structural modifications rather than brittle text-based patches.

## Core Mechanisms

### `build.rs` as the Orchestrator

The `build.rs` script plays a pivotal role. Executed prior to the main compilation, it acts as the primary orchestrator for:

1.  **Versioned Source Ingestion:** Acquiring and preparing different versions of target Rust code (e.g., `rustc` source) for analysis. This might involve locating code in the Nix store or managing local checkouts.
2.  **Code Reflection:** Initiating the process of lifting the target codebase into a high-level, computable meta-model.
3.  **Generating Metadata:** Producing necessary metadata files (e.g., semantic hashes, call graphs) that guide the patching process.

### Procedural Macros and `syn` Reflection

Procedural macros, powered by the `syn` and `quote` crates, are at the heart of the patching injection mechanism. They enable:

1.  **Deep Code Introspection:** Using `syn` to parse and analyze the Abstract Syntax Tree (AST) of the target code at compile time.
2.  **Intelligent Transformation:** Modifying the AST to inject new code, alter existing structures, or remove elements, based on the analysis performed by `build.rs`.
3.  **Seamless Integration:** The modified AST is then re-emitted as Rust code, effectively "patching" the dependency during its compilation, without any direct edits to the source files themselves.

### The "Tower of Reflection" and "Grep2Code Morphism"

To enable intelligent and robust patching, the system incorporates advanced semantic analysis capabilities:

*   **Total Reflection (Level 3):** Each version of the target codebase is processed through a "Total Reflection" mechanism (e.g., conceptually, `pure_reflect!` or `dir_reflect!`), lifting the entire codebase into a Lean4-style `Expr` meta-model. This provides a deep, computable representation of the code's structure and intent.
*   **Semantic Indexing:** The reflected code is organized within a Functional Virtual File System (VFS), generating stable identifiers (`semantic_hash.txt`) and tracking dependency relationships (`call_graph_in.json`).
*   **Semantic Diffing:** The "Grep2Code Morphism" enables structural comparison between different versions of the codebase. By converting ASTs into RDF Turtle representations, it identifies mismatches in semantic hashes and changes in the `Expr` graph, yielding a "semantic diff vector."
*   **LLM-Driven Synthesis:** High-level macros (`llm!`, `dwim!`) can then interpret this semantic diff vector to understand *why* changes occurred and even propose intelligent, context-aware patches or insights.

## Rebuilding Nix Overlays in Pure Rust

This system effectively re-imagines the concept of Nix overlays for the Rust ecosystem. Just as Nix overlays allow for declarative, reproducible modifications to package sets, this "Rust Overlay" system enables declarative, reproducible modifications to Rust codebases. This approach ensures that custom patches are applied consistently, are traceable, and can be easily updated or re-evaluated against new versions of upstream dependencies.

## Benefits

*   **Maintainability:** Easier to manage and update custom patches, as they are decoupled from the original source.
*   **Reusability:** Patches and transformations can be designed as modular macros, reusable across different projects or versions.
*   **Traceability:** Clear visibility into how and why external code is being modified.
*   **Reduced Boilerplate:** Automates complex code transformations that would otherwise require significant manual effort.

## Conceptual Usage Example

A developer might interact with this system within a `proc_macro` context as follows:

```
use patch_build_rs_macros::{nix_rust_src, pure_reflect, semantic_diff};

// 1. Ingest stable and broken versions
let v_stable = pure_reflect!(nix_rust_src!("1.82"));
let v_broken = pure_reflect!(nix_rust_src!("1.83-broken-branch"));

// 2. Calculate semantic diff vector
let diff_vector = semantic_diff!(v_stable, v_broken);

// 3. Generate a semantic change report
let report = llm! {
    prompt: "Analyze this semantic diff vector and explain why the broken branch deviates from stable intent.",
    data: diff_vector
};
```

## Current Project Functionality (`split-decls-rs`)

The `split-decls-rs` executable is a build-time code transformer designed to restructure Rust crates within a workspace. Its primary function is to "split" declarations out of a crate's main `lib.rs` file, facilitating more granular control and further processing by the "Rust Overlay" system.

For each detected crate (excluding itself and certain blacklisted directories like `submodules/rust/compiler` or `submodules/rust-analyzer`), it performs the following steps:

1.  **Backup Original Files:** Renames the existing `src/lib.rs` to `src/oldlib.rs` and `build.rs` to `oldbuild.rs`. If these files don't exist, empty placeholders are created. This preserves the original content for later use.
2.  **Generate New `src/lib.rs`:** Creates a new, minimal `src/lib.rs` for the target crate. This new `lib.rs` primarily re-exports prelude macros from `introspector_decl2_macros` and declares/exports a `decls` module (`pub mod decls; pub use decls::*;`). The actual declarations originally present in `src/oldlib.rs` are intended to be processed and moved into files within this new `decls` module by the generated `build.rs`.
3.  **Generate New `build.rs`:** Creates a custom `build.rs` for the target crate. This generated script is based on a template (`src/buildrscontent.rst`) and includes logic to:
    *   Read the content of `src/oldlib.rs`.
    *   Process these declarations (e.g., using `introspector_decl_core` and `introspector_macro_helpers`).
    *   Generate individual declaration files (`.rs` files) within the `src/decls` directory.
    *   Potentially incorporate logic from the original `oldbuild.rs`.

In essence, `split-decls-rs` prepares crates for semantic introspection and transformation by modularizing their declarations, moving them into a generated structure managed by an intermediary `build.rs` script. This sets the stage for the advanced patching and code reuse capabilities described in this README.

## Generated `build.rs` Logic

For each target crate, `split-decls-rs` generates a specialized `build.rs` script using the template found at `src/buildrscontent.rst`. This generated `build.rs` is responsible for the fine-grained parsing and splitting of declarations.

The generated `build.rs` performs the following steps:

1.  **Dependency Tracking:** Sets `cargo:rerun-if-changed` directives for `build.rs` itself, `oldlib.rs`, and `oldbuild.rs` to ensure the build script is re-executed if any of these files change.
2.  **Read and Parse `oldlib.rs`:** Reads the content of the `src/oldlib.rs` (the backed-up original `lib.rs`) and parses it into a `syn::File` (Abstract Syntax Tree).
3.  **Collect `use` Statements:** A custom `UseStatementCollector` visits the parsed AST to gather all top-level `use` statements. These are then aggregated and injected into each generated declaration file as `common_uses`.
4.  **Extract and Split Declarations:** Iterates through each top-level `syn::Item` in `oldlib.rs`.
    *   For supported declaration types (functions, structs, enums, consts, statics, traits, impls, types, unions), it extracts the `TokenStream` of the declaration.
    *   It *skips* top-level `use` statements (which are handled by the collector), `macro` invocations, and `mod` declarations.
    *   For each extracted declaration, it creates a new Rust file within the target crate's `src/decls/` directory (e.g., `src/decls/{crate_name}_decls_{decl_name}.rs`).
    *   The content of each new declaration file includes:
        *   The `common_uses` collected earlier.
        *   A `prelude!{}` macro placeholder.
        *   A `#[decl_{module_name_ident}]` attribute placeholder.
        *   The `TokenStream` of the extracted declaration itself.
5.  **Generate `decl_module!` Invocation:** After processing all declarations, it collects the module names of all generated declaration files and uses them to construct a `introspector_decl2_macros::decl_module!` invocation. This invocation is written to `src/decls/_decl_module_invocation.rs`, which is then included by the new `src/lib.rs` to re-export all processed declarations.

This process ensures that each original declaration becomes a separate, addressable unit, making it amenable to granular reflection, analysis, and targeted patching by the broader "Rust Overlay" system.

