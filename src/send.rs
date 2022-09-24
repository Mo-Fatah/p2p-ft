use std::{
    fs::File,
    io::{Read, Write},
};

use crate::{peer, SendCommand};
use anyhow::{bail, Result};

pub(crate) fn handle_send(send_cmd: SendCommand) -> Result<()> {
    let mut tcp_stream = peer::handle_peers(&send_cmd.server_addr)?;

    let mut file: File;

    if let Ok(f) = File::open(&send_cmd.file) {
        file = f;
    } else {
        bail!("File not found.");
    }

    let file_name = send_cmd.file.file_name().unwrap().to_str().unwrap();
    let file_size = file.metadata().unwrap().len();

    let metadata = format!("name/{}/size/{}", file_name, file_size);

    println!("Sending Metadata ...");

    tcp_stream.write_all(metadata.as_bytes())?;

    let mut buf = [0u8; 1024];

    // Asserting that metadata are sent correctly
    loop {
        let size = tcp_stream.read(&mut buf)?;
        if size == 0 {
            continue;
        }

        let buf_str = String::from_utf8(buf[..size].to_vec())?;

        if buf_str != "ACK_META" {
            println!("Waiting for Metadata Ack ...");
            continue;
        }
        break;
    }

    println!("Sending {} ...", file_name);
    loop {
        let mut buf = [0u8; 4096];

        let size = file.read(&mut buf)?;

        if size == 0 {
            // reached to the end of the file
            break;
        }

        tcp_stream.write_all(&buf[..size])?;
    }

    tcp_stream.flush()?;

    println!("File sent Succefully");

    Ok(())
}
