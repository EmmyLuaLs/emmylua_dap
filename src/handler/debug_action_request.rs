use dap::responses::{ContinueResponse, ResponseBody};
use tokio_util::sync::CancellationToken;

use crate::context::{ActionReq, DapSnapShot, DebugAction, Message, MessageCMD};

use super::RequestResult;

pub async fn on_pause_request(dap: DapSnapShot, _: (), _: CancellationToken) -> RequestResult {
    log::info!("Received Pause request");
    let debugger_conn = dap.debugger_conn.lock().await;
    debugger_conn
        .send_message(Message::ActionReq(ActionReq {
            cmd: MessageCMD::ActionReq as i64,
            action: DebugAction::Break,
        }))
        .await?;

    Ok(ResponseBody::Pause)
}

pub async fn on_continue_request(dap: DapSnapShot, _: (), _: CancellationToken) -> RequestResult {
    log::info!("Received Continue request");
    let debugger_conn = dap.debugger_conn.lock().await;
    debugger_conn
        .send_message(Message::ActionReq(ActionReq {
            cmd: MessageCMD::ActionReq as i64,
            action: DebugAction::Continue,
        }))
        .await?;

    Ok(ResponseBody::Continue(ContinueResponse {
        all_threads_continued: None,
    }))
}

pub async fn on_step_in_request(dap: DapSnapShot, _: (), _: CancellationToken) -> RequestResult {
    log::info!("Received StepIn request");
    let debugger_conn = dap.debugger_conn.lock().await;
    debugger_conn
        .send_message(Message::ActionReq(ActionReq {
            cmd: MessageCMD::ActionReq as i64,
            action: DebugAction::StepIn,
        }))
        .await?;

    Ok(ResponseBody::StepIn)
}

pub async fn on_step_out_request(dap: DapSnapShot, _: (), _: CancellationToken) -> RequestResult {
    log::info!("Received StepOut request");
    let debugger_conn = dap.debugger_conn.lock().await;
    debugger_conn
        .send_message(Message::ActionReq(ActionReq {
            cmd: MessageCMD::ActionReq as i64,
            action: DebugAction::StepOut,
        }))
        .await?;

    Ok(ResponseBody::StepOut)
}

pub async fn on_next_request(dap: DapSnapShot, _: (), _: CancellationToken) -> RequestResult {
    log::info!("Received Next request");
    let debugger_conn = dap.debugger_conn.lock().await;
    debugger_conn
        .send_message(Message::ActionReq(ActionReq {
            cmd: MessageCMD::ActionReq as i64,
            action: DebugAction::StepOver,
        }))
        .await?;

    Ok(ResponseBody::Next)
}
