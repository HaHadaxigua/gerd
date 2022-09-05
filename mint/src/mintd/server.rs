use std::{path::PathBuf};
use home;

struct Server {
    data_dir: PathBuf,
}

impl Default for Server {
    fn default() -> Self {
        Server {
            data_dir: PathBuf::from(home::home_dir().
                expect("cannot load home directory")).
                join("mint")

        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use home::home_dir;

    #[test]
    fn get_home_dir() {
        match home::home_dir() {
            Some(path) => println!("{}", path.display()),
            None => println!("Impossible to get your home dir!"),
        }
    }
}