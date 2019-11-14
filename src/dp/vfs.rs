use std::collections::HashSet;
use std::path::Path;

/// Abstraction over file system providing following operations
pub trait Vfs {
    /// Checks if given file exists on the underlying file system
    fn exists(&self, path: &str) -> bool;

    /// Copies a file to a new path. Returns true if operations is successful
    /// and false if it fails.
    fn copy(&mut self, path: &str, new_path: &str) -> bool;

    /// File name portion of a given path
    fn filename(&self, path: &str) -> String;

    /// Parent portion of a given path
    fn parent(&self, path: &str) -> String;
}

/// Local file system Vfs
pub struct LocalFileSystem {}

impl LocalFileSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl Vfs for LocalFileSystem {
    fn exists(&self, path: &str) -> bool {
        let path = Path::new(path);
        path.exists()
    }

    fn copy(&mut self, path: &str, new_path: &str) -> bool {
        match std::fs::copy(path, new_path) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn filename(&self, path: &str) -> String {
        let path = Path::new(path);
        path.file_name().unwrap().to_str().unwrap().to_string()
    }

    fn parent(&self, path: &str) -> String {
        let path = Path::new(path);
        path.parent().unwrap().to_str().unwrap().to_string()
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    pub struct TestFileSystem {
        files: HashSet<String>,
    }

    impl TestFileSystem {
        pub fn new(files: Vec<String>) -> Self {
            Self {
                files: files.into_iter().collect(),
            }
        }

        pub fn empty() -> Self {
            Self::new(vec![])
        }

        fn get_files(&self) -> Vec<String> {
            self.files.iter().map(|s| s.clone()).collect()
        }
    }

    impl Vfs for TestFileSystem {
        fn exists(&self, path: &str) -> bool {
            self.files.contains(path)
        }

        fn copy(&mut self, _path: &str, new_path: &str) -> bool {
            self.files.insert(new_path.to_string());
            true
        }

        fn filename(&self, path: &str) -> String {
            if let Some(pos) = path.rfind('/') {
                let name = &path[pos + 1..];
                name.to_string()
            } else {
                path.to_string()
            }
        }

        fn parent(&self, path: &str) -> String {
            if let Some(pos) = path.rfind('/') {
                let name = &path[..pos];
                name.to_string()
            } else {
                format!("")
            }
        }
    }

    #[test]
    fn test_filename() {
        let vfs = TestFileSystem::empty();
        assert_eq!(vfs.filename("/foo/bar/hello.txt"), "hello.txt");
        assert_eq!(vfs.filename("hello.txt"), "hello.txt");
    }

    #[test]
    fn test_parent() {
        let vfs = TestFileSystem::empty();
        assert_eq!(vfs.parent("/foo/bar/hello.txt"), "/foo/bar");
        assert_eq!(vfs.parent("hello.txt"), "");
    }

    #[test]
    fn test_local_file_system() {
        let lfs = LocalFileSystem::new();
        assert_eq!(lfs.parent("/hello/there.txt"), "/hello");
        assert_eq!(lfs.filename("/hello/there.txt"), "there.txt");
    }
}
