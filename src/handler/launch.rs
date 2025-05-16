use dap::{requests::LaunchRequestArguments, responses::ResponseBody};
use tokio_util::sync::CancellationToken;

use crate::{
    context::{DapSnapShot, EmmyNewDebugArguments},
    handler::RequestHandlerError,
};

use super::RequestResult;

pub async fn on_launch_request(
    dap: DapSnapShot,
    launch_arguments: LaunchRequestArguments,
    _: CancellationToken,
) -> RequestResult {
    log::info!("Received Launch request: {:?}", launch_arguments);
    // todo check mode
    let additional = match launch_arguments.additional_data {
        Some(additional) => additional,
        None => {
            return Err(
                RequestHandlerError::Message("No additional data provided".to_string()).into(),
            );
        }
    };

    let emmy_new_debug_argument = serde_json::from_value::<EmmyNewDebugArguments>(additional)
        .map_err(|_| RequestHandlerError::Message("Failed to parse additional data".to_string()))?;

    let debugger_conn = dap.debugger_conn.lock().await;
    // if emmy_new_debug_argument.ide_connect_debugger

    Ok(ResponseBody::Launch)
}
