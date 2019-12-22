use std::fmt;

#[derive(Debug)]
pub struct State {
    pub hits: usize,
    pub start_timestamp: u64,
    pub last_hpm: u64,
}

impl State {
    pub fn time_since_last(&self, stamp: u64) -> f64 {
        // NOTE: Don't know if the sizes are right
        (stamp as f64 - self.start_timestamp as f64) / 1000000.0
    }
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
