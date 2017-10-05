use std::io;
use std::sync;

use clap;
use cobalt;
use ghp;
use hyper;
use notify;

error_chain! {

    links {
    }

    foreign_links {
        Cobalt(cobalt::Error);
        Notify(notify::Error);
        Clap(clap::Error);
        Ghp(ghp::Error);
        Io(io::Error);
        Hyper(hyper::Error);
        Recv(sync::mpsc::RecvError);
    }

    errors {
    }
}
