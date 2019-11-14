mod dp;

use dp::rules::{DateRule, IncrementRule, Rule};
use dp::vfs::LocalFileSystem;
use dp::Duplicator;

fn duplicate_files(files: &[String]) {
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

    let mut duplicator = Duplicator::new(rules, fs);

    for file in files.iter() {
        if !duplicator.duplicate(&file) {
            println!("Error - file duplication failed for: {}", file);
        }
    }
}

fn print_usage() {
    println!("dp -- duplicate files");
    println!("dp <file> ...");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        duplicate_files(&args[1..])
    } else {
        print_usage();
        std::process::exit(1);
    }
}
