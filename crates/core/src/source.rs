use crate::Result;
use crate::Status;

#[derive(Debug, Clone)]
pub struct Source {
    root: std::path::PathBuf,
    ignore: ignore::gitignore::Gitignore,
}

impl Source {
    pub fn new<'i>(
        root: &std::path::Path,
        ignores: impl IntoIterator<Item = &'i str>,
    ) -> Result<Self> {
        let mut ignore = ignore::gitignore::GitignoreBuilder::new(root);
        for line in ignores.into_iter() {
            ignore
                .add_line(None, line)
                .map_err(|e| Status::new("Invalid ignore entry").with_source(e))?;
        }
        let ignore = ignore
            .build()
            .map_err(|e| Status::new("Invalid ignore entry").with_source(e))?;

        let source = Self {
            root: root.to_owned(),
            ignore,
        };
        Ok(source)
    }

    pub fn root(&self) -> &std::path::Path {
        &self.root
    }

    pub fn includes_file(&self, file: &std::path::Path) -> bool {
        let is_dir = false;
        self.includes_path(file, is_dir)
    }

    pub fn includes_dir(&self, dir: &std::path::Path) -> bool {
        let is_dir = true;
        self.includes_path(dir, is_dir)
    }

    pub fn iter(&self) -> impl Iterator<Item = std::path::PathBuf> + '_ {
        walkdir::WalkDir::new(&self.root)
            .min_depth(1)
            .follow_links(false)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(move |e| self.includes_entry(e))
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(move |e| e.path().to_path_buf())
    }

    fn includes_path(&self, path: &std::path::Path, is_dir: bool) -> bool {
        if path == self.root {
            return true;
        }

        let parent = path.parent();
        if let Some(parent) = parent {
            if parent.starts_with(&self.root) && !self.includes_path(parent, parent.is_dir()) {
                return false;
            }
        }

        self.includes_path_leaf(path, is_dir)
    }

    fn includes_path_leaf(&self, path: &std::path::Path, is_dir: bool) -> bool {
        match self.ignore.matched(path, is_dir) {
            ignore::Match::None => true,
            ignore::Match::Ignore(glob) => {
                log::trace!("{:?}: ignored {:?}", path, glob.original());
                false
            }
            ignore::Match::Whitelist(glob) => {
                log::trace!("{:?}: allowed {:?}", path, glob.original());
                true
            }
        }
    }

    fn includes_entry(&self, entry: &walkdir::DirEntry) -> bool {
        let file = entry.path();

        // Assumption: The parent paths will have been checked before we even get to this point.
        let is_dir = entry.file_type().is_dir();
        self.includes_path_leaf(file, is_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_includes_dir {
        ($root:expr, $ignores:expr, $test:expr, $included:expr) => {
            let root = $root;
            let ignores = $ignores.clone();
            let files = Source::new(std::path::Path::new(root), ignores).unwrap();
            assert_eq!(files.includes_dir(std::path::Path::new($test)), $included);
        };
    }
    macro_rules! assert_includes_file {
        ($root:expr, $ignores:expr, $test:expr, $included:expr) => {
            let root = $root;
            let ignores = $ignores.clone();
            let files = Source::new(std::path::Path::new(root), ignores).unwrap();
            assert_eq!(files.includes_file(std::path::Path::new($test)), $included);
        };
    }

    #[test]
    fn files_includes_root_dir() {
        assert_includes_dir!("/usr/cobalt/site", &[], "/usr/cobalt/site", true);

        assert_includes_dir!("./", &[], "./", true);
    }

    #[test]
    fn files_includes_child_dir() {
        assert_includes_dir!("/usr/cobalt/site", &[], "/usr/cobalt/site/child", true);

        assert_includes_dir!("./", &[], "./child", true);
    }

    #[test]
    fn files_includes_file() {
        assert_includes_file!("/usr/cobalt/site", &[], "/usr/cobalt/site/child.txt", true);

        assert_includes_file!("./", &[], "./child.txt", true);
    }

    #[test]
    fn files_ignore_hidden() {
        assert_includes_file!(
            "/usr/cobalt/site",
            &[".*"],
            "/usr/cobalt/site/.child.txt",
            false
        );
    }

    #[test]
    fn files_not_ignored_by_parent() {
        assert_includes_file!(
            "/tmp/.foo/cobalt/site",
            &[".*"],
            "/tmp/.foo/cobalt/site/child.txt",
            true
        );
    }

    #[test]
    fn files_includes_child_dir_file() {
        assert_includes_file!(
            "/usr/cobalt/site",
            &[],
            "/usr/cobalt/site/child/child.txt",
            true
        );

        assert_includes_file!("./", &[], "./child/child.txt", true);
    }
}
