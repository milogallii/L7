use network_types::eth::{EthHdr, EtherType};
use network_types::ip::IpProto::Udp;
use network_types::ip::Ipv4Hdr;
use network_types::tcp::TcpHdr;
use network_types::udp::UdpHdr;
use std::collections::HashMap;

pub struct ShipPacketParser<'a> {
    packet: &'a [u8],
    switch: HashMap<[u8; 6], usize>,
}

impl<'a> ShipPacketParser<'a> {
    pub fn new(packet: &'a [u8]) -> Self {
        ShipPacketParser {
            packet,
            switch: HashMap::new(),
        }
    }

    pub fn parse(&self) {
        self.parse_ethhdr();
    }

    pub fn check_switch(&mut self, poll_fd: usize) {
        let dst: &[u8; 6] = &self.packet[0..6].try_into().unwrap();
        let src: &[u8; 6] = &self.packet[6..12].try_into().unwrap();
        if !self.switch.contains_key(src) {
            self.switch.insert(*src, poll_fd);
        }
    }

    fn parse_ethhdr(&self) {
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

        self.parse_ether_type(eth_hdr);
    }

    fn parse_ether_type(&self, eth_hdr: &EthHdr) {
        match eth_hdr.ether_type {
            EtherType::Ipv4 => {
                let ipv4_hdr: &Ipv4Hdr =
                    unsafe { &*(self.packet[EthHdr::LEN..].as_ptr() as *const Ipv4Hdr) };
                println!(
                    "IPV4 [ SRC {:?} ] [ DST {:?} ]",
                    ipv4_hdr.src_addr(),
                    ipv4_hdr.dst_addr()
                );

                self.parse_protocol_ipv4(ipv4_hdr);
            }
            _ => println!(
                "PROTOCOL NOT YET SUPPORTED [IPV6/LOOP/ARP/FIBRECHANNEL/INFINIBAND/LOOPBACKIEEE8023]"
            )
        }
    }

    fn parse_protocol_ipv4(&self, ipv4_hdr: &Ipv4Hdr) {
        match ipv4_hdr.proto {
            Udp => {
                println!("RECEIVED A UDP PACKET");
            }

            _ => {
                println!("NOT UDP");
            }
        }
    }
}
