use std::{io, fs};
use std::path::Path;
use std::fs::metadata;

pub fn copy_recursive_filter<F>(source: &Path, dest: &Path, valid: &F) -> io::Result<()> where F : Fn(&Path) -> bool{
    if metadata(&source).unwrap().is_dir() {
        for entry in try!(fs::read_dir(source)){
            let entry = try!(entry).path();
            if metadata(&entry).unwrap().is_dir() {
                if valid(entry.as_path()) {
                    let real_entry = entry.to_str().unwrap().split("/").last().unwrap();
                    let new_dest = &dest.join(real_entry);
                    try!(fs::create_dir_all(new_dest));
                    try!(copy_recursive_filter(entry.as_path(), new_dest, valid));
                }
            } else {
                if valid(entry.as_path()) {
                    let real_entry = entry.to_str().unwrap().split("/").last().unwrap();
                    try!(fs::copy(entry.as_path(), &dest.join(real_entry)));
                }
            }
        }
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "source parameter needs to be a directory"))
    }
}
