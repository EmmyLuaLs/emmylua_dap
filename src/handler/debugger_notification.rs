use dap::{
    events::{Event, OutputEventBody, StoppedEventBody},
    types::StoppedEventReason,
};

use crate::context::{DapSnapShot, LogNotify, Message, MessageCMD};

pub async fn register_debugger_notification(dap: DapSnapShot) {
    let debugger_conn = dap.debugger_conn.lock().await;
    let break_hit_notification = debugger_conn
        .register_callback(MessageCMD::BreakNotify)
        .await;
    let log_notification = debugger_conn.register_callback(MessageCMD::LogNotify).await;

    if let Some(mut break_hit_receiver) = break_hit_notification {
        let ide_conn = dap.ide_conn.clone();
        let data = dap.data.clone();
        tokio::spawn(async move {
            while let Some(break_hit) = break_hit_receiver.recv().await {
                if let Message::BreakNotify(break_hit) = break_hit {
                    {
                        let mut data = data.lock().await;
                        data.stacks = break_hit.stacks;
                    }

                    let mut ide_conn = ide_conn.lock().unwrap();
                    match ide_conn.send_event(Event::Stopped(StoppedEventBody {
                        reason: StoppedEventReason::String("breakpoint".to_string()),
                        thread_id: Some(1),
                        description: None,
                        text: None,
                        all_threads_stopped: None,
                        preserve_focus_hint: None,
                        hit_breakpoint_ids: None,
                    })) {
                        Ok(_) => {}
                        Err(err) => {
                            log::error!("Failed to send Break Hit notification: {:?}", err);
                        }
                    }
                }
            }
        });
    } else {
        log::error!("Failed to register Break Hit notification");
    }

    if let Some(mut log_receiver) = log_notification {
        let ide_conn = dap.ide_conn.clone();
        tokio::spawn(async move {
            while let Some(log) = log_receiver.recv().await {
                if let Message::LogNotify(LogNotify { message }) = log {
                    let mut ide_conn = ide_conn.lock().unwrap();
                    match ide_conn.send_event(Event::Output(OutputEventBody {
                        category: Some(dap::types::OutputEventCategory::Console),
                        output: message,
                        ..Default::default()
                    })) {
                        Ok(_) => {}
                        Err(err) => {
                            log::error!("Failed to send Log notification: {:?}", err);
                        }
                    }
                }
            }
        });
    } else {
        log::error!("Failed to register Log notification");
    }
}
