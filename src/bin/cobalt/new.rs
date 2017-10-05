use clap;
use cobalt;

use error::*;

pub fn init_command(_config: cobalt::Config, matches: &clap::ArgMatches) -> Result<()> {
    let directory = matches.value_of("DIRECTORY").unwrap();

    cobalt::create_new_project(&directory.to_string())?;
    info!("Created new project at {}", directory);

    Ok(())
}

pub fn new_command(config: cobalt::Config, matches: &clap::ArgMatches) -> Result<()> {
    let filetype = matches.value_of("FILETYPE").unwrap();
    let filename = matches.value_of("FILENAME").unwrap();

    cobalt::create_new_document(filetype, filename, &config)?;
    info!("Created new {} {}", filetype, filename);

    Ok(())
}
