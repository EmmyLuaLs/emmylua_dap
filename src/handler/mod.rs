mod debug_action_request;
mod debugger_connected;
mod debugger_notification;
mod evaluate_request;
mod initialize_request;
mod launch_request;
mod scopes_request;
mod set_breakpoint_request;
mod stack_trace_request;
mod threads_request;
mod variables_request;

use std::error::Error;

use dap::{
    errors::ServerError,
    requests::{Command, Request},
    responses::ResponseBody,
};
use evaluate_request::on_evaluate_request;
pub use initialize_request::on_initialize_request;
pub use launch_request::on_launch_request;
use scopes_request::on_scopes_request;
use set_breakpoint_request::on_set_breakpoints_request;
use stack_trace_request::on_stack_trace_request;
use threads_request::on_threads_request;
use variables_request::on_variable_request;

use crate::context::EmmyLuaDebugContext;

pub type RequestResult = Result<ResponseBody, Box<dyn Error + Send>>;

pub async fn on_request_dispatch(
    context: &mut EmmyLuaDebugContext,
    request: Request,
) -> Result<(), Box<dyn std::error::Error>> {
    match request.command.clone() {
        Command::Initialize(initialize_argument) => {
            context
                .task(request, initialize_argument, on_initialize_request)
                .await;
        }
        Command::Launch(launch_argument) => {
            context
                .task(request, launch_argument, on_launch_request)
                .await;
        }
        Command::Threads => {
            context.task(request, (), on_threads_request).await;
        }
        Command::StackTrace(stack_trace_argument) => {
            context
                .task(request, stack_trace_argument, on_stack_trace_request)
                .await;
        }
        Command::Scopes(scopes_argument) => {
            context
                .task(request, scopes_argument, on_scopes_request)
                .await;
        }
        Command::Variables(variables_argument) => {
            context
                .task(request, variables_argument, on_variable_request)
                .await;
        }
        Command::Evaluate(evaluate_argument) => {
            context
                .task(request, evaluate_argument, on_evaluate_request)
                .await;
        }
        Command::Pause(_) => {
            context
                .task(request, (), debug_action_request::on_pause_request)
                .await;
        }
        Command::Continue(_) => {
            context
                .task(request, (), debug_action_request::on_continue_request)
                .await;
        }
        Command::StepIn(_) => {
            context
                .task(request, (), debug_action_request::on_step_in_request)
                .await;
        }

        Command::StepOut(_) => {
            context
                .task(request, (), debug_action_request::on_step_out_request)
                .await;
        }
        Command::Next(_) => {
            context
                .task(request, (), debug_action_request::on_next_request)
                .await;
        }
        Command::SetBreakpoints(set_breakpoint_argument) => {
            context
                .task(request, set_breakpoint_argument, on_set_breakpoints_request)
                .await;
        }
        Command::Cancel(cancel_argument) => {
            if let Some(req_id) = cancel_argument.request_id {
                context.cancel(req_id).await;
            }

            return Ok(());
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
    ServerError(ServerError),
}

impl std::fmt::Display for RequestHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestHandlerError::Message(msg) => write!(f, "{}", msg),
            RequestHandlerError::ServerError(err) => write!(f, "{}", err),
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
