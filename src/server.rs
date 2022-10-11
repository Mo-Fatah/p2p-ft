use anyhow::{self, bail};
use std::collections::HashMap;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

use crate::ServerCommand;

#[derive(Debug, Clone)]
struct Peer {
    pub local_address: String,
    pub local_port: u16,
    pub remote_address: String,
    pub remote_port: u16,
    pub secret: String
}

fn handle_client(
    mut socket: TcpStream,
    peers: Arc<Mutex<Vec<Peer>>>,
    hosts_tx: Sender<(std::string::String, std::string::String)>,
) -> anyhow::Result<()> {
    let stringified_address = socket.peer_addr().unwrap().ip().to_string();
    let socket_port = socket.peer_addr().unwrap().port();
    loop {
        let mut buf = [0; 1024];
        let size = socket.read(&mut buf);

        if buf.len() == 0 || size.is_err() {
            let mut lock = peers.lock().unwrap();
            let mut iter = lock.iter();

            let i = iter.position(|x| {
                x.remote_address == stringified_address && x.remote_port == socket_port
            });

            if let Some(index) = i {
                lock.remove(index);
                // println!("Removed: {:?}", lock);
            }

            drop(lock);
            return Ok(());
        }

        let buf = String::from_utf8(buf[..size.unwrap()].to_vec()).unwrap();

        println!("[INCOMING] from {} => {}", socket.peer_addr().unwrap(), buf);

        let mut local_elements = buf.split(":");
        // since it's a POC - it needs to be set and done so let us assume
        // that the message looks like xxx.xxx.xxx.xxx:ppppp
        let local_address = local_elements.next().unwrap();
        let local_port = local_elements.next().unwrap();
        let secret = local_elements.next().unwrap_or("");


        let peer = Peer {
            local_address: local_address.to_string(),
            local_port: local_port.parse::<u16>().unwrap(),

            remote_address: socket.peer_addr().unwrap().ip().to_string(),
            remote_port: socket
                .peer_addr()
                .unwrap()
                .port()
                .to_string()
                .parse::<u16>()
                .unwrap(),
            secret: secret.to_string()
        };

        //println!("{:?}", peer);

        let mut lock = peers.lock().unwrap();
        lock.push(peer);

        for p in lock.iter() {
            let filtered = filter_peers(&lock, String::from(&p.remote_address), p.remote_port);

            if filtered.len() > 0 {
                let sent = hosts_tx.send((
                    format!("{}:{}", p.remote_address, p.remote_port),
                    encode_peers(&filtered),
                ));
                if let Err(e) = sent {
                    println!("Error sending payload to channel {}", e);
                }
            }
        }

        drop(lock);
    }
}

pub(crate) fn handle_server(server_cmd: ServerCommand) -> anyhow::Result<()> {
    // default ip
    let mut ip = String::from("0.0.0.0");

    if let Some(given_ip) = server_cmd.ip {
        ip = given_ip;
    }

    let address = format!("{}:{}", ip, server_cmd.port);

    let listener = match TcpListener::bind(address) {
        Ok(listener) => listener,
        Err(err) => {
            bail!("couldn't bind to the given address.\n{}\n", err);
        }
    };

    let peers: Arc<Mutex<Vec<Peer>>> = Arc::new(Mutex::new(Vec::<Peer>::new()));
    let connections: Arc<Mutex<HashMap<String, TcpStream>>> =
        Arc::new(Mutex::new(HashMap::<String, TcpStream>::new()));
    let (hosts_tx, hosts_rx) = channel::<(String, String)>();

    let cloned_connections = Arc::clone(&connections);

    // This is the loop which is listening for incoming messages
    // from the channel
    // the idea behind this channel is to send payloads to the desired
    // socket connections
    std::thread::spawn(move || {
        loop {
            let recv = hosts_rx.recv();

            if recv.is_err() {
                println!("Recv error !");
                break;
            }
            
            // Get the desired socket via the key
            let (key, payload) = recv.unwrap();
            let mut lock = cloned_connections.lock().unwrap();
            let target = lock.get_mut(&key);
             
            if let Some(target) = target {

                let written = target.write(payload.as_bytes());
                if let Err(e) = written {
                    println!("Error sending payload to {}: {}", key, e);
                }
            }
            drop(lock);
        }
    });

    for stream in listener.incoming() {
        let peers_arc = Arc::clone(&peers);
        let stream = stream.unwrap();

        let stringified_address = stream.peer_addr().unwrap().ip().to_string();
        let stream_port = stream.peer_addr().unwrap().port();

        let key = format!("{}:{}", stringified_address, stream_port);

        let mut lock = connections.lock().unwrap();
        lock.insert(key, stream.try_clone().unwrap());
        drop(lock);

        let hosts_tx_clone = hosts_tx.clone();
        std::thread::spawn(move || {
            let handled = handle_client(stream, peers_arc, hosts_tx_clone);
            if let Err(e) = handled {
                println!("Crashed ! {}", e);
            }
        });
    }

    Ok(())
}

// filter is remote address
fn filter_peers(peers: &Vec<Peer>, filter_ip: String, filter_port: u16) -> Vec<Peer> {
    let mut result: Vec<Peer> = vec![];

    for i in peers {
        if i.remote_address == filter_ip && i.remote_port == filter_port {
            continue;
        }

        result.push(i.clone());
    }

    result
}

fn encode_peers(peers: &Vec<Peer>) -> String {
    let mut keys: Vec<String> = vec![];
    // let result = String::from("");

    for p in peers {
        keys.push(format!(
            "{}:{}|{}:{}",
            p.remote_address, p.remote_port, p.local_address, p.local_port
        ));
    }

    keys.join(",")
}
