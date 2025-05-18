use dap::{requests::LaunchRequestArguments, responses::ResponseBody};
use tokio_util::sync::CancellationToken;

use crate::{
    context::{DapSnapShot, EmmyNewDebugArguments},
    handler::{
        RequestHandlerError, debugger_connected::on_debugger_connected,
        debugger_notification::register_debugger_notification,
    },
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

    let mut debugger_conn = dap.debugger_conn.lock().await;
    let address = format!(
        "{}:{}",
        emmy_new_debug_argument.host, emmy_new_debug_argument.port
    );
    if emmy_new_debug_argument.ide_connect_debugger {
        debugger_conn
            .connect(&address, Some(5))
            .await
            .map_err(|e| {
                RequestHandlerError::Message(format!("Failed to connect to debugger: {}", e))
            })?;
    } else {
        debugger_conn.listen(&address).await.map_err(|e| {
            RequestHandlerError::Message(format!("Failed to listen on debugger: {}", e))
        })?;
    }
    debugger_conn.start_reader_task(dap.ide_conn.clone());

    let mut data = dap.data.lock().await;
    data.extension = emmy_new_debug_argument.ext.clone();

    register_debugger_notification(dap.clone()).await;
    on_debugger_connected(dap.clone(), emmy_new_debug_argument.ext).await?;

    Ok(ResponseBody::Launch)
}
