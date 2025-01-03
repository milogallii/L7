use network_types::eth::{EthHdr, EtherType};
use network_types::ip::IpProto::Udp;
use network_types::ip::Ipv4Hdr;

pub struct PacketParser<'a> {
    packet: &'a [u8],
}

impl<'a> PacketParser<'a> {
    pub fn new(packet: &'a [u8]) -> Self {
        PacketParser { packet }
    }

    pub fn parse_traffic(&self) {
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
