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
use handler::{on_initialize_request, on_launch_request};
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
    let context = EmmyLuaDebugContext::new();
    while let Some(request) = server.poll_request()? {
        match request.command.clone() {
            Command::Initialize(initialize_argument) => {
                on_initialize_request(&context, initialize_argument, request).await?;
            }
            Command::Launch(launch_argument) => {
                on_launch_request(&context, launch_argument, request).await?;
            }
            // Command::Attach(args) => {
            //     server.send_response(request, format!("Attach with args: {:?}", args)).await?;
            // }
            // Command::Disconnect(_) => {
            //     server.send_response(request, "Disconnect response").await?;
            // }
            // Command::Terminate(_) => {
            //     server.send_response(request, "Terminate response").await?;
            // }
            // _ => {
            //     server.send_error_response(request, "Unknown command").await?;
            // }
            _ => {}
        };
    }

    Ok(())
}
