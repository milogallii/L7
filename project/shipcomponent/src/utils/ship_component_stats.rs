use std::collections::VecDeque;

pub struct ShipComponentStats {
    pub current_packets_received: usize,
    pub current_total_received: usize,
    pub current_packets_sent: usize,
    pub current_total_sent: usize,
    pub packets_received: VecDeque<usize>,
    pub packets_sent: VecDeque<usize>,
    pub total_received: VecDeque<usize>,
    pub total_sent: VecDeque<usize>,
}

impl ShipComponentStats {
    pub fn new() -> Self {
        ShipComponentStats {
            current_packets_received: 0,
            current_total_received: 0,
            current_packets_sent: 0,
            current_total_sent: 0,
            packets_received: VecDeque::new(),
            packets_sent: VecDeque::new(),
            total_received: VecDeque::new(),
            total_sent: VecDeque::new(),
        }
    }
}
