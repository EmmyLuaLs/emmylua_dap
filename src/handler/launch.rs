use dap::requests::{LaunchRequestArguments, Request};

use crate::context::EmmyLuaDebugContext;

pub async fn on_launch_request(
    context: &EmmyLuaDebugContext,
    launch_arguments: LaunchRequestArguments,
    request: Request,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Received Launch request: {:?}", launch_arguments);
    Ok(())
}
