mod dp;

use clap::{App, Arg};
use dp::rules::{DateRule, IncrementRule, Rule};
use dp::vfs::LocalFileSystem;
use dp::Duplicator;
use log::error;

fn duplicate_files(files: &[&str], print_rules: bool) {
    // Dash (-) separated dates
    let r1 = DateRule::compile_now(r"\d{2}-\d{2}", "%m-%d");
    let r2 = DateRule::compile_now(r"\d{4}-\d{2}-\d{2}", "%y-%m-%d");
    // Dash (_) separated dates
    let r3 = DateRule::compile_now(r"\d{2}_\d{2}", "%m_%d");
    let r4 = DateRule::compile_now(r"\d{4}_\d{2}_\d{2}", "%y_%m_%d");

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

    let mut duplicator = Duplicator::new(rules, fs, false);

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
    let inputs: Vec<_> = matches.values_of("input").unwrap().collect();

    stderrlog::new()
        .module(module_path!())
        .verbosity(verbosity)
        .init()
        .unwrap();

    let print_rules = matches.is_present("rules");
    duplicate_files(&inputs, print_rules);
}
