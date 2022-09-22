use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
mod peer;
mod receive;
mod send;
mod server;

#[derive(Debug, Parser)]
#[clap(name = "file-transfer")]
struct Cli {
    #[clap(subcommand)]
    command: CliCommand,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    /// Used for the Rendezvous server (Must have a public IP address)
    #[clap(name = "server")]
    Server(ServerCommand),

    /// Used by the sender peer
    #[clap(name = "send")]
    Send(SendCommand),

    /// Used by the receiver peer
    #[clap(name = "rcv")]
    Receive(ReceiveCommand),
}

#[derive(Debug, Args)]
struct ServerCommand {
    /// The IP to which the server should listen. Default value is 0.0.0.0
    #[clap(short, long, value_parser)]
    ip: Option<String>,

    #[clap(short, long, value_parser)]
    /// The port to which the server should be binded. This value should be used by the client peers
    port: String,
}

#[derive(Debug, Args)]
struct SendCommand {
    /// the public address of the Rendezvous server in the form of <IpAddress>:<Port>
    server_addr: String,

    #[clap(value_parser)]
    file: PathBuf,
}

#[derive(Debug, Args)]
struct ReceiveCommand {
    /// the public address of the Rendezvous server in the form of <IpAddress>:<Port>
    server_addr: String,

//    /// the output path of the received file.
//    #[clap(short, long, value_parser)]
//    output: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli: Cli = Parser::parse();
    match cli.command {
        CliCommand::Send(send_cmd) => {
            send::handle_send(send_cmd)?;
        }

        CliCommand::Receive(rcv_cmd) => {
            receive::handle_receive(rcv_cmd)?;
        }

        CliCommand::Server(server_cmd) => {
            server::handle_server(server_cmd)?;
        }
    }

    Ok(())
}
