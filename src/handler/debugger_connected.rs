use std::error::Error;

use dap::events::{Event, OutputEventBody};

use crate::{
    context::{AddBreakPointReq, DapSnapShot, InitReq, Message, MessageCMD, ReadyReq},
    handler::RequestHandlerError,
};

pub async fn after_debugger_connected(
    dap: DapSnapShot,
    ext: Vec<String>,
) -> Result<(), Box<dyn Error + Send>> {
    {
        log::info!("on debugger connected");
        let mut ide_conn = dap.ide_conn.lock().unwrap();
        match ide_conn.send_event(Event::Output(OutputEventBody {
            category: Some(dap::types::OutputEventCategory::Console),
            output: "Debugger connected\n".to_string(),
            ..Default::default()
        })) {
            Ok(_) => {}
            Err(err) => {
                log::error!("Failed to send output event: {:?}", err);
                // Continue anyway - this is not critical
            }
        }
    }

    {
        log::info!("send init req to debugger");
        let debugger_conn = dap.debugger_conn.lock().await;
        log::info!("get debugger_conn lock");
        debugger_conn
            .send_message(Message::InitReq(InitReq {
                cmd: MessageCMD::InitReq as i64,
                emmy_helper: "".to_string(),
                ext,
            }))
            .await?;

        log::info!("Sending all breakpoints to debugger");
        let data = dap.data.lock().await;
        let breakpoints = data.breakpoints.values().cloned().collect::<Vec<_>>();
        debugger_conn
            .send_message(Message::AddBreakPointReq(AddBreakPointReq {
                cmd: MessageCMD::AddBreakPointReq as i64,
                break_points: breakpoints,
                clear: true,
            }))
            .await?;

        log::info!("Sending ready req to debugger");
        debugger_conn
            .send_message(Message::ReadyReq(ReadyReq {
                cmd: MessageCMD::ReadyReq as i64,
            }))
            .await?;
    }

    {
        log::info!("send initialized event to ide");
        let mut ide_conn = dap.ide_conn.lock().unwrap();
        match ide_conn.send_event(Event::Initialized) {
            Ok(_) => {
                log::info!("Successfully sent initialized event");
            }
            Err(err) => {
                log::error!("Failed to send initialized event: {:?}", err);
                return Err(RequestHandlerError::ServerError(err).into());
            }
        }
    }
    Ok(())
}
