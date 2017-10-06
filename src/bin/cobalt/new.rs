use clap;
use cobalt;

use error::*;

pub fn init_command(_config: cobalt::Config, matches: &clap::ArgMatches) -> Result<()> {
    let directory = matches.value_of("DIRECTORY").unwrap();

    cobalt::create_new_project(&directory.to_string())
        .chain_err(|| "Could not create a new cobalt project")?;
    info!("Created new project at {}", directory);

    Ok(())
}

pub fn new_command(config: cobalt::Config, matches: &clap::ArgMatches) -> Result<()> {
    let filetype = matches.value_of("FILETYPE").unwrap();
    let filename = matches.value_of("FILENAME").unwrap();

    cobalt::create_new_document(filetype, filename, &config)
        .chain_err(|| format!("Could not create {}", filetype))?;
    info!("Created new {} {}", filetype, filename);

    Ok(())
}
