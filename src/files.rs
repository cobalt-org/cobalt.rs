use std::path::{Path, PathBuf};

use ignore::Match;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use walkdir::{WalkDir, DirEntry, WalkDirIterator};

use error::Result;

pub struct FilesBuilder {
    root_dir: PathBuf,
    ignore: GitignoreBuilder,
}

impl FilesBuilder {
    pub fn new(root_dir: &Path) -> Result<FilesBuilder> {
        let mut ignore = GitignoreBuilder::new(root_dir);
        ignore.add_line(None, "**/.*")?;
        ignore.add_line(None, "**/.*/**")?;
        ignore.add_line(None, "**/_*")?;
        ignore.add_line(None, "**/_*/**")?;
        let builder = FilesBuilder {
            root_dir: root_dir.to_path_buf(),
            ignore: ignore,
        };

        Ok(builder)
    }

    pub fn add_ignore(&mut self, line: &str) -> Result<&mut FilesBuilder> {
        debug!("{:?}: adding '{}' ignore pattern", self.root_dir, line);
        self.ignore.add_line(None, line)?;
        Ok(self)
    }

    pub fn build(&self) -> Result<Files> {
        let files = Files::new(self.root_dir.as_path(), self.ignore.build()?);
        Ok(files)
    }
}

pub struct FilesIterator<'a> {
    inner: Box<Iterator<Item = PathBuf> + 'a>,
}

impl<'a> FilesIterator<'a> {
    fn new(files: &'a Files) -> FilesIterator<'a> {
        let walker = WalkDir::new(files.root_dir.as_path())
            .min_depth(1)
            .follow_links(false)
            .into_iter()
            .filter_entry(move |e| files.includes_entry(e))
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(move |e| {
                            e.path()
                                .strip_prefix(files.root_dir.as_path())
                                .ok()
                                .map(|p| p.to_path_buf())
                        });
        FilesIterator { inner: Box::new(walker) }
    }
}

impl<'a> Iterator for FilesIterator<'a> {
    type Item = PathBuf;

    fn next(&mut self) -> Option<PathBuf> {
        self.inner.next()
    }
}

pub struct Files {
    root_dir: PathBuf,
    ignore: Gitignore,
}

impl Files {
    fn new(root_dir: &Path, ignore: Gitignore) -> Files {
        Files {
            root_dir: root_dir.to_path_buf(),
            ignore: ignore,
        }
    }

    #[cfg(test)]
    pub fn includes_file(&self, file: &Path) -> bool {
        let is_dir = false;
        self.includes_path(file, is_dir)
    }

    #[cfg(test)]
    pub fn includes_dir(&self, dir: &Path) -> bool {
        let is_dir = true;
        self.includes_path(dir, is_dir)
    }

    pub fn files(&self) -> FilesIterator {
        FilesIterator::new(self)
    }

    fn includes_entry(&self, entry: &DirEntry) -> bool {
        self.includes_path(entry.path(), entry.file_type().is_dir())
    }

    fn includes_path(&self, path: &Path, is_dir: bool) -> bool {
        match self.ignore.matched(path, is_dir) {
            Match::None => true,
            Match::Ignore(glob) => {
                debug!("{:?}: ignored {:?}", path, glob.original());
                false
            }
            Match::Whitelist(glob) => {
                debug!("{:?}: allowed {:?}", path, glob.original());
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn files_includes_root_dir() {
        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .build()
            .unwrap();
        assert!(files.includes_dir(Path::new("/usr/cobalt/site")));
    }

    #[test]
    fn files_includes_child_dir() {
        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .build()
            .unwrap();
        assert!(files.includes_dir(Path::new("/usr/cobalt/site/child")));
    }

    #[test]
    fn files_excludes_hidden_dir() {
        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .build()
            .unwrap();
        assert!(!files.includes_dir(Path::new("/usr/cobalt/site/_child")));

        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .build()
            .unwrap();
        assert!(!files.includes_dir(Path::new("/usr/cobalt/site/child/_child")));

        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .build()
            .unwrap();
        assert!(!files.includes_dir(Path::new("/usr/cobalt/site/_child/child")));
    }

    #[test]
    fn files_excludes_dot_dir() {
        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .build()
            .unwrap();
        assert!(!files.includes_dir(Path::new("/usr/cobalt/site/.child")));

        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .build()
            .unwrap();
        assert!(!files.includes_dir(Path::new("/usr/cobalt/site/.child/child")));
    }

    #[test]
    fn files_includes_file() {
        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .build()
            .unwrap();
        assert!(files.includes_file(Path::new("/usr/cobalt/site/child.txt")));
    }

    #[test]
    fn files_includes_child_dir_file() {
        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .build()
            .unwrap();
        assert!(files.includes_file(Path::new("/usr/cobalt/site/child/child.txt")));
    }

    #[test]
    fn files_excludes_hidden_file() {
        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .build()
            .unwrap();
        assert!(!files.includes_file(Path::new("/usr/cobalt/site/_child.txt")));

        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .build()
            .unwrap();
        assert!(!files.includes_file(Path::new("/usr/cobalt/site/child/_child.txt")));
    }

    #[test]
    fn files_excludes_hidden_dir_file() {
        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .build()
            .unwrap();
        assert!(!files.includes_file(Path::new("/usr/cobalt/site/_child/child.txt")));

        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .build()
            .unwrap();
        assert!(!files.includes_file(Path::new("/usr/cobalt/site/child/_child/child.txt")));
    }

    #[test]
    fn files_excludes_dot_file() {
        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .build()
            .unwrap();
        assert!(!files.includes_file(Path::new("/usr/cobalt/site/.child.txt")));

        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .build()
            .unwrap();
        assert!(!files.includes_file(Path::new("/usr/cobalt/site/child/.child.txt")));
    }

    #[test]
    fn files_excludes_dot_dir_file() {
        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .build()
            .unwrap();
        assert!(!files.includes_file(Path::new("/usr/cobalt/site/.child/child.txt")));

        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .build()
            .unwrap();
        assert!(!files.includes_file(Path::new("/usr/cobalt/site/child/.child/child.txt")));
    }

    #[test]
    fn files_excludes_ignored_file() {
        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .add_ignore("README")
            .unwrap()
            .add_ignore("**/*.scss")
            .unwrap()
            .build()
            .unwrap();

        assert!(!files.includes_file(Path::new("/usr/cobalt/site/README")));
        assert!(!files.includes_file(Path::new("/usr/cobalt/site/child/README")));

        assert!(!files.includes_file(Path::new("/usr/cobalt/site/blog.scss")));
        assert!(!files.includes_file(Path::new("/usr/cobalt/site/child/blog.scss")));
    }

    #[test]
    fn files_includes_overriden_file() {
        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .add_ignore("!.htaccess")
            .unwrap()
            .build()
            .unwrap();

        assert!(files.includes_file(Path::new("/usr/cobalt/site/.htaccess")));
        assert!(files.includes_file(Path::new("/usr/cobalt/site/child/.htaccess")));
    }

    #[test]
    fn files_includes_overriden_dir() {
        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .add_ignore("!_posts")
            .unwrap()
            .add_ignore("!_posts/**")
            .unwrap()
            .add_ignore("_posts/**/_*")
            .unwrap()
            .add_ignore("_posts/**/_*/**")
            .unwrap()
            .build()
            .unwrap();

        assert!(files.includes_dir(Path::new("/usr/cobalt/site/_posts")));
        assert!(files.includes_dir(Path::new("/usr/cobalt/site/_posts/child")));

        // TODO These two cases should instead fail
        assert!(files.includes_dir(Path::new("/usr/cobalt/site/child/_posts")));
        assert!(files.includes_dir(Path::new("/usr/cobalt/site/child/_posts/child")));

        assert!(!files.includes_dir(Path::new("/usr/cobalt/site/_posts/child/_child")));
        assert!(!files.includes_dir(Path::new("/usr/cobalt/site/_posts/child/_child/child")));

        let files = FilesBuilder::new(Path::new("./"))
            .unwrap()
            .add_ignore("!_posts")
            .unwrap()
            .add_ignore("!_posts/**")
            .unwrap()
            .add_ignore("_posts/**/_*")
            .unwrap()
            .add_ignore("_posts/**/_*/**")
            .unwrap()
            .build()
            .unwrap();

        assert!(files.includes_dir(Path::new("./_posts")));
        assert!(files.includes_dir(Path::new("./_posts/child")));
    }


    #[test]
    fn files_includes_overriden_dir_file() {
        let files = FilesBuilder::new(Path::new("/user/cobalt/site"))
            .unwrap()
            .add_ignore("!_posts")
            .unwrap()
            .add_ignore("!_posts/**")
            .unwrap()
            .add_ignore("_posts/**/_*")
            .unwrap()
            .add_ignore("_posts/**/_*/**")
            .unwrap()
            .build()
            .unwrap();

        assert!(files.includes_file(Path::new("/usr/cobalt/site/_posts/child.txt")));
        assert!(files.includes_file(Path::new("/usr/cobalt/site/_posts/child/child.txt")));

        // TODO These two cases should instead fail
        assert!(files.includes_file(Path::new("/usr/cobalt/site/child/_posts/child.txt")));
        assert!(files.includes_file(Path::new("/usr/cobalt/site/child/_posts/child/child.txt")));

        assert!(!files.includes_file(Path::new("/usr/cobalt/site/_posts/child/_child.txt")));
        assert!(!files.includes_file(Path::new("/usr/cobalt/site/_posts/child/_child/child.txt")));
    }

    #[test]
    fn files_iter_matches_include() {
        let root_dir = Path::new("tests/fixtures/hidden_files");
        let files = FilesBuilder::new(root_dir).unwrap().build().unwrap();
        let mut actual: Vec<_> = files.files().collect();
        actual.sort();

        let expected = vec![Path::new("child/child.txt").to_path_buf(),
                            Path::new("child.txt").to_path_buf()];

        assert_eq!(expected, actual);
    }
}
