pub struct Runner;

impl Runner {
    pub fn run(path: &str) {
        println!("Generating site in {}", path);

        let documents = Runner::parse_documents(path.to_string() + "/_posts");
    }

    fn parse_documents(document_path: &str) {
        println!("{}", document_path);
    }
}
