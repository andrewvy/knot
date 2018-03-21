use std::net::SocketAddr;
use futures::sync::mpsc::{unbounded, UnboundedSender};
use std::collections::HashMap;
use std::io;

use tokio;
use tokio::prelude::*;
use tokio::net::{UdpSocket, UdpFramed};
use tokio_io::codec::BytesCodec;
use bytes::{Bytes, BytesMut};

use config::Config;
use packet::{DataPacket, packet};

fn _debugf<F: Future<Item = (), Error = ()>>(_: F) {}
fn _debugs<S: Stream<Item = (Bytes, SocketAddr), Error = ()>>(_: S) {}

pub fn start(config: &Config) {
    let local_addr = config.host.parse::<SocketAddr>().unwrap();

    info!("Starting proxy on {}", local_addr);

    let socket = UdpSocket::bind(&local_addr).unwrap();
    let (sink, stream) = UdpFramed::new(socket, BytesCodec::new()).split();

    let client_map : HashMap<SocketAddr, UnboundedSender<(BytesMut, SocketAddr)>> = HashMap::new();
    let (main_tx, main_rx) = unbounded::<(BytesMut, SocketAddr)>();

    let remote_addr = config.servers["lobby"].address.parse::<SocketAddr>().unwrap();

    let acceptor = stream.map_err(|_| ()).fold(client_map, move |mut hashmap, (msg, source_addr)| {
        {
            match packet(&msg).to_full_result() {
                Ok(packet) => {
                    match packet.data_packet {
                        Some(DataPacket::TOSERVER_INIT { player_name, ..}) => {
                            info!("New player connected: {}", player_name);
                        }
                        Some(DataPacket::TOSERVER_CHAT_MESSAGE { message }) => {
                            info!("Peer {} said: {}", packet.sender_peer_id, message);
                        }
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        if !hashmap.contains_key(&source_addr) {
            let proxy_socket = UdpSocket::bind(&"0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
            let proxy_addr = proxy_socket.local_addr().unwrap();
            let (proxy_sink, proxy_stream) = UdpFramed::new(proxy_socket, BytesCodec::new()).split();
            let (client_tx, client_rx) = unbounded::<(BytesMut, SocketAddr)>();

            info!("New client: {}, creating temporary socket {} -> {}", source_addr, proxy_addr, remote_addr);

            hashmap.insert(source_addr.clone(), client_tx.clone());

            let server_to_client = main_tx.clone().send_all(
                proxy_stream.map(move |(msg, _temp_addr)| {
                    (msg, source_addr.clone())
                }).map_err(|_| panic!())
            ).map_err(|_| ()).map(|_| ());

            let client_to_server = proxy_sink.send_all(
                client_rx.map(move |(msg, _addr)| {
                    (msg.freeze(), remote_addr.clone())
                }).map_err(|_| io::Error::new(io::ErrorKind::Other, "Test"))
            ).map_err(|_| ()).map(|_| ());

            tokio::spawn(server_to_client);
            tokio::spawn(client_to_server);

            Ok(hashmap)
        } else {
            {
                let tx = hashmap.get(&source_addr).unwrap();
                tx.unbounded_send((msg, source_addr.clone())).unwrap();
            }

            Ok(hashmap)
        }
    })
    .map(|_| ());

    let main_rx = main_rx.map(|(msg, addr)| (msg.freeze(), addr)).map_err(|_| io::Error::new(io::ErrorKind::Other, "Test"));
    let downstream = sink.send_all(main_rx).map_err(|_| ()).map(|_| ());

    tokio::run(
        acceptor.select2(downstream).map(|_| ()).map_err(|_| ())
    );
}
