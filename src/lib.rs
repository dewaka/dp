use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
use regex::Captures;
use regex::Regex;
use std::collections::HashSet;
use std::path::Path;

type DpDateTime = NaiveDateTime;

fn current_local_date_time() -> DpDateTime {
    Utc::now().naive_utc()
}

fn local_date(year: i32, month: u32, date: u32) -> DpDateTime {
    NaiveDateTime::new(
        NaiveDate::from_ymd(year, month, date),
        NaiveTime::from_hms_milli(0, 0, 0, 0),
    )
}

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

/// Rule for renaming a file
/// `apply` method returns an optional renamed to new name if the rule is successful
pub trait Rule {
    fn apply(&self, input: &str) -> Option<String>;
}

pub struct Duplicator<'a> {
    rules: Vec<&'a dyn Rule>,
    vfs: &'a mut dyn Vfs,
}

impl<'a> Duplicator<'a> {
    pub fn new(rules: Vec<&'a dyn Rule>, vfs: &'a mut dyn Vfs) -> Self {
        Self { rules, vfs }
    }

    pub fn duplicate(&mut self, file: &str) -> bool {
        for rule in &self.rules {
            if let Some(ref renamed) = rule.apply(file) {
                if self.vfs.exists(&renamed) {
                    continue;
                } else {
                    return self.vfs.copy(file, renamed);
                }
            }
        }

        false
    }
}

/// A rule for renaming files based on dates
pub struct DateRule {
    regex: Regex,
    date_fmt: String,
    now: DpDateTime,
}

impl DateRule {
    pub fn new(regex: Regex, date_fmt: &str, now: DpDateTime) -> Self {
        Self {
            regex,
            date_fmt: date_fmt.to_string(),
            now,
        }
    }

    pub fn compile(pattern: &str, date_fmt: &str, now: DpDateTime) -> Self {
        let regex_str = format!("(.*)({})(.*)", pattern);
        let regex: Regex = Regex::new(&regex_str).unwrap();
        Self::new(regex, date_fmt, now)
    }

    pub fn compile_now(pattern: &str, date_fmt: &str) -> Self {
        Self::compile(pattern, date_fmt, current_local_date_time())
    }
}

impl Rule for DateRule {
    fn apply(&self, input: &str) -> Option<String> {
        if self.regex.is_match(input) {
            let new_date = self.now.format(&self.date_fmt).to_string();
            let replaced = self.regex.replace_all(input, |caps: &Captures| {
                format!("{}{}{}", &caps[1], new_date, &caps[3])
            });

            Some(replaced.to_string())
        } else {
            None
        }
    }
}

/// IncrementRule for rewriting a file
/// - rename foo.txt to foo1.txt
pub struct IncrementRule {}

impl IncrementRule {
    pub fn new() -> Self {
        Self {}
    }

    fn increment(nstr: &str) -> i32 {
        let num: i32 = nstr.parse().unwrap();
        num + 1
    }
}

impl Rule for IncrementRule {
    fn apply(&self, input: &str) -> Option<String> {
        let regex = Regex::new(r"(.*)(\d+)(.*)").unwrap();

        if regex.is_match(input) {
            Some(
                regex
                    .replace_all(input, |caps: &Captures| {
                        format!("{}{}{}", &caps[1], Self::increment(&caps[2]), &caps[3])
                    })
                    .to_string(),
            )
        } else {
            None
        }
    }
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
mod test {
    use super::*;
    use chrono::Local;

    struct TestFileSystem {
        files: HashSet<String>,
    }

    impl TestFileSystem {
        fn new(files: Vec<String>) -> Self {
            Self {
                files: files.into_iter().collect(),
            }
        }

        fn empty() -> Self {
            Self::new(vec![])
        }

        fn print_files(&self) {
            println!("Files: {:?}", self.files);
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

    fn test_local_date() -> DpDateTime {
        local_date(2019, 11, 10)
    }

    fn date_rule(regex: &str, date_fmt: &str) -> DateRule {
        DateRule::compile(regex, date_fmt, test_local_date())
    }

    #[test]
    fn test_duplicator() {
        let r1: &dyn Rule = &date_rule(r"\d{2}-\d{2}", "%m-%d");
        let r2: &dyn Rule = &date_rule(r"\d{2}", "%d");

        let rules = vec![r1, r2];
        let mut vfs = TestFileSystem::new(vec![
            "hello-10-23.org".to_string(),
            "meeting-23.org".to_string(),
        ]);
        let mut duplicator = Duplicator::new(rules, &mut vfs);

        assert!(duplicator.duplicate("hello-10-23.org"));
        assert!(duplicator.duplicate("meeting-23.org"));

        // Now the same renames should fail
        assert!(vfs.exists("hello-11-10.org"));
        assert!(vfs.exists("meeting-10.org"));
    }

    #[test]
    fn test_date_rule() {
        assert_eq!(
            date_rule(r"\d{2}-\d{2}", "%m-%d").apply("hello-10-23.org"),
            Some("hello-11-10.org".to_string())
        );

        assert_eq!(
            date_rule(r"\d{2}", "%d").apply("hello-23.org"),
            Some("hello-10.org".to_string())
        );

        assert_eq!(
            date_rule(r"\d{2}-\d{2}", "%m-%d").apply("hello-there.org"),
            None
        );
        assert_eq!(date_rule(r"\d{2}", "%d").apply("hello-XY.org"), None);
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

    #[test]
    fn test_increment_rule() {
        let r1 = IncrementRule::new();

        assert_eq!(r1.apply("hello.txt"), None);
        assert_eq!(r1.apply(""), None);
        assert_eq!(r1.apply("hello3.txt"), Some("hello4.txt".to_string()));
        assert_eq!(r1.apply("hello7"), Some("hello8".to_string()));
    }
}
