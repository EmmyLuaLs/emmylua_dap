use dap::{requests::InitializeArguments, responses::ResponseBody, types::Capabilities};
use tokio_util::sync::CancellationToken;

use crate::context::DapSnapShot;

use super::RequestResult;

pub async fn on_initialize_request(
    _: DapSnapShot,
    initialize_arguments: InitializeArguments,
    _: CancellationToken,
) -> RequestResult {
    log::info!("Received Initialize request: {:?}", initialize_arguments);

    Ok(ResponseBody::Initialize(Capabilities {
        supports_evaluate_for_hovers: Some(true),
        support_terminate_debuggee: Some(true),
        supports_log_points: Some(true),
        supports_conditional_breakpoints: Some(true),
        ..Default::default()
    }))
}
