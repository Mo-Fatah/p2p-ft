use std::{io::Write, thread::sleep, time::Duration};

use crate::{SendCommand, peer};
use anyhow::Result;

pub(crate) fn handle_send(send_cmd: SendCommand) -> Result<()>{
    let mut tcp_stream = peer::handle_peers(&send_cmd.server_addr)?; 
    loop {
        tcp_stream.write(b"Hi from the sender peer")?;
        sleep(Duration::from_secs(1));
    }
}
