use dap::{requests::LaunchRequestArguments, responses::ResponseBody};
use tokio_util::sync::CancellationToken;

use crate::{
    context::{DapSnapShot, EmmyNewDebugArguments},
    handler::{
        RequestHandlerError, debugger_connected::after_debugger_connected,
        debugger_notification::register_debugger_notification,
    },
};

use super::RequestResult;

pub async fn on_launch_request(
    dap: DapSnapShot,
    launch_arguments: LaunchRequestArguments,
    _: CancellationToken,
) -> RequestResult {
    log::info!("Received Launch request: {:#?}", launch_arguments);
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

    let mut host = emmy_new_debug_argument.host;
    if host == "localhost" {
        host = "[::1]".into();
    }

    let address = format!("{}:{}", host, emmy_new_debug_argument.port);
    if emmy_new_debug_argument.ide_connect_debugger {
        log::info!("Debugger connected to {}", address);
        debugger_conn
            .connect(&address, Some(5))
            .await
            .map_err(|e| {
                RequestHandlerError::Message(format!("Failed to connect to debugger: {}", e))
            })?;
    } else {
        log::info!("Debugger listening on {}", address);
        debugger_conn.listen(&address).await.map_err(|e| {
            RequestHandlerError::Message(format!("Failed to listen on debugger: {}", e))
        })?;
    }

    log::info!("Debugger connection established, starting reader task");
    debugger_conn.start_reader_task(dap.ide_conn.clone());

    let mut data = dap.data.lock().await;
    data.extension = emmy_new_debug_argument.ext.clone();

    let dap = dap.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        log::info!("Registering debugger notification");
        register_debugger_notification(dap.clone()).await;

        log::info!("after debugger connected");
        match after_debugger_connected(dap, emmy_new_debug_argument.ext).await {
            Ok(_) => {}
            Err(err) => {
                log::error!("Failed to handle debugger connected: {}", err);
                // exit
                std::process::exit(1);
            }
        }
    });

    Ok(ResponseBody::Launch)
}
