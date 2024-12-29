use std::os::fd::AsRawFd;
use std::sync::Arc;
use xdrippi::{utils::interface_name_to_index, BPFRedirectManager, Umem, UmemAllocator, XDPSocket};

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

        unsafe {
            libc::poll(poll_fds.as_mut_ptr(), poll_fds.len() as _, -1);
        }

        loop {
            for (i, _) in poll_fds
                .iter()
                .enumerate()
                .filter(|(_, fd)| fd.revents & libc::POLLIN != 0)
            {
                println!("Received on socket {i}");

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
                }
            }
        }
    }
}

pub struct ShipComponent<'a> {
    ifname: String,
    ifindex: libc::c_uint,
    bpf_manager: BPFRedirectManager,
    sock: XDPSocket<'a>,
    umem_allocator: UmemAllocator,
    poll_fd: libc::pollfd,
}

impl ShipComponent<'_> {
    pub fn new(ifname: String) -> Self {
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
            ifname,
            ifindex,
            bpf_manager,
            sock,
            umem_allocator,
            poll_fd,
        }
    }
}

fn main() {
    // Setting up the ship components
    let girobussola = ShipComponent::new(String::from("test1"));
    let ais = ShipComponent::new(String::from("test2"));
    let gps = ShipComponent::new(String::from("test3"));
    let ecoscandaglio = ShipComponent::new(String::from("test4"));
    let velocita = ShipComponent::new(String::from("test5"));
    let radar = ShipComponent::new(String::from("test6"));
    let ecdis = ShipComponent::new(String::from("test7"));

    // Setting up ship
    let components: Vec<ShipComponent> =
        vec![girobussola, ais, gps, ecoscandaglio, velocita, radar, ecdis];
    let mut ship = Ship::new(components);
    ship.monitor_components();
}
