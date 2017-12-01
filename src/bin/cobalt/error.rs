use std::io;
use std::sync;

use clap;
use cobalt;
use ghp;
use hyper;
use notify;
use serde_yaml;

error_chain! {

    links {
    }

    foreign_links {
        Clap(clap::Error);
        Cobalt(cobalt::Error);
        Ghp(ghp::Error);
        Hyper(hyper::Error);
        Io(io::Error);
        Notify(notify::Error);
        Recv(sync::mpsc::RecvError);
        SerdeYaml(serde_yaml::Error);
    }

    errors {
    }
}
