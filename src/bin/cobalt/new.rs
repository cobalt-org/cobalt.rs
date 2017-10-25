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
        .about("Create a new post or page")
        .args(&args::get_config_args())
        .arg(clap::Arg::with_name("FILETYPE")
                 .help("Type of file to create eg post or page")
                 .default_value("post")
                 .takes_value(true))
        .arg(clap::Arg::with_name("FILENAME")
                 .help("File to create")
                 .default_value_if("FILETYPE", Some("page"), "new_page.md")
                 .default_value("new_post.md")
                 .takes_value(true))
}

pub fn new_command(matches: &clap::ArgMatches) -> Result<()> {
    let config = args::get_config(matches)?;
    let config = config.build()?;

    let filetype = matches.value_of("FILETYPE").unwrap();
    let filename = matches.value_of("FILENAME").unwrap();

    cobalt::create_new_document(filetype, filename, &config)
        .chain_err(|| format!("Could not create {}", filetype))?;
    info!("Created new {} {}", filetype, filename);

    Ok(())
}
