use core::net::Ipv4Addr;
use pnet::packet::ethernet::{EthernetPacket, MutableEthernetPacket};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::udp::MutableUdpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use pnet::util::MacAddr;
use shipcomponent::ShipComponent;
use std::str::FromStr;

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

        let mut ship_switch = hashbrown::HashMap::new();

        loop {
            unsafe {
                libc::poll(poll_fds.as_mut_ptr(), poll_fds.len() as _, -1);
            }

            // prepare the structure for the network traffic
            let mut ship_traffic: Vec<(usize, Vec<u8>, bool, String)> = Vec::new();

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
            self.send_traffic(ship_traffic, &ship_switch);

            self.components.iter_mut().for_each(|component| {
                component.refill_umem_allocator();
            });

            self.components.iter_mut().for_each(|component| {
                component.refill_fill_rings();
            });
        }
    }

    pub fn send_traffic(
        &mut self,
        ship_traffic: Vec<(usize, Vec<u8>, bool, String)>,
        ship_switch: &hashbrown::HashMap<[u8; 6], usize>,
    ) {
        println!("|-----[ TRAFFIC LOG ]")
        ship_traffic
            .iter()
            .for_each(|(destination_poll_fd_index, data, is_nmea, prefix)| {
                if *is_nmea {
                    // the nmea sentence should be multicasted to all ship's components that can receive it
                    for i in 0..self.components.len() {
                        if self.components[i].receives.contains(prefix) {
                            let destination_mac =
                                MacAddr::from_str(&self.components[i].mac).unwrap();
                            let destination_poll_fd_idx =
                                ship_switch.get(&destination_mac.octets());

                            match destination_poll_fd_idx {
                                Some(idx) => {
                                    let new_destination_poll_fd_index = idx;
                                    let destination_ip =
                                        Ipv4Addr::from_str(&self.components[i].ip).unwrap();

                                    // modify destination mac
                                    let ethernet = EthernetPacket::new(data).unwrap();
                                    let mut ethernet_buffer = vec![0u8; ethernet.packet().len()];
                                    let mut mutable_ethernet =
                                        MutableEthernetPacket::new(&mut ethernet_buffer).unwrap();
                                    mutable_ethernet.set_source(ethernet.get_source());
                                    mutable_ethernet.set_ethertype(ethernet.get_ethertype());
                                    mutable_ethernet.set_destination(destination_mac);

                                    // modify destination ip
                                    let ipv4 = Ipv4Packet::new(ethernet.payload()).unwrap();
                                    let mut ipv4_buffer = vec![0u8; ipv4.packet().len()];
                                    let mut mutable_ipv4 =
                                        MutableIpv4Packet::new(&mut ipv4_buffer).unwrap();
                                    mutable_ipv4.set_version(ipv4.get_version());
                                    mutable_ipv4.set_header_length(ipv4.get_header_length());
                                    mutable_ipv4.set_dscp(ipv4.get_dscp());
                                    mutable_ipv4.set_ecn(ipv4.get_ecn());
                                    mutable_ipv4.set_total_length(ipv4.get_total_length());
                                    mutable_ipv4.set_identification(ipv4.get_identification());
                                    mutable_ipv4.set_flags(ipv4.get_flags());
                                    mutable_ipv4.set_fragment_offset(ipv4.get_fragment_offset());
                                    mutable_ipv4.set_ttl(ipv4.get_ttl());
                                    mutable_ipv4
                                        .set_next_level_protocol(ipv4.get_next_level_protocol());
                                    mutable_ipv4.set_source(ipv4.get_source());
                                    mutable_ipv4.set_destination(destination_ip);

                                    // recalculate ip checksum
                                    mutable_ipv4.set_checksum(0);
                                    let checksum =
                                        pnet::packet::ipv4::checksum(&mutable_ipv4.to_immutable());
                                    mutable_ipv4.set_checksum(checksum);

                                    // get udp layer
                                    let udp = UdpPacket::new(mutable_ipv4.payload()).unwrap();
                                    let mut udp_buffer = vec![0u8; udp.packet().len()];
                                    let mut mutable_udp =
                                        MutableUdpPacket::new(&mut udp_buffer).unwrap();
                                    mutable_udp.set_source(udp.get_source());
                                    mutable_udp.set_destination(udp.get_destination());
                                    mutable_udp.set_length(udp.get_length());
                                    mutable_udp.set_payload(udp.payload());
                                    mutable_udp.set_checksum(0);
                                    let checksum = pnet::packet::udp::ipv4_checksum(
                                        &mutable_udp.to_immutable(),
                                        &mutable_ipv4.get_source(),
                                        &mutable_ipv4.get_destination(),
                                    );

                                    mutable_udp.set_checksum(checksum);
                                    mutable_ipv4.set_payload(mutable_udp.packet());
                                    mutable_ethernet.set_payload(mutable_ipv4.packet());
                                    let new_data = mutable_ethernet.packet().to_vec();
                                    println!(
                                        "MULTICASTING TO [ ifname : {} - mac : {:?} - ip: {:?}]",
                                        self.components[i].ifname, destination_mac, destination_ip
                                    );

                                    self.transmit(new_destination_poll_fd_index, &new_data);
                                }
                                None => {}
                            }
                        }
                    }
                } else {
                    // proceed with normal packet flow if the packet is not a nmea sentence
                    self.transmit(destination_poll_fd_index, data);
                }
            });
        println!("-------------------------------------");
    }

    fn transmit(&mut self, destination_poll_fd_index: &usize, data: &Vec<u8>) {
        let current_component = &mut self.components[*destination_poll_fd_index];
        match current_component.umem_allocator.try_allocate() {
            Some(chunk_index) => {
                // memory for transmission is allocated, needs to be set up
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

                // copy the data to transmit to the memory location
                tx_slice.copy_from_slice(data);
                current_component.sock.tx_ring.advance_producer_index();
                // actually sends the data
                match current_component.sock.wake_for_transmission() {
                    Ok(()) => {}
                    Err(_) => println!(
                        "\n\n[{}] packets_transmission = ko\n\n",
                        destination_poll_fd_index
                    ),
                }
            }

            None => println!("chunk_allocation = ko"),
        }
    }
}
