use std::os::fd::AsRawFd;
use std::sync::Arc;
use xdrippi::{utils::interface_name_to_index, BPFRedirectManager, Umem, UmemAllocator, XDPSocket};

pub struct ShipComponent<'a> {
    pub name: String,
    pub ifname: String,
    pub ifindex: libc::c_uint,
    pub bpf_manager: BPFRedirectManager,
    pub sock: XDPSocket<'a>,
    pub umem_allocator: UmemAllocator,
    pub poll_fd: libc::pollfd,
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
