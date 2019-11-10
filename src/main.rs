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

trait Vfs {
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

trait Rule {
    fn apply(&self, input: &str) -> Option<String>;
}

struct Duplicator<'a> {
    rules: Vec<&'a dyn Rule>,
    vfs: &'a mut dyn Vfs,
}

impl<'a> Duplicator<'a> {
    fn new(rules: Vec<&'a dyn Rule>, vfs: &'a mut dyn Vfs) -> Self {
        Self { rules, vfs }
    }

    fn duplicate(&mut self, file: &str) -> bool {
        for rule in &self.rules {
            match rule.apply(file) {
                Some(ref renamed) => {
                    println!("Renaming file: {} => {}", file, renamed);
                    return self.vfs.copy(file, renamed);
                }
                None => (),
            }
        }

        false
    }
}

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

fn main() {}

mod test {
    use super::*;

    fn test_local_date() -> DpDateTime {
        local_date(2019, 11, 10)
    }

    #[test]
    fn test_duplicator() {
        let r1: &dyn Rule = &DateRule::compile(r"\d{2}-\d{2}", "%m-%d", test_local_date());
        let rules = vec![r1];
        let mut vfs = TestFileSystem::new(vec!["hello-10-23.org".to_string()]);
        let mut duplicator = Duplicator::new(rules, &mut vfs);

        assert!(duplicator.duplicate("hello-10-23.org"));
        assert!(vfs.exists("hello-11-10.org"));
    }

    #[test]
    fn test_date_rule() {
        let r1 = DateRule::compile(r"\d{2}-\d{2}", "%m-%d", test_local_date());
        let renamed = r1.apply("hello-10-23.org");

        assert_eq!(renamed, Some("hello-11-10.org".to_string()));
    }
}
