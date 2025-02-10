use std::collections::VecDeque;
use std::time::Duration;

pub struct ShipComponentStats {
    pub total_transmitted: VecDeque<usize>,
    pub total_sent: VecDeque<usize>,
    pub times_elapsed_sent: VecDeque<Duration>,
    pub times_elapsed_transmitted: VecDeque<Duration>,
}

impl ShipComponentStats {
    pub fn new() -> Self {
        let mut total_transmitted: VecDeque<usize> = VecDeque::new();
        let mut total_sent: VecDeque<usize> = VecDeque::new();
        let mut times_elapsed_sent: VecDeque<Duration> = VecDeque::new();
        let mut times_elapsed_transmitted: VecDeque<Duration> = VecDeque::new();

        total_sent.push_back(0);
        total_transmitted.push_back(0);
        times_elapsed_sent.push_back(Duration::from_secs(0));
        times_elapsed_transmitted.push_back(Duration::from_secs(0));

        ShipComponentStats {
            total_transmitted,
            total_sent,
            times_elapsed_sent,
            times_elapsed_transmitted,
        }
    }

    pub fn show(&self) {
        println!(
            "|-- total sent:  {} {:?}",
            self.total_sent[self.total_sent.len() - 1],
            self.times_elapsed_sent[self.times_elapsed_sent.len() - 1]
        );
        println!(
            "|-- total transmitted:  {} {:?}",
            self.total_transmitted[self.total_transmitted.len() - 1],
            self.times_elapsed_transmitted[self.times_elapsed_transmitted.len() - 1]
        );
    }
}
