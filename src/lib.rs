use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
use regex::Captures;
use regex::Regex;
use std::collections::HashSet;

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
/// - Check if a file exist
/// - Copy an existing file with a new name
pub trait Vfs {
    fn exists(&self, file: &str) -> bool;
    fn copy(&mut self, file: &str, new_file: &str) -> bool;
}

struct TestFileSystem {
    files: HashSet<String>,
}

impl TestFileSystem {
    fn new(files: Vec<String>) -> Self {
        Self {
            files: files.into_iter().collect(),
        }
    }

    fn print_files(&self) {
        println!("Files: {:?}", self.files);
    }
}

impl Vfs for TestFileSystem {
    fn exists(&self, file: &str) -> bool {
        self.files.contains(file)
    }

    fn copy(&mut self, _file: &str, new_file: &str) -> bool {
        self.files.insert(new_file.to_string());
        true
    }
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
    fn new(rules: Vec<&'a dyn Rule>, vfs: &'a mut dyn Vfs) -> Self {
        Self { rules, vfs }
    }

    fn duplicate(&mut self, file: &str) -> bool {
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
struct DateRule {
    regex: Regex,
    date_fmt: String,
    now: DpDateTime,
}

impl DateRule {
    fn new(regex: Regex, date_fmt: &str, now: DpDateTime) -> Self {
        Self {
            regex,
            date_fmt: date_fmt.to_string(),
            now,
        }
    }

    fn compile(pattern: &str, date_fmt: &str, now: DpDateTime) -> Self {
        let regex_str = format!("(.*)({})(.*)", pattern);
        let regex: Regex = Regex::new(&regex_str).unwrap();
        Self::new(regex, date_fmt, now)
    }

    fn compile_now(pattern: &str, date_fmt: &str) -> Self {
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
struct IncrementRule<'a> {
    vfs: &'a mut dyn Vfs,
}

impl<'a> Rule for IncrementRule<'a> {
    fn apply(&self, input: &str) -> Option<String> {
        unimplemented!()
    }
}

mod test {
    use super::*;

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
    }
}
