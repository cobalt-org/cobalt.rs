// Deny warnings, except in `dev` mode

// To update this list
// 1. Run `rustc -W help`
// 2. Grab all `default=warn` warnings
// 3. Paste them here, deleting `warnings`, and any with `deprecated` in the name
#![cfg_attr(not(feature="dev"), deny(const_err,
        dead_code,
        illegal_floating_point_literal_pattern,
        improper_ctypes,
        non_camel_case_types,
        non_shorthand_field_patterns,
        non_snake_case,
        non_upper_case_globals,
        no_mangle_generic_items,
        overflowing_literals,
        path_statements,
        patterns_in_fns_without_body,
        plugin_as_library,
        private_in_public,
        private_no_mangle_fns,
        private_no_mangle_statics,
        renamed_and_removed_lints,
        stable_features,
        unconditional_recursion,
        unions_with_drop_fields,
        unknown_lints,
        unreachable_code,
        unreachable_patterns,
        unused_allocation,
        unused_assignments,
        unused_attributes,
        unused_comparisons,
        unused_features,
        unused_imports,
        unused_macros,
        unused_must_use,
        unused_mut,
        unused_parens,
        unused_unsafe,
        unused_variables,
        while_true))]
// This list is select `allow` warnings
#![deny(trivial_casts,
       trivial_numeric_casts,
       unused_extern_crates,
       unused_import_braces)]
#![cfg_attr(feature="cargo-clippy", allow(
        cyclomatic_complexity,
        needless_pass_by_value))]

extern crate chrono;
extern crate ignore;
extern crate itertools;
extern crate liquid;
extern crate normalize_line_endings;
extern crate pulldown_cmark;
extern crate regex;
extern crate rss;
extern crate jsonfeed;
extern crate walkdir;
extern crate serde_yaml;
extern crate serde_json;
extern crate toml;

#[cfg(feature = "sass")]
extern crate sass_rs;

#[cfg(all(feature = "syntax-highlight"))]
extern crate syntect;

#[macro_use]
extern crate log;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde;

pub use cobalt::build;
pub use error::Error;
pub use cobalt_model::Config;
pub use cobalt_model::ConfigBuilder;

pub mod error;
pub mod cobalt_model;

mod cobalt;
mod document;

pub mod jekyll_model;
mod syntax_highlight;

pub use syntax_highlight::{list_syntax_themes, list_syntaxes};
