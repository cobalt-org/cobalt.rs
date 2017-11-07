use std::env;
use std::path;

use clap;
use cobalt;

use args;
use error::*;

pub fn init_command_args() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("init")
        .about("create a new cobalt project")
        .arg(clap::Arg::with_name("DIRECTORY")
                 .help("Target directory")
                 .default_value("./")
                 .index(1))
}

pub fn init_command(matches: &clap::ArgMatches) -> Result<()> {
    let directory = matches.value_of("DIRECTORY").unwrap();

    cobalt::create_new_project(&directory.to_string())
        .chain_err(|| "Could not create a new cobalt project")?;
    info!("Created new project at {}", directory);

    Ok(())
}

pub fn new_command_args() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("new")
        .about("Create a document")
        .args(&args::get_config_args())
        .arg(clap::Arg::with_name("TITLE")
                 .required(true)
                 .help("Title of the post")
                 .takes_value(true))
        .arg(clap::Arg::with_name("file")
                 .short("f")
                 .long("file")
                 .value_name("DIR_OR_FILE")
                 .help("New document's parent directory or file (default: `<CWD>/title.ext`)")
                 .takes_value(true))
}

pub fn new_command(matches: &clap::ArgMatches) -> Result<()> {
    let config = args::get_config(matches)?;
    let config = config.build()?;

    let title = matches.value_of("TITLE").unwrap();

    let mut file = env::current_dir().expect("How does this fail?");
    if let Some(rel_file) = matches.value_of("file") {
        file.push(path::Path::new(rel_file))
    }

    cobalt::create_new_document(&config, title, file)
        .chain_err(|| format!("Could not create `{}`", title))?;

    Ok(())
}

pub fn publish_command_args() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("publish")
        .about("Publish a document")
        .arg(clap::Arg::with_name("FILENAME")
                 .required(true)
                 .help("Document path to publish")
                 .takes_value(true))
}

pub fn publish_command(matches: &clap::ArgMatches) -> Result<()> {
    let file = matches
        .value_of("FILENAME")
        .expect("required parameters are present");
    let file = path::Path::new(file);

    cobalt::publish_document(file)
        .chain_err(|| format!("Could not publish `{:?}`", file))?;

    Ok(())
}
