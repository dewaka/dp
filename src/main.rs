mod lib;

use lib::{DateRule, Duplicator, LocalFileSystem, Rule};

fn duplicate_files(files: &[String]) {
    // Dash (-) separated dates
    let r1 = DateRule::compile_now(r"\d{2}-\d{2}", "%m-%d");
    let r2 = DateRule::compile_now(r"\d{4}-\d{2}-\d{2}", "%y-%m-%d");
    // Dash (_) separated dates
    let r3 = DateRule::compile_now(r"\d{2}_\d{2}", "%m_%d");
    let r4 = DateRule::compile_now(r"\d{4}_\d{2}_\d{2}", "%y_%m_%d");

    let rules: Vec<&dyn Rule> = vec![&r1, &r2, &r3, &r4];
    let mut fs = LocalFileSystem::new();

    let mut duplicator = Duplicator::new(rules, &mut fs);

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
