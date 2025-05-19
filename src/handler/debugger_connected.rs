use std::error::Error;

use dap::events::{Event, OutputEventBody};

use crate::{
    context::{AddBreakPointReq, DapSnapShot, InitReq, Message, ReadyReq},
    handler::RequestHandlerError,
};

pub async fn after_debugger_connected(
    dap: DapSnapShot,
    ext: Vec<String>,
) -> Result<(), Box<dyn Error + Send>> {
    {
        log::info!("on debugger connected");
        let mut ide_conn = dap.ide_conn.lock().unwrap();
        ide_conn
            .send_event(Event::Output(OutputEventBody {
                category: Some(dap::types::OutputEventCategory::Console),
                output: "Debugger connected\n".to_string(),
                ..Default::default()
            }))
            .map_err(|err| RequestHandlerError::ServerError(err))?;
    }

    {
        log::info!("send init req to debugger");
        let debugger_conn = dap.debugger_conn.lock().await;
        log::info!("get debugger_conn lock");
        let rsp = debugger_conn
            .send_request(Message::InitReq(InitReq {
                // todo
                emmy_helper: "".to_string(),
                ext,
            }))
            .await?;
        log::info!("get rsp {:#?}", rsp);

        log::info!("Sending all breakpoints to debugger");
        let data = dap.data.lock().await;
        let breakpoints = data.breakpoints.values().cloned().collect::<Vec<_>>();
        debugger_conn
            .send_message(Message::AddBreakPointReq(AddBreakPointReq {
                break_points: breakpoints,
                clear: true,
            }))
            .await?;

        log::info!("Sending ready req to debugger");
        debugger_conn
            .send_message(Message::ReadyReq(ReadyReq {}))
            .await?;
    }

    {
        log::info!("send initialized event to ide");
        let mut ide_conn = dap.ide_conn.lock().unwrap();
        ide_conn
            .send_event(Event::Initialized)
            .map_err(|err| RequestHandlerError::ServerError(err))?;
    }
    Ok(())
}
