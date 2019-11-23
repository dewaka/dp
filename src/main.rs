#[macro_use]
extern crate lazy_static;

mod dp;

use dp::rules::{DateRule, IncrementRule, Rule};
use dp::vfs::LocalFileSystem;
use dp::Duplicator;
use log::error;
use structopt::StructOpt;

fn get_duplicator(fallthrough: bool) -> Duplicator {
    let now = dp::rules::current_local_date_time();

    // Dash (-) separated dates
    let r1 = DateRule::compile(r"\d{2}-\d{2}", "%m-%d", now);
    let r2 = DateRule::compile(r"\d{4}-\d{2}-\d{2}", "%y-%m-%d", now);
    // Dash (_) separated dates
    let r3 = DateRule::compile(r"\d{2}_\d{2}", "%m_%d", now);
    let r4 = DateRule::compile(r"\d{4}_\d{2}_\d{2}", "%y_%m_%d", now);

    // Increment rule
    let inc_rule = IncrementRule::new();

    let rules: Vec<Box<dyn Rule>> = vec![
        Box::new(r1),
        Box::new(r2),
        Box::new(r3),
        Box::new(r4),
        Box::new(inc_rule),
    ];

    let fs = Box::new(LocalFileSystem::new());

    Duplicator::new(rules, fs, fallthrough)
}

fn duplicate_files<T: AsRef<str>>(files: &[T], fallthrough: bool, print_rules: bool) {
    let mut duplicator = get_duplicator(fallthrough);

    if print_rules {
        duplicator.print_help();
    } else {
        for file in files.iter() {
            if !duplicator.duplicate(file.as_ref()) {
                error!("File duplication failed for: {}", file.as_ref());
            }
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "dp - file duplicator",
    about = "Duplicates files",
    author = "Chathura Colombage <dcdewaka@gmail.com>"
)]
struct Opt {
    /// Optional argument to print rules
    #[structopt(long = "rules", short = "r", help = "Printing renaming rules")]
    rules: bool,

    /// Optional argument for fallthrough of renaming rules
    #[structopt(
        long = "fallthrough",
        short = "f",
        help = "Fallthrough renaming patterns when a matched renaming rule fails"
    )]
    fallthrough: bool,

    /// Files to duplicate
    files: Vec<String>,

    /// Controls the log verbosity
    #[structopt(short = "v", parse(from_occurrences))]
    verbosity: usize,
}

fn main() {
    let opt: Opt = Opt::from_args();

    stderrlog::new()
        .module(module_path!())
        .verbosity(opt.verbosity)
        .init()
        .unwrap();

    duplicate_files(&opt.files, opt.fallthrough, opt.rules);
}
