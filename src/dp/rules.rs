use chrono::{NaiveDateTime, Utc};
use regex::Captures;
use regex::Regex;

type DpDateTime = NaiveDateTime;

fn current_local_date_time() -> DpDateTime {
    Utc::now().naive_utc()
}

pub trait Rule {
    fn apply(&self, input: &str) -> Option<String>;
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

#[cfg(test)]
pub mod test {
    use super::*;
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

    fn local_date(year: i32, month: u32, date: u32) -> DpDateTime {
        NaiveDateTime::new(
            NaiveDate::from_ymd(year, month, date),
            NaiveTime::from_hms_milli(0, 0, 0, 0),
        )
    }

    pub fn default_test_date_rule(regex: &str, date_fmt: &str) -> DateRule {
        DateRule::compile(regex, date_fmt, test_local_date())
    }

    fn test_local_date() -> DpDateTime {
        local_date(2019, 11, 10)
    }

    #[test]
    fn test_date_rule() {
        assert_eq!(
            default_test_date_rule(r"\d{2}-\d{2}", "%m-%d").apply("hello-10-23.org"),
            Some("hello-11-10.org".to_string())
        );

        assert_eq!(
            default_test_date_rule(r"\d{2}", "%d").apply("hello-23.org"),
            Some("hello-10.org".to_string())
        );

        assert_eq!(
            default_test_date_rule(r"\d{2}-\d{2}", "%m-%d").apply("hello-there.org"),
            None
        );
        assert_eq!(
            default_test_date_rule(r"\d{2}", "%d").apply("hello-XY.org"),
            None
        );
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
