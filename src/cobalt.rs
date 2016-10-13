use std::fs;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use walkdir::{WalkDir, DirEntry, WalkDirIterator};
use error::Result;
use config::Config;
use glob::Pattern;

fn ignore_filter(entry: &DirEntry, source: &Path, ignore: &[Pattern]) -> bool {
    if compare_paths(entry.path(), source) {
        return true;
    }
    let path = entry.path().strip_prefix(&source).unwrap_or(entry.path());
    let file_name = entry.file_name().to_str().unwrap_or("");
    if file_name.starts_with('_') || file_name.starts_with('.') {
        return false;
    }
    !ignore.iter().any(|p| p.matches_path(path))
}

fn compare_paths(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(p), Ok(p2)) => p == p2,
        _ => false,
    }
}

use processors::post::Post;
use processor::Processor;

/// The primary build function that transforms a directory into a site
pub fn build(config: &Config) -> Result<()> {
    trace!("Build configuration: {:?}", config);

    // join("") makes sure path has a trailing slash
    let source = PathBuf::from(&config.source).join("");
    let source = source.as_path();
    let dest = PathBuf::from(&config.dest).join("");
    let dest = dest.as_path();

    let template_extensions: Vec<&OsStr> = config.template_extensions
        .iter()
        .map(OsStr::new)
        .collect();

    let mut pros = vec![Post];

    for entry in WalkDir::new(&source)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok()) {
        for mut p in &mut pros {
            if p.match_dir(&entry, config) {
                try!(p.process(entry, config));
                break;
            }
        }
    }

    // copy all remaining files in the source to the destination
    if !compare_paths(source, dest) {
        info!("Copying remaining assets");
        let source_str = try!(source.to_str()
            .ok_or(format!("Cannot convert pathname {:?} to UTF-8", source)));

        let walker = WalkDir::new(&source)
            .into_iter()
            .filter_entry(|e| {
                ignore_filter(e, source, &config.ignore) &&
                !template_extensions.contains(&e.path()
                    .extension()
                    .unwrap_or(OsStr::new(""))) && !compare_paths(e.path(), dest)
            })
            .filter_map(|e| e.ok());

        for entry in walker {
            let entry_path = try!(entry.path()
                .to_str()
                .ok_or(format!("Cannot convert pathname {:?} to UTF-8", entry.path())));

            let relative = try!(entry_path.split(source_str)
                .last()
                .ok_or(format!("Empty path")));

            if try!(entry.metadata()).is_dir() {
                try!(fs::create_dir_all(&dest.join(relative)));
                debug!("Created new directory {:?}", dest.join(relative));
            } else {
                if let Some(parent) = Path::new(relative).parent() {
                    try!(fs::create_dir_all(&dest.join(parent)));
                }

                try!(fs::copy(entry.path(), &dest.join(relative))
                    .map_err(|e| format!("Could not copy {:?}: {}", entry.path(), e)));
                debug!("Copied {:?} to {:?}", entry.path(), dest.join(relative));
            }
        }
    }

    Ok(())
}
