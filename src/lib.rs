// This library uses Clippy!
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

// Deny warnings, except in dev mode
#![deny(warnings)]
// #![deny(missing_docs)]
#![cfg_attr(feature="dev", warn(warnings))]

// Ignore clippy, except in dev mode
#![cfg_attr(feature="clippy", allow(clippy))]
#![cfg_attr(feature="dev", warn(clippy))]

// Stuff we want clippy to fail on
#![cfg_attr(feature="clippy", deny(
        clone_on_copy,
        cmp_owned,
        explicit_iter_loop,
        len_zero,
        map_clone,
        map_entry,
        match_bool,
        match_same_arms,
        new_ret_no_self,
        new_without_default,
        needless_borrow,
        needless_lifetimes,
        needless_range_loop,
        needless_return,
        no_effect,
        ok_expect,
        out_of_bounds_indexing,
        ptr_arg,
        redundant_closure,
        single_char_pattern,
        unused_collect,
        useless_vec,
        ))]

extern crate liquid;
extern crate pulldown_cmark;
extern crate walkdir;
extern crate crossbeam;
extern crate chrono;
extern crate yaml_rust;
extern crate rss;
extern crate glob;
extern crate regex;

#[macro_use]
extern crate log;

pub use cobalt::build;
pub use error::Error;
pub use config::Config;
pub use new::create_new_project;

// modules
mod cobalt;
mod config;
pub mod error;
mod document;
mod new;
