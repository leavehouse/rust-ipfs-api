use multiaddr::{Multiaddr, Protocol};

fn is_ip(p:Protocol) -> bool {
    match p {
        Protocol::IP4 | Protocol::IP6 => true,
        _ => false,
    }
}

// Comment from go-multiaddr-net:
//
// "IsThinWaist returns whether a Multiaddr starts with "Thin Waist" Protocols.
// This means: /{IP4, IP6}[/{TCP, UDP}]"
fn is_thin_waist(m: &Multiaddr) -> bool {
    let protocol = m.protocol();
    if protocol.len() == 0 {
        return false
    }
    let p1 = protocol[0];
    if !is_ip(p1) {
        return false
    }
    if protocol.len() == 1 {
        return true
    }
    let p2 = protocol[1];
    is_ip(p1) && (p2 == Protocol::TCP || p2 == Protocol::UDP || is_ip(p2))
}


#[cfg(test)]
mod tests {
    use multiaddr::{Multiaddr};

    #[test]
    fn thin_waist() {
        let test_maddrs = vec![
            "/ip4/127.0.0.1/udp/1234",
            "/ip4/127.0.0.1/tcp/1234",
            "/ip4/1.2.3.4",
            "/ip4/0.0.0.0",
            "/ip6/::1",
            "/ip6/2601:9:4f81:9700:803e:ca65:66e8:c21",
            "/ip6/2601:9:4f81:9700:803e:ca65:66e8:c21/udp/1234"
        ];

        for maddr_str in &test_maddrs {
            let maddr = match Multiaddr::new(maddr_str) {
                Err(e) => panic!("Error parsing multiaddr {}: {}", maddr_str, e),
                Ok(maddr) => maddr,
            };
            assert!(super::is_thin_waist(&maddr));
        }
    }
}
