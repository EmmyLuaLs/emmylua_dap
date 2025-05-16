mod initialize;
mod launch;

use std::error::Error;

use dap::{requests::{Command, Request}, responses::ResponseBody};
pub use initialize::on_initialize_request;
pub use launch::on_launch_request;

use crate::context::EmmyLuaDebugContext;

pub type RequestResult = Result<ResponseBody, Box<dyn Error + Send>>;

pub async fn on_request_dispatch(
    context: &mut EmmyLuaDebugContext,
    request: Request,
) -> Result<(), Box<dyn std::error::Error>> {
    match request.command.clone() {
        Command::Initialize(initialize_argument) => {
            context.task(request, initialize_argument, on_initialize_request).await;
        }
        Command::Launch(launch_argument) => {
            context.task(request, launch_argument, on_launch_request).await;
        }
        Command::Cancel(cancel_argument) => {
            if let Some(req_id) = cancel_argument.request_id {
                context.cancel(req_id).await;
            }
            
            return Ok(())
        }
        _ => {
            let response = request.error("Unsupported request");
            context.respond(response).await;
        }
    };

    Ok(())
}

#[derive(Debug)]
pub enum RequestHandlerError {
    Message(String),
}

impl std::fmt::Display for RequestHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestHandlerError::Message(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for RequestHandlerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl From<RequestHandlerError> for Box<dyn std::error::Error + Send> {
    fn from(e: RequestHandlerError) -> Self {
        Box::new(e)
    }
}