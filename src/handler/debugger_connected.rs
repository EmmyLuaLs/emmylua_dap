use std::error::Error;

use dap::events::{Event, OutputEventBody};

use crate::{
    context::{DapSnapShot, InitReq, Message, ReadyReq},
    handler::RequestHandlerError,
};

pub async fn on_debugger_connected(
    dap: DapSnapShot,
    ext: Vec<String>,
) -> Result<(), Box<dyn Error + Send>> {
    log::info!("Debugger connected");

    {
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
        let debugger_conn = dap.debugger_conn.lock().await;
        debugger_conn
            .send_message(Message::InitReq(InitReq {
                // todo
                emmy_helper: "".to_string(),
                ext,
            }))
            .await?;

        // todo add breakpoints

        debugger_conn
            .send_message(Message::ReadyReq(ReadyReq {}))
            .await?;
    }

    {
        let mut ide_conn = dap.ide_conn.lock().unwrap();
        ide_conn
            .send_event(Event::Initialized)
            .map_err(|err| RequestHandlerError::ServerError(err))?;
    }
    Ok(())
}
