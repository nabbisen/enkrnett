//! The Wi-Fi networks' subnets and their renumbering on an uplink collision (ENK-TOPO-003,
//! ENK-TOPO-006). Both networks are /24. The management subnet comes from a range the common site
//! routers of the target markets do not use by default, so that it is the colliding one only by
//! exception; when it does collide, only it moves.

use std::net::Ipv4Addr;

/// Both Wi-Fi networks are this size.
pub const PREFIX: u8 = 24;

/// An IPv4 subnet in network form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Subnet {
    pub network: Ipv4Addr,
    pub prefix: u8,
}

impl Subnet {
    /// The subnet of `address` with `prefix` bits.
    pub fn of(address: Ipv4Addr, prefix: u8) -> Subnet {
        let prefix = prefix.min(32);
        let mask = if prefix == 0 {
            0
        } else {
            u32::MAX << (32 - prefix)
        };
        Subnet {
            network: Ipv4Addr::from(u32::from(address) & mask),
            prefix,
        }
    }

    /// `"10.0.2.15/24"` or an address with a dotted mask, as `netifd` reports them.
    pub fn parse(address: &str, mask: u8) -> Option<Subnet> {
        Some(Subnet::of(address.parse().ok()?, mask))
    }

    pub fn overlaps(&self, other: &Subnet) -> bool {
        let prefix = self.prefix.min(other.prefix);
        Subnet::of(self.network, prefix) == Subnet::of(other.network, prefix)
    }
}

/// The device's address on each Wi-Fi network; the subnet is the /24 around it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Addresses {
    pub management: Ipv4Addr,
    pub guest: Ipv4Addr,
}

impl Default for Addresses {
    fn default() -> Addresses {
        Addresses {
            management: Ipv4Addr::new(192, 168, 207, 1),
            guest: Ipv4Addr::new(10, 207, 0, 1),
        }
    }
}

impl Addresses {
    pub fn management_subnet(&self) -> Subnet {
        Subnet::of(self.management, PREFIX)
    }

    pub fn guest_subnet(&self) -> Subnet {
        Subnet::of(self.guest, PREFIX)
    }
}

/// Candidates for the management network, in order: 192.168.207 to 192.168.254, then 172.31.x.
fn management_candidates() -> impl Iterator<Item = Ipv4Addr> {
    (207u8..=254)
        .map(|c| Ipv4Addr::new(192, 168, c, 1))
        .chain((1u8..=254).map(|c| Ipv4Addr::new(172, 31, c, 1)))
}

/// Candidates for the guest network, in order: 10.207.0 to 10.207.254, then 172.16.x.
fn guest_candidates() -> impl Iterator<Item = Ipv4Addr> {
    (0u8..=254)
        .map(|c| Ipv4Addr::new(10, 207, c, 1))
        .chain((1u8..=254).map(|c| Ipv4Addr::new(172, 16, c, 1)))
}

fn free(address: Ipv4Addr, uplink: Option<&Subnet>, other: Option<&Subnet>) -> bool {
    let s = Subnet::of(address, PREFIX);
    !uplink.is_some_and(|u| s.overlaps(u)) && !other.is_some_and(|o| s.overlaps(o))
}

/// Keeps every address that does not collide with the uplink's subnet; moves a colliding one to
/// the first free candidate (ENK-TOPO-006: only the colliding network moves). The management
/// network is decided first, so that it moves only when it collides itself.
pub fn renumber(current: Addresses, uplink: Option<Subnet>) -> Addresses {
    let management = if free(current.management, uplink.as_ref(), None) {
        current.management
    } else {
        management_candidates()
            .find(|a| free(*a, uplink.as_ref(), None))
            .unwrap_or(current.management)
    };
    let mgmt_subnet = Subnet::of(management, PREFIX);
    let guest = if free(current.guest, uplink.as_ref(), Some(&mgmt_subnet)) {
        current.guest
    } else {
        guest_candidates()
            .find(|a| free(*a, uplink.as_ref(), Some(&mgmt_subnet)))
            .unwrap_or(current.guest)
    };
    Addresses { management, guest }
}

#[cfg(test)]
mod tests;
