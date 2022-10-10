use anyhow::{bail, Result};
#[cfg(feature = "unix")]
use net2::unix::UnixTcpBuilderExt;
use net2::TcpBuilder;
use std::io::prelude::*;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub(crate) fn handle_peers(addr: &String) -> Result<TcpStream> {
    let connection_builder = TcpBuilder::new_v4()?;
    connection_builder.reuse_address(true).unwrap();

    #[cfg(feature = "unix")]
    connection_builder.reuse_port(true).unwrap();

    let mut stream = connection_builder.connect(addr)?;

    let formatted_msg = format!(
        "{}:{}",
        stream.local_addr()?.ip(),
        stream.local_addr()?.port()
    );

    println!("[ME -> S] publishing local endpoint {}", formatted_msg);
    stream.write(formatted_msg.as_bytes())?;

    loop {
        let mut buf = [0; 1024];
        let size = stream.read(&mut buf).unwrap();
        let buf = String::from_utf8(buf[..size].to_vec()).unwrap();
        println!("[S -> ME] {}", buf);

        if size == 0 {
            break;
        }

        let connection_established = Arc::new(Mutex::new(false));
        let connection_established_clone = Arc::clone(&connection_established);
        let cloned_stream = stream.try_clone().unwrap();

        // listen
        std::thread::spawn(move || {
            let listen_on = cloned_stream.local_addr().unwrap().to_string();
            println!(
                "[LISTENING] on the same port used to connect to S {}",
                listen_on
            );
            listen(listen_on).unwrap();
        });


        println!("[YOU -> PEER] Trying to connect to your peer");

        // PUBLIC
        let cloned_stream = stream.try_clone().unwrap();
        let buf_clone = buf.clone();

        let public_thread = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(200));

            let ips: Vec<&str> = buf_clone.split("|").collect();
            let connect_to = ips.get(0).unwrap();
            let laddr = cloned_stream.local_addr().unwrap().to_string();

            connect(&laddr, connect_to, connection_established, "public")
        });

        // PRIVATE
        let cloned_stream = stream.try_clone().unwrap();
        let private_thread = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(200));

            let ips: Vec<&str> = (&buf).split("|").collect();
            let connect_to = ips.get(1).unwrap();
            let laddr = cloned_stream.local_addr().unwrap().to_string();

            connect(&laddr, connect_to, connection_established_clone, "private")
        });

        if let Ok(tcp_stream) = public_thread.join().unwrap() {
            return Ok(tcp_stream);
        }

        if let Ok(tcp_stream) = private_thread.join().unwrap() {
            return Ok(tcp_stream);
        }
    }

    bail!("Couldn't connect to peer");
}

fn connect(
    laddr: &str,
    ip: &str,
    connection_established: Arc<Mutex<bool>>,
    flag: &'static str,
) -> Result<TcpStream> {
    let connection_builder = TcpBuilder::new_v4()?;
    connection_builder.reuse_address(true).unwrap();
    #[cfg(feature = "unix")]
    connection_builder.reuse_port(true).unwrap();

    connection_builder
        .bind(laddr)
        .expect(&format!("binding from : {}", flag))
        .reuse_address(true)
        .unwrap();
    
    #[cfg(feature = "unix")]
    connection_builder.reuse_port(true).unwrap();

    loop {
        let established = *connection_established.lock().unwrap();
        if established {
            //dbg!("Breaking {} loop cause the other one connected", flag);
            break Err(anyhow::Error::msg("Already connected"));
        }

        drop(established);

        //dbg!(
        //    "[ME -> B] Trying to connect to {} which is {} from {}",
        //    ip, flag, laddr
        //);
        let stream = connection_builder.connect(ip);

        if stream.is_err() {
            //dbg!("[ME -> B] Connection failed: repeating");
            continue;
        }
        println!("Connected to {} successfully!", ip);

        *connection_established.lock().unwrap() = true;
        let stream = stream.unwrap();
        break Ok(stream);
    }
}

fn listen(ip: String) -> std::io::Result<()> {
    let server_builder = TcpBuilder::new_v4()?;
    println!("Listening b: {}", ip);
    server_builder
        .reuse_address(true)
        .unwrap()
        .bind(ip)
        .unwrap()
        .reuse_address(true)
        .unwrap();

    #[cfg(feature = "unix")]
    server_builder.reuse_port(true).unwrap();

    let server = server_builder.listen(1)?;
    for stream in server.incoming() {
        let stream = stream.unwrap();

        println!(
            "[B -> ME] PEER: {:?} | LOCAL: {:?}",
            stream.peer_addr().unwrap(),
            stream.local_addr().unwrap()
        );
    }
    Ok(())
}
