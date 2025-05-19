mod cache;
mod error;
mod proto;

use cache::DebuggerCache;
pub use cache::*;
use dap::events::{Event, OutputEventBody};
use dap::server::ServerOutput;
pub use error::DebuggerError;
#[allow(unused)]
pub use proto::*;
use std::collections::HashMap;
use std::error::Error;
use std::io::Stdout;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;
use tokio::time::timeout;

type DebuggerResult<T> = Result<T, Box<dyn Error + Send>>;

#[derive(Debug)]
pub struct DebuggerConnection {
    read_stream: Option<OwnedReadHalf>,
    write_stream: Option<Arc<Mutex<OwnedWriteHalf>>>,
    reader_task: Option<JoinHandle<()>>,
    response_senders: Arc<Mutex<HashMap<MessageCMD, mpsc::Sender<Message>>>>,
    eval_seq_id: i64,
    eval_response: Arc<Mutex<HashMap<i64, mpsc::Sender<EvalRsp>>>>,
}

#[allow(unused)]
impl DebuggerConnection {
    pub fn new() -> Self {
        DebuggerConnection {
            read_stream: None,
            write_stream: None,
            reader_task: None,
            response_senders: Arc::new(Mutex::new(HashMap::new())),
            eval_seq_id: 0,
            eval_response: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn connect(&mut self, addr: &str, timeout_secs: Option<u64>) -> DebuggerResult<()> {
        let addr: SocketAddr = addr.parse().map_err(|e| DebuggerError::from(e))?;

        let stream = if let Some(secs) = timeout_secs {
            let connect_future = TcpStream::connect(addr);
            match timeout(Duration::from_secs(secs), connect_future).await {
                Ok(result) => result.map_err(|e| DebuggerError::from(e))?,
                Err(_) => {
                    return Err(DebuggerError::ConnectionError(format!(
                        "connect {} timeout",
                        addr
                    ))
                    .into());
                }
            }
        } else {
            TcpStream::connect(addr)
                .await
                .map_err(|e| DebuggerError::from(e))?
        };
        let (read_stream, write_stream) = stream.into_split();
        self.read_stream = Some(read_stream);
        self.write_stream = Some(Arc::new(Mutex::new(write_stream)));
        Ok(())
    }

    pub async fn listen(&mut self, addr: &str) -> DebuggerResult<()> {
        let addr: SocketAddr = addr.parse().map_err(|e| DebuggerError::from(e))?;
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| DebuggerError::from(e))?;

        let (stream, _) = listener
            .accept()
            .await
            .map_err(|e| DebuggerError::from(e))?;

        let (read_stream, write_stream) = stream.into_split();
        self.read_stream = Some(read_stream);
        self.write_stream = Some(Arc::new(Mutex::new(write_stream)));
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.write_stream.is_some()
    }

    pub async fn close(&mut self) {
        if let Some(handle) = self.reader_task.take() {
            handle.abort();
        }
        self.read_stream = None;
        self.write_stream = None;
    }

    pub fn start_reader_task(&mut self, ide_conn: Arc<std::sync::Mutex<ServerOutput<Stdout>>>) {
        if self.reader_task.is_some() {
            return;
        }

        let read_stream = self.read_stream.take();
        if let Some(stream) = read_stream {
            let senders = self.response_senders.clone();
            let eval_response = self.eval_response.clone();

            let handle = tokio::spawn(async move {
                let mut msg_id_string = String::new();
                let mut msg_json_string = String::new();
                let mut reader = BufReader::new(stream);
                loop {
                    msg_id_string.clear();
                    msg_json_string.clear();

                    match reader.read_line(&mut msg_id_string).await {
                        Ok(0) => {
                            log::error!("Connection closed by peer");
                            let mut ide_conn = ide_conn.lock().unwrap();
                            ide_conn.send_event(Event::Output(OutputEventBody {
                                category: Some(dap::types::OutputEventCategory::Console),
                                output: "Disconnected\n".to_string(),
                                ..Default::default()
                            }));

                            ide_conn.send_event(Event::Terminated(None));
                            break;
                        }
                        Ok(n) => {
                            if n == 0 {
                                break;
                            }
                        }
                        Err(e) => {
                            log::error!("Error reading from stream: {}", e);
                            break;
                        }
                    }

                    log::debug!("read message id {}", msg_id_string);

                    match reader.read_line(&mut msg_json_string).await {
                        Ok(0) => {
                            log::error!("Connection closed by peer");
                            let mut ide_conn = ide_conn.lock().unwrap();
                            ide_conn.send_event(Event::Output(OutputEventBody {
                                category: Some(dap::types::OutputEventCategory::Console),
                                output: "Disconnected\n".to_string(),
                                ..Default::default()
                            }));

                            ide_conn.send_event(Event::Terminated(None));
                            break;
                        }
                        Ok(n) => {
                            if n == 0 {
                                break;
                            }
                        }
                        Err(e) => {
                            log::error!("Error reading from stream: {}", e);
                            break;
                        }
                    }

                    log::debug!("read message json {}", msg_json_string);

                    let msg_id = match msg_id_string.trim().parse::<i32>() {
                        Ok(id) => id,
                        Err(e) => {
                            log::error!("Error parsing message ID: {}", e);
                            continue;
                        }
                    };

                    let message = match Message::from_str(
                        &msg_json_string,
                        MessageCMD::from(msg_id as i64),
                    ) {
                        Ok(msg) => msg,
                        Err(e) => {
                            log::error!("Error parsing message JSON: {}", e);
                            continue;
                        }
                    };

                    Self::dispatch_message(message, &senders, &eval_response).await;
                }
            });

            self.reader_task = Some(handle);
        }
    }

    async fn dispatch_message(
        message: Message,
        senders: &Arc<Mutex<HashMap<MessageCMD, mpsc::Sender<Message>>>>,
        eval_response: &Arc<Mutex<HashMap<i64, mpsc::Sender<EvalRsp>>>>,
    ) {
        let cmd = message.get_cmd();
        log::info!("dispatch message: {:?}", cmd);
        match cmd {
            MessageCMD::EvalRsp => {
                if let Message::EvalRsp(eval_rsp) = message {
                    let seq = eval_rsp.seq as i64;
                    log::info!("response seq: {}", seq);
                    let mut senders_guard = eval_response.lock().await;
                    if let Some(sender) = senders_guard.remove(&seq) {
                        let _ = sender.send(eval_rsp).await;
                    }
                }
            }
            _ => {
                let senders_guard = senders.lock().await;
                if let Some(sender) = senders_guard.get(&cmd) {
                    let _ = sender.send(message).await;
                }
            }
        }
    }

    pub async fn register_callback(&self, cmd: MessageCMD) -> Option<mpsc::Receiver<Message>> {
        if !self.is_connected() {
            return None;
        }

        let (tx, rx) = mpsc::channel(32); // 创建容量为32的通道

        let mut senders_guard = self.response_senders.lock().await;
        senders_guard.insert(cmd, tx);

        Some(rx)
    }

    async fn register_eval_callback(&self, seq: i64) -> Option<mpsc::Receiver<EvalRsp>> {
        if !self.is_connected() {
            return None;
        }

        let (tx, rx) = mpsc::channel(1); // 创建容量为32的通道

        let mut senders_guard = self.eval_response.lock().await;
        senders_guard.insert(seq, tx);

        Some(rx)
    }

    pub async fn send_message(&self, message: Message) -> DebuggerResult<()> {
        if let Some(stream) = &self.write_stream {
            let mut stream_guard = stream.lock().await;
            let json = match serde_json::to_string(&message) {
                Ok(json) => json,
                Err(e) => {
                    log::error!("serde fail: {}", e);
                    return Err(
                        DebuggerError::SerializationError(format!("serde fail: {}", e)).into(),
                    );
                }
            };

            let msg_id = message.get_cmd() as i32;
            let message_text = format!("{}\n{}\n", msg_id, json);
            log::debug!("send message: {}", message_text);
            match stream_guard
                .write_all(message_text.as_bytes())
                .await
                .map_err(|e| DebuggerError::IoError(e).into())
            {
                Ok(_) => {}
                Err(e) => {
                    log::error!("send message fail: {}", e);
                    return Err(e);
                }
            }
            log::debug!("send message ok");
            match stream_guard
                .flush()
                .await
                .map_err(|e| DebuggerError::IoError(e).into())
            {
                Ok(_) => {}
                Err(e) => {
                    log::error!("flush stream fail: {}", e);
                    return Err(e);
                }
            }
            log::debug!("flush stream ok");
            Ok(())
        } else {
            Err(DebuggerError::ConnectionError("not connected".to_string()).into())
        }
    }

    pub async fn send_request(&self, request: Message) -> DebuggerResult<Message> {
        if let Some(stream) = &self.write_stream {
            let mut stream_guard = stream.lock().await;
            let json = match serde_json::to_string(&request) {
                Ok(json) => json,
                Err(e) => {
                    log::error!("serde fail: {}", e);
                    return Err(
                        DebuggerError::SerializationError(format!("serde fail: {}", e)).into(),
                    );
                }
            };

            let msg_id = request.get_cmd() as i32;
            let message_text = format!("{}\n{}\n", msg_id, json);
            log::debug!("send message: {}", message_text);
            match stream_guard
                .write_all(message_text.as_bytes())
                .await
                .map_err(|e| DebuggerError::IoError(e).into())
            {
                Ok(_) => {}
                Err(e) => {
                    log::error!("send message fail: {}", e);
                    return Err(e);
                }
            }
            log::debug!("send message ok");

            match stream_guard
                .flush()
                .await
                .map_err(|e| DebuggerError::IoError(e).into())
            {
                Ok(_) => {}
                Err(e) => {
                    log::error!("flush stream fail: {}", e);
                    return Err(e);
                }
            }

            log::debug!("flush stream ok");

            drop(stream_guard);
            // 等待响应
            let receiver = self
                .register_callback(request.get_cmd().get_rsp_cmd())
                .await;
            if let Some(mut rx) = receiver {
                if let Some(response) = rx.recv().await {
                    return Ok(response);
                }
            }
        }

        Err(DebuggerError::ConnectionError("not connected".to_string()).into())
    }

    pub async fn eval_expr(
        &mut self,
        expression: String,
        cache_id: i64,
        depth: i64,
        frame_id: i64,
    ) -> DebuggerResult<EvalRsp> {
        if let Some(stream) = &self.write_stream {
            let seq = self.eval_seq_id;
            self.eval_seq_id += 1;
            let eval_req = EvalReq {
                cmd: MessageCMD::EvalReq as i64,
                seq: seq as i32,
                expr: expression.clone(),
                stack_level: frame_id as i32,
                depth: depth as i32,
                cache_id: cache_id as i32,
                value: None,
                set_value: None,
            };

            let mut stream_guard = stream.lock().await;
            let json = serde_json::to_string(&eval_req)
                .map_err(|e| DebuggerError::SerializationError(format!("serde fail: {}", e)))?;

            let msg_id = MessageCMD::EvalReq as i32;
            let message_text = format!("{}\n{}\n", msg_id, json);
            log::info!("request eval :{}, seq: {}", expression, seq);
            let receiver = self.register_eval_callback(seq).await;

            match stream_guard
                .write_all(message_text.as_bytes())
                .await
                .map_err(|e| DebuggerError::IoError(e).into())
            {
                Ok(_) => {}
                Err(e) => {
                    log::error!("send message fail: {}", e);
                    return Err(e);
                }
            }

            match stream_guard
                .flush()
                .await
                .map_err(|e| DebuggerError::IoError(e).into())
            {
                Ok(_) => {}
                Err(e) => {
                    log::error!("flush stream fail: {}", e);
                    return Err(e);
                }
            }

            drop(stream_guard);
            if let Some(mut rx) = receiver {
                if let Some(response) = rx.recv().await {
                    return Ok(response);
                }
            }
        }

        Err(DebuggerError::ConnectionError("not connected".to_string()).into())
    }
}

#[derive(Debug, Default)]
pub struct DebuggerData {
    pub stacks: Vec<Stack>,
    pub file_cache: HashMap<String, Option<String>>,
    pub extension: Vec<String>,
    pub sources: Vec<String>,
    pub current_frame_id: i64,
    pub cache: DebuggerCache,
    pub breakpoints: HashMap<(String, i64), BreakPoint>,
    pub breakpoint_id: i64,
}
