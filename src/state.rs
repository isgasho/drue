use std::fmt;

#[derive(Debug)]
pub struct State {
    pub hits: u8,
    pub start_timestamp: u64,
    pub last_hpm: u64,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "timestamp: {} hits: {} last_hpm: {} ",
            self.start_timestamp, self.hits, self.last_hpm
        )
    }
}
