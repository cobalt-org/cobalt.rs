pub struct Document;

impl Document {
    pub fn new(path: &Path) {
        println!("Creating Document from path {}", path.as_str().unwrap());
    }
}
