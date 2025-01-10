use nmea::Nmea;
use packet_parser::PacketParser;
use std::os::fd::AsRawFd;
use std::{collections::HashMap, sync::Arc};
use xdrippi::{utils::interface_name_to_index, BPFRedirectManager, Umem, UmemAllocator, XDPSocket};

pub struct ShipComponent<'a> {
    pub name: String,
    pub ifname: String,
    pub ifindex: libc::c_uint,
    pub bpf_manager: BPFRedirectManager,
    pub sock: XDPSocket<'a>,
    pub umem_allocator: UmemAllocator,
    pub poll_fd: libc::pollfd,
    pub sends: Vec<String>,
    pub receives: Vec<String>,
}

impl ShipComponent<'_> {
    pub fn new(name: String, ifname: String, sends: Vec<String>, receives: Vec<String>) -> Self {
        // Getting interface index
        let ifindex = interface_name_to_index(ifname.as_str()).unwrap();

        // Setting up umem for xsk
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

        ShipComponent {
            name,
            ifname,
            ifindex,
            bpf_manager,
            sock,
            umem_allocator,
            poll_fd,
            sends,
            receives,
        }
    }

    pub fn consume_rx_ring(
        &mut self,
        poll_fd_index: usize,
        poll_fds_len: usize,
        ship_traffic: &mut Vec<(usize, Vec<u8>)>,
        ship_switch: &mut HashMap<[u8; 6], usize>,
    ) {
        println!(
            "[INTERFACE {} : {} ]---[{}]",
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

        match packet_parser.parse_traffic() {
            Ok(message) => {
                message_ok = self.apply_policy(message);
                if message_ok {
                    println!("|-- NMEA SENTENCE ALLOWED");
                }
            }
            Err(_) => {
                println!("|-- TRAFFIC NOT PARSED");
            }
        }

        if message_ok {
            // Update the ship switch and add the packets to the ship traffic
            let eth_dst_addr: &[u8; 6] = &rx_slice[0..6].try_into().unwrap();
            let eth_src_addr: &[u8; 6] = &rx_slice[6..12].try_into().unwrap();

            // Add mac src address to the ship switch
            if !ship_switch.contains_key(eth_src_addr) {
                ship_switch.insert(*eth_src_addr, poll_fd_index);
            }

            if let Some(out_sock_idx) = ship_switch.get(eth_dst_addr) {
                ship_traffic.push((*out_sock_idx, rx_slice.to_vec()));
            } else {
                for j in 0..poll_fds_len {
                    if poll_fd_index == j {
                        continue;
                    }
                    ship_traffic.push((j, rx_slice.to_vec()));
                }
            }
        } else {
            println!("|-- MESSAGE IS NOT A NMEA SENTENCE OR IS NOT ALLOWED ");
            println!("|-- REC ALLOWED {:?}", self.receives);
            println!("|-- SND ALLOWED {:?}", self.sends);
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

    fn apply_policy(&self, message: String) -> bool {
        // commented part is to check if the received message is a valid Nmea sentence
        // let mut nmea = Nmea::default();
        // match nmea.parse(message.as_str()) {
        //     Ok(_) => {
        // if valid apply the component's policy
        self.receives
            .iter()
            .any(|allowed_message| message.starts_with(allowed_message))

        //     }
        //     Err(_) => {
        //         println!("|-- MESSAGE IS NOT ALLOWED... REMOVED FROM TRAFFIC");
        //         false
        //     }
        // }
    }
}
