use std::{io::Stdout, sync::Arc};

use dap::server::ServerOutput;
use tokio::sync::Mutex;

use super::{DebuggerData, debugger::DebuggerConnection};

#[derive(Clone)]
pub struct DapSnapShot {
    pub debugger_conn: Arc<Mutex<DebuggerConnection>>,
    pub ide_conn: Arc<std::sync::Mutex<ServerOutput<Stdout>>>,
    pub data: Arc<Mutex<DebuggerData>>,
}
