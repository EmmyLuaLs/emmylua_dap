mod cmd_args;
mod context;
mod handler;
mod logger;

use std::io::{BufReader, BufWriter, Stdin, Stdout};

use cmd_args::CmdArgs;
use context::EmmyLuaDebugContext;
use dap::server::Server;
use handler::on_request_dispatch;
use logger::init_logger;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd_args = CmdArgs::from_args();

    init_logger(&cmd_args);
    let input = BufReader::new(std::io::stdin());
    let output = BufWriter::new(std::io::stdout());
    let server = Server::new(input, output);

    main_loop(server).await?;
    Ok(())
}

async fn main_loop(mut server: Server<Stdin, Stdout>) -> Result<(), Box<dyn std::error::Error>> {
    let output = server.output.clone();
    let mut context = EmmyLuaDebugContext::new(output);
    while let Some(request) = server.poll_request()? {
        on_request_dispatch(&mut context, request).await?;
    }

    Ok(())
}
