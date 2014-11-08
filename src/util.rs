use std::io;
use std::io::{IoResult, InvalidInput, standard_error};
use std::io::fs;
use std::io::fs::PathExtensions;

pub fn copy_recursive_filter(source: &Path, dest: &Path, valid: |&Path| -> bool) -> IoResult<()> {
    if source.is_dir() {
        let contents = try!(fs::readdir(source));
        for entry in contents.iter() {
            if entry.is_dir() {
                if valid(entry) {
                    let new_dest = &dest.join(entry.path_relative_from(source).unwrap());
                    try!(fs::mkdir_recursive(new_dest, io::USER_RWX));
                    try!(copy_recursive_filter(entry, new_dest, |p| valid(p)));
                }
            } else {
                if valid(entry) {
                    try!(fs::copy(entry, &dest.join(entry.path_relative_from(source).unwrap())));
                }
            }
        }
        Ok(())
    } else {
        Err(standard_error(InvalidInput))
    }
}
