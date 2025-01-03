use crate::l7_components::ship_components::ShipComponent;
use crate::l7_components::ship_packet_parser::ShipPacketParser;
use std::collections::HashMap;

pub struct Ship<'a> {
    components: Vec<ShipComponent<'a>>,
}

impl<'a> Ship<'a> {
    pub fn new(components: Vec<ShipComponent<'a>>) -> Self {
        Ship { components }
    }

    pub fn monitor_components(&mut self) {
        let mut poll_fds = vec![];
        self.components.iter().for_each(|component| {
            poll_fds.push(component.poll_fd);
        });

        loop {
            unsafe {
                libc::poll(poll_fds.as_mut_ptr(), poll_fds.len() as _, -1);
            }

            for (i, _) in poll_fds
                .iter()
                .enumerate()
                .filter(|(_, fd)| fd.revents & libc::POLLIN != 0)
            {
                println!("----------------------");
                println!("[ IFACE {i} ]---[ {} ]", self.components[i].name);

                let current_component = &mut self.components[i];

                while current_component.sock.rx_ring.can_consume() {
                    // process inbound packet
                    let rx_descriptor = current_component.sock.rx_ring.get_nth_descriptor(
                        current_component.sock.rx_ring.get_consumer_index() as _,
                    );
                    let rx_slice = current_component.sock.rx_ring.get_nth_slice(
                        current_component.sock.rx_ring.get_consumer_index() as _,
                        &current_component.sock.umem,
                    );

                    let parser: ShipPacketParser = ShipPacketParser::new(rx_slice);
                    parser.check_switch(i);
                    parser.parse();

                    // refill allocator or fill ring
                    if current_component.sock.fill_ring.can_produce() {
                        current_component
                            .sock
                            .fill_ring
                            .produce_umem_offset(rx_descriptor.addr);
                    } else {
                        current_component
                            .umem_allocator
                            .release_offset(rx_descriptor.addr);
                    }

                    // advance index
                    current_component.sock.rx_ring.advance_consumer_index();

                    println!("----------------------")
                }
            }

            self.components.iter_mut().for_each(|component| {
                component.refill_umem_allocator();
            });

            self.components.iter_mut().for_each(|component| {
                component.refill_fill_rings();
            });
        }
    }
}
