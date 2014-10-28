extern crate cobalt;
extern crate getopts;

use cobalt::Runner;
use getopts::{optflag, getopts, OptGroup};
use std::os;

fn print_usage(program: &str, _opts: &[OptGroup]) {
    println!("Usage: {} [path] [options]", program);
    println!("-h --help\tUsage");
    println!("-v --version\tDisplay version");
}

fn print_version() {
    let version = vec!["0", "0", "1"];

    println!("{}", version.connect("."));
}

fn main() {
    let args: Vec<String> = os::args();
    let program = args[0].clone();

    let opts = [
        optflag("h", "help", "print this help menu"),
        optflag("v", "version", "Display version")
    ];

    let matches = match getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { fail!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(program.as_slice(), opts);
        return;
    }

    if matches.opt_present("v") {
        print_version();
        return;
    }

    let input = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(program.as_slice(), opts);
        return;
    };

    Runner::run(input.as_slice());
}
