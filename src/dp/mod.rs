pub mod rules;
pub mod vfs;

use log::warn;
use rules::Rule;
use vfs::Vfs;

/// Rule for renaming a file
/// `apply` method returns an optional renamed to new name if the rule is successful
pub struct Duplicator {
    rules: Vec<Box<dyn Rule>>,
    vfs: Box<dyn Vfs>,
    fallthrough: bool,
}

impl Duplicator {
    pub fn new(rules: Vec<Box<dyn Rule>>, vfs: Box<dyn Vfs>, fallthrough: bool) -> Self {
        Self {
            rules,
            vfs,
            fallthrough,
        }
    }

    fn duplicate_with_rules(&mut self, idx: usize, file: &str) -> bool {
        for i in idx..self.rules.len() {
            let rule = &self.rules[i];

            if let Some(ref renamed) = rule.apply(file) {
                if self.vfs.exists(&renamed) {
                    warn!("Renamed file already exists: {}", renamed);

                    if self.fallthrough {
                        return self.duplicate_with_rules(i + 1, renamed);
                    } else {
                        break;
                    }
                } else {
                    return self.vfs.copy(file, renamed);
                }
            }
        }

        false
    }

    pub fn duplicate(&mut self, file: &str) -> bool {
        self.duplicate_with_rules(0, file)
    }

    pub fn print_help(&self) {
        println!("=== Rules ===");
        for rule in &self.rules {
            rule.print()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;
    use std::iter::FromIterator;

    #[test]
    fn test_duplicator() {
        let r1 = rules::test::default_test_date_rule(r"\d{2}-\d{2}", "%m-%d");
        let r2 = rules::test::default_test_date_rule(r"\d{2}", "%d");

        let rules: Vec<Box<dyn Rule>> = vec![Box::new(r1), Box::new(r2)];

        let vfs = Box::new(vfs::test::TestFileSystem::new(vec![
            "hello-10-23.org".to_string(),
            "meeting-23.org".to_string(),
        ]));

        let mut duplicator = Duplicator::new(rules, vfs, false);

        assert!(duplicator.duplicate("hello-10-23.org"));
        assert!(duplicator.duplicate("meeting-23.org"));

        // Now same calls should fail
        assert!(!duplicator.duplicate("hello-10-23.org"));
        assert!(!duplicator.duplicate("meeting-23.org"));
    }

    #[test]
    fn test_fallthrough_rules() {
        let vfs = Box::new(vfs::test::TestFileSystem::new(vec![
            "hello-10-23.org".to_string(),
            "meeting-23.org".to_string(),
        ]));

        let r1 = rules::test::default_test_date_rule(r"\d{2}", "%d");
        let r2 = rules::IncrementRule::new();

        let mut duplicator = Duplicator::new(vec![Box::new(r1), Box::new(r2)], vfs, true);

        assert!(duplicator.duplicate("hello-10-23.org"));
        assert!(duplicator.duplicate("meeting-23.org"));
        assert!(duplicator.duplicate("meeting-11.org"));

        let files: HashSet<String> = HashSet::from_iter(duplicator.vfs.get_files().iter().cloned());

        assert!(files.contains("meeting-10.org"));
        assert!(files.contains("meeting-11.org"));
        assert!(files.contains("meeting-23.org"));
        assert!(files.contains("hello-10-23.org"));
        assert!(files.contains("hello-10-10.org"));
    }
}
