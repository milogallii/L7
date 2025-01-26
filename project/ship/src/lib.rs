use pnet::packet::ethernet::{EthernetPacket, MutableEthernetPacket};
use pnet::packet::ipv4::{Ipv4Packet, MutableIpv4Packet};
use pnet::packet::udp::{MutableUdpPacket, UdpPacket};
use pnet::packet::{MutablePacket, Packet};
use pnet::util::MacAddr;
use shipcomponent::ShipComponent;
use std::collections::VecDeque;
use std::net::Ipv4Addr;
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
            self.send_traffic(&ship_traffic, &ship_switch);

            self.components.iter_mut().for_each(|component| {
                component.refill_umem_allocator();
            });

            self.components.iter_mut().for_each(|component| {
                component.refill_fill_ring();
            });
        }
    }

    pub fn send_traffic(
        &mut self,
        ship_traffic: &Vec<(usize, Vec<u8>, bool, String)>,
        ship_switch: &hashbrown::HashMap<[u8; 6], usize>,
    ) {
        // println!("|-----[ TRAFFIC LOG ]");
        ship_traffic
            .iter()
            .for_each(|(destination_poll_fd_index, data, is_nmea, prefix)| {
                if *is_nmea {
                    // println!("| FILTERED AND MULTICAST FLOW");
                    // the nmea sentence should be multicasted to all ship's components that can receive it
                    self.transmit_multicast(data, ship_switch, prefix);
                } else {
                    // println!("| NORMAL FLOW");
                    // proceed with normal packet flow if the packet is not a nmea sentence
                    self.transmit(destination_poll_fd_index, data);
                }
            });

        // println!("|-----[SWITCH STATE]");
        // ship_switch
        //     .iter()
        //     .for_each(|(address, sock)| println!("| [ {:x?} - {} ]", address, sock));
        // println!("-------------------------------------");
    }

    fn transmit_multicast(
        &mut self,
        data: &Vec<u8>,
        ship_switch: &hashbrown::HashMap<[u8; 6], usize>,
        prefix: &String,
    ) {
        for i in 0..self.components.len() {
            if self.components[i].receives.contains(prefix) {
                let destination_mac = MacAddr::from_str(&self.components[i].mac);

                match destination_mac {
                    Ok(destination_mac) => {
                        let destination_poll_fd_idx = ship_switch.get(&destination_mac.octets());
                        match destination_poll_fd_idx {
                            Some(new_destination_poll_fd_index) => {
                                // println!("| MULTICASTING TO [ {} ]", self.components[i].ifname,);
                                let new_packet = self.forge_packet(&data, destination_mac, i);
                                if new_packet.is_empty() {
                                    return;
                                }
                                self.transmit(new_destination_poll_fd_index, &new_packet);
                            }
                            None => {}
                        }
                    }
                    Err(_) => {}
                }
            }
        }
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
                tx_slice.copy_from_slice(&data);
                current_component.sock.tx_ring.advance_producer_index();
                // actually sends the data
                match current_component.sock.wake_for_transmission() {
                    Ok(()) => {} //println!("| TRANSMISSION USING SOCK {} SUCCESSFULL",*destination_poll_fd_index),
                    Err(_) => println!(
                        "| TRANSMISSION USING SOCK {} FAILED",
                        destination_poll_fd_index
                    ),
                }
            }

            None => println!("| MEMORY ALLOCATION FOR TRANSMISSION FAILED"),
        }
    }

    fn forge_packet(
        &self,
        data: &Vec<u8>,
        destination_mac: MacAddr,
        current_component_index: usize,
    ) -> Vec<u8> {
        // Check if the packet is a ethernet packet
        match EthernetPacket::new(&data) {
            Some(ethernet_packet) => {
                let mut new_packet_buffer = vec![0u8; data.len()];
                // check if forging a new mutable error generates a error
                match MutableEthernetPacket::new(&mut new_packet_buffer) {
                    Some(mut new_ethernet_packet) => {
                        new_ethernet_packet.clone_from(&ethernet_packet);
                        new_ethernet_packet.set_destination(destination_mac);
                        let ipv4_payload = new_ethernet_packet.payload().to_vec();

                        // check if the packet is a valid ipv4 packet
                        match Ipv4Packet::new(&ipv4_payload) {
                            Some(ipv4_packet) => {
                                let mut ipv4_buffer = vec![0u8; ipv4_payload.len()];
                                // check if forging a new ipv4 packet generates a error
                                match MutableIpv4Packet::new(&mut ipv4_buffer) {
                                    Some(mut new_ipv4_packet) => {
                                        new_ipv4_packet.clone_from(&ipv4_packet);
                                        match Ipv4Addr::from_str(
                                            &self.components[current_component_index].ip,
                                        ) {
                                            Ok(destination_ip) => {
                                                new_ipv4_packet.set_destination(destination_ip);
                                                // Recalculate the IPv4 checksum
                                                new_ipv4_packet.set_checksum(0); // Reset checksum before calculation
                                                let checksum = pnet::packet::ipv4::checksum(
                                                    &new_ipv4_packet.to_immutable(),
                                                );
                                                new_ipv4_packet.set_checksum(checksum);

                                                let udp_payload =
                                                    new_ipv4_packet.payload().to_vec();
                                                match UdpPacket::new(&udp_payload) {
                                                    Some(udp_packet) => {
                                                        let mut udp_buffer =
                                                            vec![0u8; udp_payload.len()];
                                                        match MutableUdpPacket::new(&mut udp_buffer)
                                                        {
                                                            Some(mut new_udp_packet) => {
                                                                new_udp_packet
                                                                    .clone_from(&udp_packet);
                                                                new_udp_packet.set_checksum(0);
                                                                let udp_checksum = pnet::packet::udp::ipv4_checksum(
                                                                    &new_udp_packet.to_immutable(),
                                                                    &new_ipv4_packet.get_source(),
                                                                    &new_ipv4_packet.get_destination(),);

                                                                new_udp_packet
                                                                    .set_checksum(udp_checksum);

                                                                // Forge the final packet
                                                                new_ipv4_packet.set_payload(
                                                                    new_udp_packet.packet(),
                                                                );
                                                                new_ethernet_packet.set_payload(
                                                                    new_ipv4_packet.packet(),
                                                                );

                                                                return new_packet_buffer;
                                                            }
                                                            None => {
                                                                println!("ERROR CREATING IPV4 PACKET FOR MULTICAST");
                                                                return Vec::new();
                                                            }
                                                        }
                                                    }
                                                    None => {
                                                        println!("NMEA MESSAGE NOT VALID FOR TRANSMISSION - NO VALID UDP PACKET");
                                                        return Vec::new();
                                                    }
                                                }
                                            }
                                            Err(_) => {
                                                println!("ERROR GETTING NEW DESTINATION IP");
                                                return Vec::new();
                                            }
                                        }
                                    }
                                    None => {
                                        println!("ERROR CREATING IPV4 PACKET FOR MULTICAST");
                                        return Vec::new();
                                    }
                                }
                            }

                            None => {
                                println!(
                                "NMEA MESSAGE NOT VALID FOR TRANSMISSION - NO VALID IPV4 PACKET"
                            );
                                return Vec::new();
                            }
                        }
                    }
                    None => {
                        println!("ERROR CREATING ETHERNET PACKET FOR MULTICAST");
                        return Vec::new();
                    }
                }
            }
            None => {
                println!("NMEA MESSAGE NOT VALID FOR TRANSMISSION - NO VALID ETHERNET PACKET");
                return Vec::new();
            }
        }
    }
}
