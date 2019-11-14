pub mod rules;
pub mod vfs;

use rules::Rule;
use vfs::Vfs;

/// Rule for renaming a file
/// `apply` method returns an optional renamed to new name if the rule is successful
pub struct Duplicator {
    rules: Vec<Box<dyn Rule>>,
    vfs: Box<dyn Vfs>,
}

impl Duplicator {
    pub fn new(rules: Vec<Box<dyn Rule>>, vfs: Box<dyn Vfs>) -> Self {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_duplicator() {
        let r1 = rules::test::date_rule(r"\d{2}-\d{2}", "%m-%d");
        let r2 = rules::test::date_rule(r"\d{2}", "%d");

        let rules: Vec<Box<dyn Rule>> = vec![Box::new(r1), Box::new(r2)];

        let vfs = Box::new(vfs::test::TestFileSystem::new(vec![
            "hello-10-23.org".to_string(),
            "meeting-23.org".to_string(),
        ]));

        let mut duplicator = Duplicator::new(rules, vfs);

        assert!(duplicator.duplicate("hello-10-23.org"));
        assert!(duplicator.duplicate("meeting-23.org"));

        // Now same calls should fail
        //        assert_eq!(duplicator.duplicate("hello-10-23.org"), false);
        //        assert!(!duplicator.duplicate("meeting-23.org"));
    }
}
