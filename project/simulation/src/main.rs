use network_types::eth::EthHdr;
use network_types::ip::{Ipv4Hdr, Ipv6Hdr};
use network_types::tcp::TcpHdr;
use network_types::udp::UdpHdr;
use std::os::fd::AsRawFd;
use std::sync::Arc;
use xdrippi::{utils::interface_name_to_index, BPFRedirectManager, Umem, UmemAllocator, XDPSocket};

pub struct PacketParser<'a> {
    packet: &'a [u8],
}

impl<'a> PacketParser<'a> {
    pub fn new(packet: &'a [u8]) -> Self {
        PacketParser { packet }
    }

    pub fn parse(&self) {
        let eth_hdr: &EthHdr = unsafe { &*(self.packet.as_ptr() as *const EthHdr) };
        print!(
            "ETH [ SRC {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} ]",
            (eth_hdr).src_addr[0],
            (eth_hdr).src_addr[1],
            (eth_hdr).src_addr[2],
            (eth_hdr).src_addr[3],
            (eth_hdr).src_addr[4],
            (eth_hdr).src_addr[5]
        );
        println!(
            " [ DST {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} ]",
            (eth_hdr).dst_addr[0],
            (eth_hdr).dst_addr[1],
            (eth_hdr).dst_addr[2],
            (eth_hdr).dst_addr[3],
            (eth_hdr).dst_addr[4],
            (eth_hdr).dst_addr[5]
        );

        let ipv4_hdr: &Ipv4Hdr =
            unsafe { &*(self.packet[EthHdr::LEN..].as_ptr() as *const Ipv4Hdr) };
        println!(
            "IPV4 [ SRC {:?} ] [ DST {:?} ]",
            ipv4_hdr.src_addr(),
            ipv4_hdr.dst_addr()
        );
    }
}

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
                println!("[ INTERFACE {i} ] - [ {} ]", self.components[i].name);

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

                    let parser: PacketParser = PacketParser::new(rx_slice);
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

pub struct ShipComponent<'a> {
    name: String,
    ifname: String,
    ifindex: libc::c_uint,
    bpf_manager: BPFRedirectManager,
    sock: XDPSocket<'a>,
    umem_allocator: UmemAllocator,
    poll_fd: libc::pollfd,
}

impl ShipComponent<'_> {
    pub fn new(name: String, ifname: String) -> Self {
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
}

fn main() {
    // Setting up the ship components
    let c0 = ShipComponent::new(String::from("girobussola"), String::from("test1"));
    let c1 = ShipComponent::new(String::from("ais"), String::from("test2"));
    let c2 = ShipComponent::new(String::from("gps"), String::from("test3"));
    let c3 = ShipComponent::new(String::from("ecoscandaglio"), String::from("test4"));
    let c4 = ShipComponent::new(String::from("velocita"), String::from("test5"));
    let c5 = ShipComponent::new(String::from("radar"), String::from("test6"));
    let c6 = ShipComponent::new(String::from("ecdis"), String::from("test7"));

    // Setting up ship
    let components: Vec<ShipComponent> = vec![c0, c1, c2, c3, c4, c5, c6];
    let mut ship = Ship::new(components);
    ship.monitor_components();
}
