use anyhow::{Context, Result};
use proc_macro2::TokenStream;
use quote::ToTokens; // Added
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use syn::{
    visit::{self, Visit},
    visit_mut::{self, VisitMut},
    Item, ItemUse,
};

use split_decls_types::SplitDeclsConfig;
