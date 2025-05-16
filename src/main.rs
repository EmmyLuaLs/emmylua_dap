mod context;
mod handler;
mod logger;
mod cmd_args;

use std::io::{BufReader, BufWriter, Stdin, Stdout};

use context::EmmyLuaDebugContext;
use dap::{
    requests::{Command, Request},
    server::Server,
};
use handler::{on_initialize_request, on_launch_request, on_request_dispatch};
use logger::init_stderr_logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_stderr_logger(log::LevelFilter::Info);
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
