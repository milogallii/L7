use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;

pub struct PacketParser<'a> {
    packet: &'a [u8],
}

impl<'a> PacketParser<'a> {
    pub fn new(packet: &'a [u8]) -> Self {
        PacketParser { packet }
    }

    pub fn parse_traffic(&self) {
       if let Some(eth_packet) = EthernetPacket::new(self.packet) {
            let eth_source = eth_packet.get_source();
            let eth_destination = eth_packet.get_destination();
            println!("ETH [SRC: {:?}] [DST: {:?}]", eth_source, eth_destination);
            match eth_packet.get_ethertype() {
                EtherTypes::Ipv4 =>
                {
                    if let Some(ipv4packet) = Ipv4Packet::new(eth_packet.payload()) {
                        self.parse_protocol_ipv4(ipv4packet);
                    }else{
                        println!("ERROR PARSING IPV4 PACKET");
                    }
                } 
                _ => println!("PROTOCOL NOT YET SUPPORTED [IPV6/LOOP/ARP/FIBRECHANNEL/INFINIBAND/LOOPBACKIEEE8023]"),
            }
        }
    }

    fn parse_protocol_ipv4(&self, ipv4_packet: Ipv4Packet) {
        match ipv4_packet.get_next_level_protocol() {
            IpNextHeaderProtocols::Udp => {
                if let Some(udp_packet) = UdpPacket::new(ipv4_packet.payload()) {
                    println!("UDP [SRC : {:?}] [DST : {:?}]", udp_packet.get_source(), udp_packet.get_destination());
                    println!("PAYLOAD -----------------");
                    let payload = udp_packet.payload();
                    if let Ok(payload_str) = std::str::from_utf8(payload) {
                        println!("{}", payload_str);
                    } else {
                        println!("UDP Payload is not valid UTF-8");
                    }       
                    println!("-----------------");
                    
                }
            }

            _ => {
                println!("NOT A UDP PACKET");
            }
        }
    }
}
