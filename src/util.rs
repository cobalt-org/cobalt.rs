use std::{io, fs};
use std::fs::PathExt;
use std::path::Path;

pub fn copy_recursive_filter<F>(source: &Path, dest: &Path, valid: &F) -> io::Result<()> where F : Fn(&Path) -> bool{
    if source.is_dir() {
        for entry in try!(fs::read_dir(source)){
            let entry = try!(entry).path();
            if entry.is_dir() {
                if valid(entry.as_path()) {
                    let new_dest = &dest.join(entry.relative_from(source).unwrap());
                    try!(fs::create_dir_all(new_dest));
                    try!(copy_recursive_filter(entry.as_path(), new_dest, valid));
                }
            } else {
                if valid(entry.as_path()) {
                    try!(fs::copy(entry.as_path(), &dest.join(entry.relative_from(source).unwrap())));
                }
            }
        }
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "source parameter needs to be a directory"))
    }
}
