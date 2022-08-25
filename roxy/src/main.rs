mod err;
mod roxy;
mod scanner;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct RoxyArgs {
    #[clap(short, long, value_parser)]
    file: Option<PathBuf>,
}

fn main() {
    let args = RoxyArgs::parse();
    let mut roxy = roxy::Roxy::new();
    match args.file {
        None => {
            println!("run prompt!");
            roxy.run_prompt();
        }
        Some(path) => {
            println!("run file: {}", path.display());
            roxy.run_file(&path)
        }
    }
}
