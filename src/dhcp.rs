use data::DhcpLease;
use embassy_net::{udp::UdpSocket, EthernetAddress, Ipv4Address, Stack};
use esp_wifi::wifi::{WifiApDevice, WifiDevice};
use smoltcp::{
    socket::udp::PacketMetadata,
    wire::{DhcpMessageType, DhcpPacket, DhcpRepr},
};

use crate::prelude::*;

pub const ROUTER_IP: Ipv4Address = Ipv4Address::new(192, 168, 0, 1);

pub struct DHCPServer {
    pub dhcp_socket: UdpSocket<'static>,
    pub leases: Vec<DhcpLease, 16>,
}

#[task]
pub async fn dhcp(ap_stack: &'static Stack<WifiDevice<'static, WifiApDevice>>) -> ! {
    let mut rx_meta1 = [PacketMetadata::EMPTY; 4];
    let mut rx_buffer1 = [0u8; 1536];
    let mut tx_meta1 = [PacketMetadata::EMPTY; 4];
    let mut tx_buffer1 = [0u8; 1536];

    // let mut socket = UdpSocket::new(&stack, &mut rx_buffer, &mut tx_buffer);

    info!("DHCP socket up");

    let mut dhcp_socket = UdpSocket::new(
        ap_stack,
        &mut rx_meta1,
        &mut rx_buffer1,
        &mut tx_meta1,
        &mut tx_buffer1,
    );

    dhcp_socket.bind(67).unwrap();

    let mut buf = [0; 1536];

    loop {
        let (n, ep) = dhcp_socket.recv_from(&mut buf).await.unwrap();

        let dhcp_packet = match DhcpPacket::new_checked(&buf[..n]) {
            Ok(raw_packet) => raw_packet,

            Err(e) => {
                warn!("{e:?}");
                continue;
            }
        };

        trace!("rxd from {}: {:?}", ep, dhcp_packet);

        let dhcp_repr = match DhcpRepr::parse(&dhcp_packet) {
            Ok(repr) => repr,
            Err(e) => {
                warn!("failed to parse DHCP packet: {:?}", e);
                continue;
            }
        };

        let hardware_addr = dhcp_repr.client_hardware_address;

        let client_id = dhcp_repr.client_identifier.unwrap();

        //sanity check
        if hardware_addr == EthernetAddress::from_bytes(&[0; 6]) {
            warn!("ignoring DHCP packet with zeroed client hardware address");
            continue;
        }

        // remove later
        if hardware_addr != EthernetAddress::from_bytes(&[0x5c, 0xe9, 0xfe, 0xac, 0xd5, 0xdf]) {
            error!("not my mac (haha)");
            continue;
        }

        match dhcp_repr.message_type {
            DhcpMessageType::Discover => {
                //make random from pool in future
                let provisioned_address = Ipv4Address::new(192, 168, 0, 100);

                let reply_repr = DhcpRepr {
                    message_type: DhcpMessageType::Offer,
                    transaction_id: dhcp_repr.transaction_id,
                    secs: dhcp_repr.secs,
                    client_hardware_address: dhcp_repr.client_hardware_address,
                    client_ip: Ipv4Address::UNSPECIFIED,
                    your_ip: provisioned_address,
                    server_ip: dhcp_repr.server_ip,
                    router: Some(ROUTER_IP),
                    subnet_mask: Some(Ipv4Address::new(255, 255, 255, 0)),
                    relay_agent_ip: Ipv4Address::UNSPECIFIED,
                    broadcast: false,
                    requested_ip: None,
                    client_identifier: None,
                    server_identifier: Some(ROUTER_IP),
                    parameter_request_list: None,
                    max_size: None,
                    lease_duration: Some(60 * 60 * 24),
                    renew_duration: Some(60 * 60 * 12),
                    rebind_duration: Some(60 * 60 * 21),
                    dns_servers: Some(
                        Vec::from_slice(&[
                            ROUTER_IP,
                            Ipv4Address::UNSPECIFIED,
                            Ipv4Address::UNSPECIFIED,
                        ])
                        .unwrap(),
                    ),
                    additional_options: Default::default(),
                };

                let mut reply_packet = DhcpPacket::new_unchecked(&mut buf);

                info!("{reply_repr:#?}");

                if let Err(e) = reply_repr.emit(&mut reply_packet) {
                    warn!("{e:?}");

                    continue;
                }

                let addr = if client_id
                    == EthernetAddress::from_bytes(&[0x5c, 0xe9, 0xfe, 0xac, 0xd5, 0xdf])
                {
                    provisioned_address
                } else {
                    Ipv4Address::new(255, 255, 255, 255)
                };

                if let Err(e) = dhcp_socket
                    .send_to(&buf[..reply_repr.buffer_len()], (addr, 68))
                    .await
                {
                    warn!("{e:?}");

                    continue;
                }

                error!("sent");
            }
            DhcpMessageType::Request => {
                // let transaction_id = rng.random();

                //make random from pool in future
                let provisioned_address = Ipv4Address::new(192, 168, 0, 100);

                let reply_repr = DhcpRepr {
                    message_type: DhcpMessageType::Ack,
                    transaction_id: dhcp_repr.transaction_id,
                    secs: dhcp_repr.secs,
                    client_hardware_address: dhcp_repr.client_hardware_address,
                    client_ip: Ipv4Address::UNSPECIFIED,
                    your_ip: provisioned_address,
                    server_ip: dhcp_repr.server_ip,
                    router: Some(ROUTER_IP),
                    subnet_mask: Some(Ipv4Address::new(255, 255, 255, 0)),
                    relay_agent_ip: Ipv4Address::UNSPECIFIED,
                    broadcast: false,
                    requested_ip: None,
                    client_identifier: None,
                    server_identifier: Some(ROUTER_IP),
                    parameter_request_list: None,
                    max_size: None,
                    lease_duration: Some(60 * 60 * 24),
                    renew_duration: Some(60 * 60 * 12),
                    rebind_duration: Some(60 * 60 * 21),
                    dns_servers: Some(
                        Vec::from_slice(&[
                            ROUTER_IP,
                            Ipv4Address::UNSPECIFIED,
                            Ipv4Address::UNSPECIFIED,
                        ])
                        .unwrap(),
                    ),
                    additional_options: Default::default(),
                };

                let mut reply_packet = DhcpPacket::new_unchecked(&mut buf);

                if let Err(e) = reply_repr.emit(&mut reply_packet) {
                    warn!("{e:?}");

                    continue;
                }

                let addr = if client_id
                    == EthernetAddress::from_bytes(&[0x5c, 0xe9, 0xfe, 0xac, 0xd5, 0xdf])
                {
                    provisioned_address
                } else {
                    Ipv4Address::new(255, 255, 255, 255)
                };

                if let Err(e) = dhcp_socket
                    .send_to(&buf[..reply_repr.buffer_len()], (addr, 68))
                    .await
                {
                    warn!("{e:?}");

                    continue;
                }
            }
            _ => error!(
                "can't handle DHCP message type {:?}",
                dhcp_repr.message_type
            ),
        }
    }
}
