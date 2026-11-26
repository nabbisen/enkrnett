//! ENK-TOPO-003, ENK-TOPO-006.

use super::*;

fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}

#[test]
fn subnets_mask_and_overlap() {
    let a = Subnet::of(ip("10.0.2.15"), 24);
    assert_eq!(a.network, ip("10.0.2.0"));
    assert!(a.overlaps(&Subnet::of(ip("10.0.2.200"), 24)));
    assert!(!a.overlaps(&Subnet::of(ip("10.0.3.1"), 24)));
    // A large uplink covers a small Wi-Fi subnet, and the other way round.
    assert!(Subnet::of(ip("10.5.5.5"), 8).overlaps(&Subnet::of(ip("10.207.0.1"), 24)));
    assert!(Subnet::of(ip("10.207.0.1"), 24).overlaps(&Subnet::of(ip("10.5.5.5"), 8)));
    assert_eq!(
        Subnet::parse("192.168.1.20", 24).unwrap().network,
        ip("192.168.1.0")
    );
    assert!(Subnet::parse("not an address", 24).is_none());
}

#[test]
fn no_uplink_or_no_collision_keeps_the_addresses() {
    let d = Addresses::default();
    assert_eq!(renumber(d, None), d);
    assert_eq!(renumber(d, Subnet::parse("192.168.1.20", 24)), d);
    assert_eq!(renumber(d, Subnet::parse("10.0.2.15", 24)), d);
}

#[test]
fn a_colliding_guest_subnet_moves_and_the_management_address_stays() {
    // ENK-TOPO-006: the uplink is 10.207.0.0/24, as a site router handing out that range would.
    let d = Addresses::default();
    let r = renumber(d, Subnet::parse("10.207.0.15", 24));
    assert_eq!(r.management, d.management);
    assert_eq!(r.guest, ip("10.207.1.1"));
    // A /16 uplink across the whole 10.207 range: the guest network leaves the range.
    let r = renumber(d, Subnet::parse("10.207.9.9", 16));
    assert_eq!(r.guest, ip("172.16.1.1"));
    assert_eq!(r.management, d.management);
}

#[test]
fn a_colliding_management_subnet_moves_to_the_next_one_and_the_guest_stays() {
    let d = Addresses::default();
    let r = renumber(d, Subnet::parse("192.168.207.15", 24));
    assert_eq!(r.management, ip("192.168.208.1"));
    assert_eq!(r.guest, d.guest);
    // 192.168.0.0/16 taken entirely: the management network leaves the range.
    let r = renumber(d, Subnet::parse("192.168.3.1", 16));
    assert_eq!(r.management, ip("172.31.1.1"));
}

#[test]
fn a_renumbered_network_stays_where_it_is_once_the_collision_is_gone() {
    // ENK-TOPO-003 says "choose again when the uplink's subnet changes"; a network that no
    // longer collides is not moved back, so that addresses do not flap.
    let moved = Addresses {
        management: ip("192.168.207.1"),
        guest: ip("10.207.1.1"),
    };
    assert_eq!(renumber(moved, Subnet::parse("10.0.2.15", 24)), moved);
}

#[test]
fn the_two_networks_never_share_a_subnet() {
    let odd = Addresses {
        management: ip("10.207.0.1"),
        guest: ip("10.207.0.1"),
    };
    let r = renumber(odd, None);
    assert_ne!(r.management_subnet(), r.guest_subnet());
}
