use std::io::Read;

use crate::{ReceiveCommand, peer};
use anyhow::Result;

pub(crate) fn handle_receive(send_cmd: ReceiveCommand) -> Result<()>{
    let mut tcp_stream = peer::handle_peers(&send_cmd.server_addr)?; 
    loop {
        let mut buf = [0; 1024];
        let size = tcp_stream.read(&mut buf)?;
        if size == 0 {
            continue;
        }
        let buf_str = String::from_utf8(buf[..size].to_vec())?;
        println!("{}", buf_str);
    }
}
