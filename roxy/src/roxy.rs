use std::{
    fs,
    io::{self, BufRead, BufReader},
    path::PathBuf,
    process::exit,
};

pub struct Roxy {
    had_error: bool,
}

impl Roxy {
    pub fn new() -> Self {
        Roxy { had_error: false }
    }
    pub fn run_file(&mut self, path: &PathBuf) {
        if !path.exists() {
            panic!("specified {} not existed", (*path).display())
        }
        let code = fs::read_to_string(path).unwrap();
        self.run(code);
        if self.had_error {
            exit(65);
        }
    }

    pub fn run_prompt(&mut self) {
        let mut reader = BufReader::new(io::stdin());
        loop {
            println!("> ");
            let mut line = String::new();
            reader.read_line(&mut line).expect("failed scan to string");
            if line.is_empty() {
                break;
            }
            self.run(line);
            self.had_error = false;
        }
    }

    fn run(&mut self, code: String) {
        // let mut reader = BufReader::new(code);
        // let mut tokens: Vec<Token> = Vec::new();
    }
}

fn report(line_cnt: usize, location: &str, message: &str) {
    eprintln!("[line {} ] Error {}: {} ", line_cnt, location, message)
}
