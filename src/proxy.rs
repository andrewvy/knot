use std::ops::DerefMut;
use std::net::SocketAddr;
use futures::sync::mpsc::{unbounded, UnboundedSender};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::io;

use tokio;
use tokio::prelude::*;
use tokio::net::{UdpSocket, UdpFramed};
use tokio_io::codec::BytesCodec;
use bytes::{Bytes, BytesMut};

use config::Config;

fn _debugf<F: Future<Item = (), Error = ()>>(_: F) {}
fn _debugs<S: Stream<Item = (Bytes, SocketAddr), Error = ()>>(_: S) {}

pub fn start(config: &Config) {
    let local_addr = config.host.parse::<SocketAddr>().unwrap();
    let remote_addr = "127.0.0.1:30000".parse::<SocketAddr>().unwrap();

    println!("Starting proxy on {}", local_addr);

    let socket = UdpSocket::bind(&local_addr).unwrap();
    let (sink, stream) = UdpFramed::new(socket, BytesCodec::new()).split();

    let client_map : Arc<Mutex<HashMap<SocketAddr, UnboundedSender<(BytesMut, SocketAddr)>>>> = Arc::new(Mutex::new(HashMap::new()));
    let (main_tx, main_rx) = unbounded::<(BytesMut, SocketAddr)>();

    let writer_client_map = client_map.clone();

    let acceptor = stream.map(move |(msg, source_addr)| {
        let mut lock = writer_client_map.lock().unwrap();
        let hashmap = lock.deref_mut();

        if !hashmap.contains_key(&source_addr) {
            let proxy_socket = UdpSocket::bind(&"0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
            let proxy_addr = proxy_socket.local_addr().unwrap();
            let (proxy_sink, proxy_stream) = UdpFramed::new(proxy_socket, BytesCodec::new()).split();
            let (client_tx, client_rx) = unbounded::<(BytesMut, SocketAddr)>();

            println!("New client: {:?}", source_addr);
            println!("Creating proxy to {:?}", remote_addr);
            println!("Creating temporary socket {:?}", proxy_addr);

            hashmap.insert(source_addr.clone(), client_tx.clone());

            let server_to_client = main_tx.clone().send_all(
                proxy_stream.map(move |(msg, temp_addr)| {
                    println!("Server to client packet {:?} -> {:?}", temp_addr, source_addr.clone());
                    (msg, source_addr.clone())
                }).map_err(|_| panic!())
            ).map_err(|_| ()).map(|_| ());

            let client_to_server = proxy_sink.send_all(
                client_rx.map(move |(msg, addr)| {
                    println!("Client to server packet sent {:?} -> {:?}", addr.clone(), proxy_addr);
                    (msg.freeze(), remote_addr.clone())
                }).map_err(|_| io::Error::new(io::ErrorKind::Other, "Test"))
            ).map_err(|_| ()).map(|_| ());

            tokio::spawn(server_to_client);
            tokio::spawn(client_to_server);

            Ok(())
        } else {
            let tx = hashmap.get(&source_addr).unwrap();
            tx.unbounded_send((msg, source_addr.clone())).unwrap();
            Ok(())
        }
    })
    .for_each(|_: Result<(), ()>| Ok(()));

    let main_rx = main_rx.map(|(msg, addr)| (msg.freeze(), addr)).map_err(|_| io::Error::new(io::ErrorKind::Other, "Test"));
    let downstream = sink.send_all(main_rx).map_err(|_| ()).map(|_| ());

    tokio::run(
        acceptor.select2(downstream).map(|_| ()).map_err(|_| ())
    );
}
