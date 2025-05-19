use dap::{
    requests::SetBreakpointsArguments,
    responses::{ResponseBody, SetBreakpointsResponse},
};
use tokio_util::sync::CancellationToken;

use crate::context::{AddBreakPointReq, BreakPoint, DapSnapShot, Message};

use super::RequestResult;

pub async fn on_set_breakpoints_request(
    dap: DapSnapShot,
    set_breakpoints_arguments: SetBreakpointsArguments,
    _: CancellationToken,
) -> RequestResult {
    log::info!(
        "Received SetBreakpoints request: {:#?}",
        set_breakpoints_arguments
    );
    let source = set_breakpoints_arguments.source;
    let mut response_breakpoints = vec![];
    if let Some(path) = source.path {
        let mut data = dap.data.lock().await;
        if let Some(breakpoints) = set_breakpoints_arguments.breakpoints {
            for breakpoint in breakpoints {
                let line = breakpoint.line;
                let key = (path.clone(), line);
                let debugger_point = BreakPoint {
                    file: path.clone(),
                    line: line as i32,
                    condition: breakpoint.condition.clone(),
                    hit_condition: breakpoint.hit_condition.clone(),
                    log_message: breakpoint.log_message.clone(),
                };
                data.breakpoints.insert(key, debugger_point);
                let id = data.breakpoint_id;
                data.breakpoint_id += 1;
                let response_breakpoint = dap::types::Breakpoint {
                    verified: true,
                    id: Some(id),
                    line: Some(line),
                    ..Default::default()
                };

                response_breakpoints.push(response_breakpoint);
            }

            send_all_breakpoints(dap.clone()).await;
        }
    } else {
        log::error!("No path provided in source");
    }

    Ok(ResponseBody::SetBreakpoints(SetBreakpointsResponse {
        breakpoints: response_breakpoints,
    }))
}

pub async fn send_all_breakpoints(dap: DapSnapShot) {
    let data = dap.data.lock().await;
    let breakpoints = data.breakpoints.values().cloned().collect::<Vec<_>>();
    let debugger_conn = dap.debugger_conn.lock().await;
    match debugger_conn
        .send_message(Message::AddBreakPointReq(AddBreakPointReq {
            break_points: breakpoints,
            clear: true,
        }))
        .await
    {
        Ok(_) => {}
        Err(err) => {
            log::error!("Failed to send breakpoints: {}", err);
        }
    }
}
