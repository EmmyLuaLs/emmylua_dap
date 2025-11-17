mod debugger;
mod emmy_attach_debugger;
mod emmy_new_debugger;
mod snapshot;

use std::{collections::HashMap, future::Future, io::Stdout, sync::Arc};

use dap::{requests::Request, responses::Response, server::ServerOutput};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::handler::RequestResult;
pub use debugger::*;
pub use emmy_new_debugger::*;
pub use snapshot::DapSnapShot;

pub struct EmmyLuaDebugContext {
    debugger_conn: Arc<Mutex<debugger::DebuggerConnection>>,
    cancellations: Arc<Mutex<HashMap<i64, CancellationToken>>>,
    ide_conn: Arc<std::sync::Mutex<ServerOutput<Stdout>>>,
    data: Arc<Mutex<DebuggerData>>,
}

impl EmmyLuaDebugContext {
    pub fn new(ide_conn: Arc<std::sync::Mutex<ServerOutput<Stdout>>>) -> Self {
        EmmyLuaDebugContext {
            debugger_conn: Arc::new(Mutex::new(debugger::DebuggerConnection::new())),
            cancellations: Arc::new(Mutex::new(HashMap::new())),
            ide_conn,
            data: Arc::new(Mutex::new(DebuggerData::default())),
        }
    }

    fn snapshot(&self) -> DapSnapShot {
        DapSnapShot {
            debugger_conn: self.debugger_conn.clone(),
            ide_conn: self.ide_conn.clone(),
            data: self.data.clone(),
        }
    }

    pub async fn task<F, P, Fut>(&self, request: Request, param: P, exec: F)
    where
        F: FnOnce(DapSnapShot, P, CancellationToken) -> Fut + Send + 'static,
        Fut: Future<Output = RequestResult> + Send + 'static,
        P: Send + 'static,
    {
        let cancel_token = CancellationToken::new();
        let req_id = request.seq;
        {
            let mut cancellations = self.cancellations.lock().await;
            cancellations.insert(req_id, cancel_token.clone());
        }

        let cancellations = self.cancellations.clone();
        let output = self.ide_conn.clone();
        let snapshot = self.snapshot();
        tokio::spawn(async move {
            let res = exec(snapshot, param, cancel_token.clone()).await;
            let response = if cancel_token.is_cancelled() {
                Some(request.cancellation())
            } else if let Err(err) = res {
                Some(request.error(&err.to_string()))
            } else if let Ok(body) = res {
                Some(request.success(body))
            } else {
                None
            };

            if let Some(response) = response {
                match output.lock().unwrap().respond(response) {
                    Err(server_err) => {
                        log::error!(
                            "Failed to send response for request {}: {:?}",
                            req_id,
                            server_err
                        );
                        // Don't exit - DAP clients expect responses even on errors
                        // The error will be logged and the client should handle the situation
                    }
                    Ok(_) => {
                        log::debug!("Successfully sent response for request {}", req_id);
                    }
                }
            } else {
                log::warn!("No response generated for request {}", req_id);
            }

            let mut cancellations = cancellations.lock().await;
            cancellations.remove(&req_id);
        });
    }

    pub async fn cancel(&self, req_id: i64) {
        let cancellations = self.cancellations.lock().await;
        if let Some(cancel_token) = cancellations.get(&req_id) {
            cancel_token.cancel();
        }
    }

    pub async fn respond(&self, response: Response) {
        let mut output = self.ide_conn.lock().unwrap();
        let result = output.respond(response);
        if let Err(server_err) = result {
            log::error!("Failed to send response: {:?}", server_err);
            // Don't panic - log the error and continue
            // The client should handle missing responses gracefully
        } else {
            log::debug!("Successfully sent response");
        }
    }
}
