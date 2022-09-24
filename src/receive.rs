use crate::{peer, ReceiveCommand};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::fmt;
use std::{
    fs::File,
    io::{Read, Write},
};

pub(crate) fn handle_receive(rcv_cmd: ReceiveCommand) -> Result<()> {
    let mut tcp_stream = peer::handle_peers(&rcv_cmd.server_addr)?;
    let file_name: String;
    let file_size: u64;

    println!("Waiting For Metadata ...");

    loop {
        let mut buf = [0u8; 1024];
        let size = tcp_stream.read(&mut buf)?;

        if size == 0 {
            continue;
        }

        let metadata = String::from_utf8(buf[..size].to_vec())?;
        let meta_vec: Vec<&str> = metadata.split("/").collect();

        assert_eq!(meta_vec[0], "name");
        file_name = meta_vec[1].to_string();
        assert_eq!(meta_vec[2], "size");
        file_size = meta_vec[3].parse::<u64>().unwrap();

        tcp_stream.write(b"ACK_META")?;

        break;
    }

    let pb = ProgressBar::new(file_size);
    pb.set_style(ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})"
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));

    let mut file = File::create(file_name)?;

    let mut received: u64 = 0;

    loop {
        let mut buf = [0u8; 4096];
        let size = tcp_stream.read(&mut buf)?;
        if size == 0 {
            continue;
        }

        file.write(&buf[..size])?;
        received += size as u64;
        pb.set_position(received);

        if received == file_size {
            println!("Done");
            break;
        }
    }


    Ok(())
}
