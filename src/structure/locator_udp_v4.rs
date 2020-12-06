use speedy::{Readable, Writable};
use std::net::{Ipv4Addr, SocketAddrV4};

/// Specialization of Locator_t used to hold UDP IPv4 locators using a more
/// compact representation. Equivalent to Locator_t with kind set to
/// LOCATOR_KIND_UDPv4. Need only be able to hold an IPv4 address and a port
/// number.
#[derive(Debug, PartialEq, Eq, Readable, Writable)]
pub struct LocatorUDPv4_t {
    /// The mapping between the dot-notation “a.b.c.d” of an IPv4 address and
    /// its representation as an unsigned long is as follows:
    /// address = (((a*256 + b)*256) + c)*256 + d
    address: u32,
    port: u32,
}

impl LocatorUDPv4_t {
    pub const LOCATORUDPv4_INVALID: LocatorUDPv4_t = LocatorUDPv4_t {
        address: 0,
        port: 0,
    };
}

impl From<SocketAddrV4> for LocatorUDPv4_t {
    fn from(socket_address: SocketAddrV4) -> Self {
        LocatorUDPv4_t {
            address: socket_address.ip().to_owned().into(),
            port: socket_address.port().into(),
        }
    }
}

impl From<LocatorUDPv4_t> for SocketAddrV4 {
    fn from(locator_udp_v4: LocatorUDPv4_t) -> Self {
        SocketAddrV4::new(
            Ipv4Addr::from(locator_udp_v4.address),
            locator_udp_v4.port as u16,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion_test() {
        let socket_addr: SocketAddrV4 = "127.0.0.1:8080".parse().unwrap();
        let locator_udp_v4 = LocatorUDPv4_t::from(socket_addr);

        assert_eq!(socket_addr, SocketAddrV4::from(locator_udp_v4));
    }

    serialization_test!(type = LocatorUDPv4_t,
        {
            locator_invalid,
            LocatorUDPv4_t::from("127.0.0.1:8080".parse::<SocketAddrV4>().unwrap()),
            le = [
                0x01, 0x00, 0x00, 0x7F,
                0x90, 0x1F, 0x00, 0x00
            ],
            be = [
                0x7F, 0x00, 0x00, 0x01,
                0x00, 0x00, 0x1F, 0x90
            ]
        }
    );
}
