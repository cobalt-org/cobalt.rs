extern crate cobalt;

use cobalt::Runner;
use std::os;

fn main() {
    let args        = os::args();
    let use_default = args.len() == 1u;

    if use_default {
        Runner::run(".");
    } else {
        Runner::run(args[1].as_slice());
    }
}
