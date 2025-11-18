mod cmd_args;
mod context;
mod handler;
mod logger;

use std::io::{BufReader, BufWriter, Stdin, Stdout};

use clap::Parser;
use cmd_args::CmdArgs;
use context::EmmyLuaDebugContext;
use dap::server::Server;
use handler::on_request_dispatch;
use logger::init_logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd_args = CmdArgs::parse();

    init_logger(&cmd_args);
    let current_path = std::env::current_dir()?;
    log::info!("Starting path {:?}", current_path);

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
