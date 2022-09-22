use std::{
    fs::File,
    io::{Read, Write},
};

use crate::{peer, ReceiveCommand};
use anyhow::Result;

pub(crate) fn handle_receive(rcv_cmd: ReceiveCommand) -> Result<()> {
    let mut tcp_stream = peer::handle_peers(&rcv_cmd.server_addr)?;
    let file_name: String;
    let _file_size: u64;

    println!("Waiting For Metadata ...");

    loop {
        let mut buf = [0u8; 1024];
        let size = tcp_stream.read(&mut buf)?;

        if size == 0 {
            continue;
        }

        let metadata= String::from_utf8(buf[..size].to_vec())?;
        let meta_vec : Vec<&str> = metadata.split("/").collect();

        assert_eq!(meta_vec[0], "name");
        file_name = meta_vec[1].to_string();
        assert_eq!(meta_vec[2], "size");
        _file_size = meta_vec[3].parse::<u64>().unwrap();

        tcp_stream.write(b"ACK_META")?;

        break;
        
    }

    let mut file = File::create(file_name)?;

    loop {
        let mut buf = [0u8; 4096];
        let size = tcp_stream.read(&mut buf)?;
        if size == 0 {
            continue;
        }
        println!("Received {} Bytes", size);
        file.write(&buf[..size])?;
    }
}
