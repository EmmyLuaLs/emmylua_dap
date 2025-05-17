use dap::responses::{ResponseBody, ThreadsResponse};
use tokio_util::sync::CancellationToken;

use crate::context::DapSnapShot;

use super::RequestResult;

pub async fn on_threads_request(dap: DapSnapShot, _: (), _: CancellationToken) -> RequestResult {
    Ok(ResponseBody::Threads(ThreadsResponse {
        threads: vec![dap::types::Thread {
            id: 1,
            name: "Main Thread".to_string(),
        }],
    }))
}
