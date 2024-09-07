use embassy_net::{EthernetAddress, Ipv4Address};
use net::WifiCredentials;

use crate::prelude::*;

#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq)]
// #[repr(packed(4))]
pub struct DhcpLease {
    // #[serde(with = "Ipv4AddressRef")]
    pub ip: [u8; 4],
    #[serde(with = "EthernetAddressRef")]
    pub mac: EthernetAddress,
    #[serde(with = "EmbassyInstantRef")]
    pub expires: Instant,
}

impl DhcpLease {
    pub fn new(Ipv4Address(ip): Ipv4Address, mac: EthernetAddress, expires: Instant) -> Self {
        Self { ip, mac, expires }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SpotifyAccessToken {
    access_token: String<290>,
    #[serde(with = "EmbassyInstantRef")]
    expires: Instant,
    refresh_token: String<290>,
}

impl SpotifyAccessToken {
    pub fn new(access_token: String<290>, expires: Instant, refresh_token: String<290>) -> Self {
        Self {
            access_token,
            expires,
            refresh_token,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SpotifyData {
    pub access_token: SpotifyAccessToken,
    pub client_id: String<32>,
}

impl SpotifyData {
    pub fn new(access_token: SpotifyAccessToken, client_id: String<32>) -> Self {
        Self {
            access_token,
            client_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
// #[non_exhaustive]
pub struct TeenyData {
    #[serde(with = "EmbassyInstantRef")]
    pub time: Instant,
    pub ap_creds: WifiCredentials,
    pub sta_creds: WifiCredentials,
    pub dhcp: Vec<DhcpLease, 16>,
    pub spotify: Option<SpotifyData>,
}

impl Default for TeenyData {
    fn default() -> Self {
        Self {
            time: Instant::MIN,
            ap_creds: Default::default(),
            sta_creds: Default::default(),
            dhcp: Default::default(),
            spotify: Default::default(),
        }
    }
}

impl TeenyData {
    const VERSION: u8 = 0;
}

// impl TeenyData {
//     pub fn new() -> Self {
//         Self::default()
//     }

//     pub fn ap_creds(&self) -> &WifiCredentials {
//         &self.ap_creds
//     }

//     pub fn sta_creds(&self) -> &WifiCredentials {
//         &self.sta_creds
//     }

//     pub fn dhcp(&self) -> &[DhcpLease] {
//         &self.dhcp
//     }

//     pub fn set_ap_creds(&mut self, creds: WifiCredentials) {
//         self.ap_creds = creds;
//     }

//     pub fn set_sta_creds(&mut self, creds: WifiCredentials) {
//         self.sta_creds = creds;
//     }

//     pub fn add_dhcp_lease(&mut self, lease: DhcpLease) {
//         self.dhcp.push(lease);
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(remote = "EthernetAddress")]
struct EthernetAddressRef([u8; 6]);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(remote = "Ipv4Address")]
struct Ipv4AddressRef([u8; 4]);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(remote = "Instant")]
struct EmbassyInstantRef {
    #[serde(getter = "Instant::as_ticks")]
    ticks: u64,
}

impl From<EmbassyInstantRef> for Instant {
    fn from(value: EmbassyInstantRef) -> Self {
        Self::from_ticks(value.ticks)
    }
}
