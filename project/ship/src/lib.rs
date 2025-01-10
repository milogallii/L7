use hashbrown::HashMap;
use shipcomponent::ShipComponent;

pub struct Ship<'a> {
    components: Vec<ShipComponent<'a>>,
}

impl<'a> Ship<'a> {
    pub fn new(components: Vec<ShipComponent<'a>>) -> Self {
        Ship { components }
    }

    pub fn monitor_network(&mut self) {
        let mut poll_fds: Vec<libc::pollfd> = Vec::new();
        self.components.iter().for_each(|component| {
            poll_fds.push(component.poll_fd);
        });

        let mut ship_switch = HashMap::new();

        loop {
            unsafe {
                libc::poll(poll_fds.as_mut_ptr(), poll_fds.len() as _, -1);
            }

            // prepare the structure for the network traffic
            let mut ship_traffic: Vec<(usize, Vec<u8>)> = Vec::new();

            for (poll_fd_index, _) in poll_fds
                .iter()
                .enumerate()
                .filter(|(_, fd)| fd.revents & libc::POLLIN != 0)
            {
                // check every component for traffic to analyse
                let current_component = &mut self.components[poll_fd_index];
                while current_component.sock.rx_ring.can_consume() {
                    current_component.consume_rx_ring(
                        poll_fd_index,
                        poll_fds.len(),
                        &mut ship_traffic,
                        &mut ship_switch,
                    );
                }
            }

            // send the ship traffic according to each component's policy
            self.send_traffic(ship_traffic);

            self.components.iter_mut().for_each(|component| {
                component.refill_umem_allocator();
            });

            self.components.iter_mut().for_each(|component| {
                component.refill_fill_rings();
            });
        }
    }

    pub fn send_traffic(&mut self, ship_traffic: Vec<(usize, Vec<u8>)>) {
        for (out_sock_id, data) in ship_traffic {
            let current_component = &mut self.components[out_sock_id];
            match current_component.umem_allocator.try_allocate() {
                Some(chunk_index) => {
                    let tx_offset = current_component
                        .sock
                        .umem
                        .chunk_start_offset_for_index(chunk_index);
                    let tx_slice = current_component.sock.tx_ring.get_nth_slice_mut(
                        current_component.sock.tx_ring.get_producer_index() as _,
                        &current_component.sock.umem,
                        Some(tx_offset),
                        Some(data.len() as _),
                    );

                    tx_slice.copy_from_slice(&data);
                    current_component.sock.tx_ring.advance_producer_index();
                    current_component.sock.wake_for_transmission().unwrap();
                }

                None => println!("ERROR SENDING TRAFFIC"),
            }
        }
    }
}
