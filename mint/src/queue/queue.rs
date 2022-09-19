use std::{
    fmt::{
        self,
        Display,
        Formatter,
    },
    path,
    sync::{
        atomic,
        atomic::Ordering,
    },
};

use regex::Regex;


// Queue represents a persistent FIFO structure, that stores the data in leveldb
struct Queue {
    pub name: String,
    pub data_dir: path::PathBuf,
    stats: Stats,
    db: sled::Db,
    opts: Options,
    head: u64,
    tail: u64,
    is_opened: bool,
    is_shared: bool,
}


impl Queue {
    pub fn open(name: String, data_dir: String, opts: Options) -> Result<Queue, Box<dyn std::error::Error>> {
        Ok(Queue {
            name: name.clone(),
            data_dir: path::PathBuf::from(data_dir.as_str()),
            stats: Stats::default(),
            db: sled::open(format!("{}/{}", data_dir.as_str(), name.as_str()))?,
            opts,
            head: 0,
            tail: 0,
            is_opened: true,
            is_shared: false,
        })
    }

    fn _open(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let re = Regex::new(r#"[^a-zA-Z0-9_\-\:]+"#).expect("invalid regular expression");
        if !re.is_match(name.as_ref()) {
            return Err(Box::new(QueueError::InvalidQueueName(name.clone())));
        }
        if name.len() > 100 {
            return Err(Box::new(QueueError::NameTooLong(name.clone())));
        }
        Ok(())
    }

    fn path(&mut self) -> String {
        String::from(format!("{}/{}", self.data_dir.clone().to_str().unwrap(), self.name.clone()).as_str())
    }
}

#[derive(Debug)]
struct Stats {
    pub open_reads: atomic::AtomicI64,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            open_reads: atomic::AtomicI64::new(0),
        }
    }
}

impl Stats {
    pub fn update_open_reads(&mut self, value: i64) {
        self.open_reads.fetch_add(value, Ordering::SeqCst);
    }
}

struct Options {
    key_prefix: Vec<u8>,
}

/// Item represents a queue item
struct Item {
    id: u64,
    key: Vec<u8>,
    value: Vec<u8>,
}

#[derive(Debug, Clone)]
enum QueueError {
    InvalidQueueName(String),
    NameTooLong(String),
}

impl ::std::error::Error for QueueError {}

impl Display for QueueError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            QueueError::NameTooLong(v) => {
                write!(f, "given name is exceed the 100 length limit {}", v)
            }
            QueueError::InvalidQueueName(v) => {
                write!(f, "contains invalid character in {}", v)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_stats() {
        let mut stats = Stats::default();
        stats.update_open_reads(1);
        assert_eq!(1, stats.open_reads.load(Ordering::SeqCst))
    }
}