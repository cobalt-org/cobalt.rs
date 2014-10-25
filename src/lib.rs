extern crate mustache;

// reexport cobalt::cobalt::Runner as cobalt::Runner
pub use cobalt::Runner;
// without this main.rs would have to use cobalt::cobalt::Runner
// with this approach you can explicitly say which part of a module is public and which not

// modules
mod cobalt;
mod document;
