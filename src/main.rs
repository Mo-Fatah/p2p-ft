use clap::{Parser, Subcommand, Args};
mod server;
mod peer;
mod send;
mod receive;

#[derive(Debug, Parser)]
#[clap(name="file-transfer")]
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
    Receive(ReceiveCommand)
}

#[derive(Debug, Args)]
struct ServerCommand {
    ip: String,
    port: String
}

#[derive(Debug, Args)]
struct SendCommand {
    /// the public address of the Rendezvous server in the form of <IpAddress>:<Port>
    server_addr: String,
    data: String
}

#[derive(Debug, Args)]
struct ReceiveCommand {
    /// the public address of the Rendezvous server in the form of <IpAddress>:<Port>
    server_addr: String
}

fn main() -> anyhow::Result<()>{
    let cli: Cli = Parser::parse();
    match cli.command {
        CliCommand::Send(send_cmd) => {
            send::handle_send(send_cmd)?;
        },

        CliCommand::Receive(rcv_cmd) => {
            receive::handle_receive(rcv_cmd)?;
        },

        CliCommand::Server(server_cmd) => {
            server::handle_server(server_cmd)?;
        }
    }

    Ok(())
}
