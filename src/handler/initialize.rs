use dap::{requests::{InitializeArguments, Request}, responses::ResponseBody, types::Capabilities};

use crate::context::EmmyLuaDebugContext;

pub async fn on_initialize_request(
    _: &EmmyLuaDebugContext,
    initialize_arguments: InitializeArguments,
    request: Request,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Received Initialize request: {:?}", initialize_arguments);

    let response = ResponseBody::Initialize(Capabilities {
        supports_evaluate_for_hovers: Some(true),
        support_terminate_debuggee: Some(true),
        supports_log_points: Some(true),
        supports_conditional_breakpoints: Some(true),
        ..Default::default()
    });
    request.success(response);

    Ok(())
}
