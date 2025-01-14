use core::net::Ipv4Addr;
use nmea::Nmea;
use packet_parser::PacketParser;
use pnet::packet::ethernet::MutableEthernetPacket;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::Packet;
use pnet::util::MacAddr;
use std::sync::Arc;
use std::{os::fd::AsRawFd, str::FromStr};
use xdrippi::{utils::interface_name_to_index, BPFRedirectManager, Umem, UmemAllocator, XDPSocket};

struct ShipNeighbour {
    mac: MacAddr,
    ip: Ipv4Addr,
}

pub struct ShipComponent<'a> {
    pub name: String,
    pub ifname: String,
    pub mac: String,
    pub ip: String,
    pub ifindex: libc::c_uint,
    pub bpf_manager: BPFRedirectManager,
    pub sock: XDPSocket<'a>,
    pub umem_allocator: UmemAllocator,
    pub poll_fd: libc::pollfd,
    pub sends: Vec<String>,
    pub receives: Vec<String>,
    neighbours: Vec<ShipNeighbour>,
}

impl ShipComponent<'_> {
    pub fn new(
        name: String,
        ifname: String,
        mac: String,
        ip: String,
        sends: Vec<String>,
        receives: Vec<String>,
        talks_to_macs: Vec<String>,
        talks_to_ips: Vec<String>,
    ) -> Self {
        // Getting interface index
        let ifindex = interface_name_to_index(ifname.as_str()).unwrap();

        // Setting up umem
        let umem = Umem::new_2k(16384).unwrap();
        let umem = Arc::new(umem);

        // Setting up xsk
        let mut sock = XDPSocket::new(ifindex, 0, umem.clone(), 4096).unwrap();
        let mut bpf_manager = BPFRedirectManager::attach(ifindex);
        bpf_manager.add_redirect(0, sock.as_raw_fd());

        // setting up the memory allocator for the rings
        let umem_allocator = UmemAllocator::for_umem(umem.clone());

        // setting up the fill ring
        while let Some(chunk_index) = umem_allocator.try_allocate() {
            if sock.fill_ring.can_produce() {
                sock.fill_ring
                    .produce_umem_offset(sock.umem.chunk_start_offset_for_index(chunk_index));
            } else {
                umem_allocator.release(chunk_index);
                break;
            }
        }

        let poll_fd = libc::pollfd {
            fd: sock.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };

        let mut neighbours = Vec::new();

        for i in 0..talks_to_macs.len() {
            let mac = MacAddr::from_str(&talks_to_macs[i]).unwrap();
            let ip = Ipv4Addr::from_str(&talks_to_ips[i]).unwrap();
            neighbours.push(ShipNeighbour { mac, ip });
        }

        ShipComponent {
            name,
            ifname,
            mac,
            ip,
            ifindex,
            bpf_manager,
            sock,
            umem_allocator,
            poll_fd,
            sends,
            receives,
            neighbours,
        }
    }

    pub fn consume_rx_ring(
        &mut self,
        poll_fd_index: usize,
        poll_fds_len: usize,
        ship_traffic: &mut Vec<(usize, Vec<u8>)>,
        ship_switch: &mut hashbrown::HashMap<[u8; 6], usize>,
    ) {
        println!(
            "[INTERFACE {} : {} ]---[ {} ]---[ sending ]",
            self.ifindex, self.ifname, self.name
        );

        let rx_descriptor = self
            .sock
            .rx_ring
            .get_nth_descriptor(self.sock.rx_ring.get_consumer_index() as _);

        let rx_slice = self
            .sock
            .rx_ring
            .get_nth_slice(self.sock.rx_ring.get_consumer_index() as _, &self.sock.umem);

        // Parse the incoming message
        let packet_parser = PacketParser::new(rx_slice);
        let mut message_ok: bool = true;
        let mut is_nmea: bool = false;
        let mut prefix: String = String::from("NONMEA");

        match packet_parser.parse_traffic() {
            Ok(message) => (message_ok, is_nmea, prefix) = self.apply_policy(message),
            Err(_) => {}
        }

        if message_ok {
            self.handle_network(
                rx_slice,
                ship_switch,
                &poll_fd_index,
                &poll_fds_len,
                ship_traffic,
                is_nmea,
                prefix,
            );
        } else {
            println!("|-- MESSAGE IS NOT A NMEA SENTENCE OR IS NOT ALLOWED ");
            println!("|-- REC ALLOWED {:?}", self.receives);
            println!("|-- SND ALLOWED {:?}", self.sends);
            let mac_neighbours: Vec<MacAddr> = self
                .neighbours
                .iter()
                .map(|neighbour| neighbour.mac)
                .collect();
            println!("|-- MAC NEIGHBOURS {:?}", mac_neighbours);
            let ip_neighbours: Vec<Ipv4Addr> = self
                .neighbours
                .iter()
                .map(|neighbour| neighbour.ip)
                .collect();
            println!("|-- IP NEIGHBOURS {:?}", ip_neighbours);
        }

        // refill allocator or fill ring
        if self.sock.fill_ring.can_produce() {
            self.sock.fill_ring.produce_umem_offset(rx_descriptor.addr);
        } else {
            self.umem_allocator.release_offset(rx_descriptor.addr);
        }

        // advance index
        self.sock.rx_ring.advance_consumer_index();

        println!("\n----------------------\n")
    }

    fn handle_network(
        &self,
        rx_slice: &[u8],
        ship_switch: &mut hashbrown::HashMap<[u8; 6], usize>,
        poll_fd_index: &usize,
        poll_fds_len: &usize,
        ship_traffic: &mut Vec<(usize, Vec<u8>)>,
        is_nmea: bool,
        prefix: String,
    ) {
        // Update the ship switch and add the packets to the ship traffic
        let eth_dst_addr: &[u8; 6] = &rx_slice[0..6].try_into().unwrap();
        let eth_src_addr: &[u8; 6] = &rx_slice[6..12].try_into().unwrap();

        // Add mac src address to the ship switch
        if !ship_switch.contains_key(eth_src_addr) {
            ship_switch.insert(*eth_src_addr, *poll_fd_index);
        }

        if let Some(out_sock_idx) = ship_switch.get(eth_dst_addr) {
            // if destination is known send directly to it
            if is_nmea {
                ship_traffic.push((*out_sock_idx, rx_slice.to_vec()));
            } else {
                ship_traffic.push((*out_sock_idx, rx_slice.to_vec()));
            }
        } else {
            for j in 0..*poll_fds_len {
                if *poll_fd_index == j {
                    continue;
                }
                // if destination is not known broadcast
                ship_traffic.push((j, rx_slice.to_vec()));
            }
        }
    }

    pub fn refill_umem_allocator(&mut self) {
        while self.sock.completion_ring.can_consume() {
            let offset = self
                .sock
                .completion_ring
                .get_nth_umem_offset(self.sock.completion_ring.get_consumer_index() as _);
            self.umem_allocator.release_offset(offset);
            self.sock.completion_ring.advance_consumer_index();
        }
    }

    pub fn refill_fill_rings(&mut self) {
        while let Some(chunk_index) = self.umem_allocator.try_allocate() {
            if self.sock.fill_ring.can_produce() {
                self.sock
                    .fill_ring
                    .produce_umem_offset(self.sock.umem.chunk_start_offset_for_index(chunk_index));
            } else {
                self.umem_allocator.release(chunk_index);
                break;
            }
        }
    }

    fn apply_policy(&self, message: String) -> (bool, bool, String) {
        let mut nmea = Nmea::new();
        let message_ok = nmea.parse(message.clone());

        match message_ok {
            Ok(()) => {
                // message is valid nmea
                // now gotta check if the message can be received by the component
                nmea.show();
                let prefix = format!("${}{}", nmea.str_talker_id(), nmea.str_sentence_type());
                let is_allowed = self
                    .sends
                    .iter()
                    .any(|allowed_message| prefix == *allowed_message);
                (is_allowed, true, prefix)
            }
            Err(_) => (false, false, String::from("NONMEA")),
        }
    }
}
