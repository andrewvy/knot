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
use serializer;

fn _debugf<F: Future<Item = (), Error = ()>>(_: F) {}
fn _debugs<S: Stream<Item = (Bytes, SocketAddr), Error = ()>>(_: S) {}

pub struct Client {
    pub clientbound_tx: UnboundedSender<(BytesMut, SocketAddr)>,
    pub serverbound_tx: UnboundedSender<(BytesMut, SocketAddr)>,
    pub proxy_addr: SocketAddr,
    pub player_name: String,
}

impl Client {
    pub fn new(source_addr: SocketAddr, remote_addr: SocketAddr, clientbound_tx: UnboundedSender<(BytesMut, SocketAddr)>) -> Client {
        let random_addr = &"0.0.0.0:0".parse::<SocketAddr>().unwrap();
        let proxy_socket = UdpSocket::bind(&random_addr).unwrap();
        let proxy_addr = proxy_socket.local_addr().unwrap();

        let (proxy_sink, proxy_stream) = UdpFramed::new(proxy_socket, BytesCodec::new()).split();
        let (client_tx, client_rx) = unbounded::<(BytesMut, SocketAddr)>();

        info!("New client: {}, creating temporary socket {} -> {}", source_addr, proxy_addr, remote_addr);

        let server_to_client = clientbound_tx.clone().send_all(
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

        Client {
            serverbound_tx: client_tx.clone(),
            clientbound_tx: clientbound_tx.clone(),
            player_name: String::from(""),
            proxy_addr,
        }
    }

    pub fn set_player_name(&mut self, player_name: &String) {
        self.player_name = player_name.clone();
    }
}

pub fn start(config: &Config) {
    let local_addr = config.host.parse::<SocketAddr>().unwrap();

    info!("Starting proxy on {}", local_addr);

    let socket = UdpSocket::bind(&local_addr).unwrap();
    let (sink, stream) = UdpFramed::new(socket, BytesCodec::new()).split();

    let client_map : HashMap<SocketAddr, Client> = HashMap::new();
    let (main_tx, main_rx) = unbounded::<(BytesMut, SocketAddr)>();

    let remote_addr = config.servers["lobby"].address.parse::<SocketAddr>().unwrap();

    let acceptor = stream.map_err(|_| ()).map(|(msg, source_addr)| {
        match packet(&msg).to_full_result() {
            Ok(packet) => (msg, Some(packet), source_addr),
            _ => (msg, None, source_addr),
        }
    }).fold(client_map, move |mut hashmap, (msg, packet, source_addr)| {
        if !hashmap.contains_key(&source_addr) {
            let client = Client::new(source_addr.clone(), remote_addr.clone(), main_tx.clone());
            hashmap.insert(source_addr.clone(), client);

            Ok(hashmap)
        } else {
            {
                let client = hashmap.get_mut(&source_addr).unwrap();

                match packet {
                    Some(packet) => {
                        match packet.data_packet {
                            Some(DataPacket::TOSERVER_INIT(data_packet)) => {
                                info!("New player connected: {}", &data_packet.player_name);
                                client.set_player_name(&data_packet.player_name);
                                client.serverbound_tx.unbounded_send((msg, source_addr.clone())).unwrap();
                            }
                            Some(DataPacket::TOSERVER_CHAT_MESSAGE(ref data_packet)) => {
                                info!("[CHAT] <{}>: {}", client.player_name, &data_packet.message);
                                match serializer::serialize(&packet) {
                                    Ok(bytes) => {
                                        client.serverbound_tx.unbounded_send((BytesMut::from(bytes), source_addr.clone())).unwrap();
                                    }
                                    _ => client.serverbound_tx.unbounded_send((msg, source_addr.clone())).unwrap(),
                                }
                            }
                            _ => {
                                client.serverbound_tx.unbounded_send((msg, source_addr.clone())).unwrap();
                            },
                        }

                    }
                    None => client.serverbound_tx.unbounded_send((msg, source_addr.clone())).unwrap(),
                };
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
