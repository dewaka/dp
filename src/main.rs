#[macro_use]
extern crate lazy_static;

mod dp;

use clap::{App, Arg};
use dp::rules::{DateRule, IncrementRule, Rule};
use dp::vfs::LocalFileSystem;
use dp::Duplicator;
use log::error;

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

fn duplicate_files(files: &[&str], fallthrough: bool, print_rules: bool) {
    let mut duplicator = get_duplicator(fallthrough);

    if print_rules {
        duplicator.print_help();
    } else {
        for file in files.iter() {
            if !duplicator.duplicate(&file) {
                error!("File duplication failed for: {}", file);
            }
        }
    }
}

fn main() {
    let matches = App::new("dp - file duplicator")
        .version("0.1")
        .author("Chathura Colombage <dcdewaka@gmail.com>")
        .about("Duplicates files")
        .arg(
            Arg::with_name("input")
                .help("Sets the input file to use")
                .multiple(true)
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("fallthrough")
                .short("f")
                .help("Fallthrough renaming patterns when a matched renaming rule fails")
                .multiple(false),
        )
        .arg(
            Arg::with_name("rules")
                .short("r")
                .help("Print duplication rules")
                .multiple(false),
        )
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Increase message verbosity"),
        )
        .get_matches();

    let verbosity = matches.occurrences_of("verbosity") as usize;
    let fallthrough = matches.is_present("fallthrough");
    let inputs: Vec<_> = matches.values_of("input").unwrap().collect();
    let print_rules = matches.is_present("rules");

    stderrlog::new()
        .module(module_path!())
        .verbosity(verbosity)
        .init()
        .unwrap();

    duplicate_files(&inputs, fallthrough, print_rules);
}
