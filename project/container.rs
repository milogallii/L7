use std::os::fd::AsRawFd;
use std::sync::Arc;
use xdrippi::{utils::interface_name_to_index, BPFRedirectManager, Umem, UmemAllocator, XDPSocket};

pub struct Container<'a> {
    ifname: String,
    ifindex: libc::c_uint,
    bpf_manager: BPFRedirectManager,
    sock: XDPSocket<'a>,
    umem_allocator: UmemAllocator,
}

impl Container<'_> {
    pub fn new(ifname: String) -> Self {
        // Getting interface index
        let ifindex = interface_name_to_index(ifname.as_str()).unwrap();

        // Setting up umem for xsk
        let umem = Umem::new_2k(16384).unwrap();
        let umem = Arc::new(umem);

        // Setting up xsk
        let sock = XDPSocket::new(ifindex, 0, umem.clone(), 4096).unwrap();
        let mut bpf_manager = BPFRedirectManager::attach(ifindex);
        bpf_manager.add_redirect(0, sock.as_raw_fd());

        // setting up the allocator for umem
        let umem_allocator = UmemAllocator::for_umem(umem.clone());

        Container {
            ifname,
            ifindex,
            bpf_manager,
            sock,
            umem_allocator,
        }
    }
}

fn main() {
    let girobussola = Container::new(String::from("test1"));
    let ais = Container::new(String::from("test2"));
    let gps = Container::new(String::from("test3"));
    let ecoscandaglio = Container::new(String::from("test4"));
    let velocita = Container::new(String::from("test5"));
    let radar = Container::new(String::from("test6"));
    let ecdis = Container::new(String::from("test7"));
}
