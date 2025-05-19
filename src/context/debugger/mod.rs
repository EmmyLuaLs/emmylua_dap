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
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;
use tokio::time::timeout;

type DebuggerResult<T> = Result<T, Box<dyn Error + Send>>;

#[allow(unused)]
#[derive(Debug)]
pub struct DebuggerConnection {
    read_stream: Option<Arc<Mutex<OwnedReadHalf>>>,
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
        self.read_stream = Some(Arc::new(Mutex::new(read_stream)));
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
        self.read_stream = Some(Arc::new(Mutex::new(read_stream)));
        self.write_stream = Some(Arc::new(Mutex::new(write_stream)));
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.read_stream.is_some() && self.write_stream.is_some()
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

        if let Some(stream) = &self.read_stream {
            let stream_clone = stream.clone();
            let senders = self.response_senders.clone();
            let eval_response = self.eval_response.clone();

            let handle = tokio::spawn(async move {
                let mut buffer = vec![0u8; 4096];
                let mut pos = 0;

                loop {
                    let read_result = {
                        let mut stream_guard = stream_clone.lock().await;
                        stream_guard.read(&mut buffer[pos..]).await
                    };

                    match read_result {
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
                            log::debug!("read {} bytes, total bytes {}", n, pos);
                            pos += n; // 解析消息格式：第一行是整数ID，第二行是JSON内容
                            let mut start = 0;
                            let mut id_line = None;
                            let mut i = 0;

                            while i < pos {
                                // 查找换行符
                                if buffer[i] == b'\n' {
                                    if id_line.is_none() {
                                        // 解析第一行作为消息ID
                                        if let Ok(id_str) = std::str::from_utf8(&buffer[start..i]) {
                                            if let Ok(_msg_id) = id_str.parse::<i32>() {
                                                // 记录ID并继续寻找JSON内容
                                                id_line = Some(start);
                                                start = i + 1;
                                            }
                                        }
                                    } else {
                                        // 已有ID，这一行是JSON内容
                                        if let Ok(msg_str) = std::str::from_utf8(&buffer[start..i])
                                        {
                                            if let Ok(message) =
                                                serde_json::from_str::<Message>(msg_str)
                                            {
                                                Self::dispatch_message(
                                                    message,
                                                    &senders,
                                                    &eval_response,
                                                )
                                                .await;
                                            } else {
                                                log::error!("parse fail: {}", msg_str);
                                            }
                                        }
                                        // 重置解析状态，准备解析下一条消息
                                        id_line = None;
                                        start = i + 1;
                                    }
                                }
                                i += 1;
                            }

                            log::debug!("parsed {} bytes", start);

                            // 处理完整消息后移动剩余数据到缓冲区开头
                            if start > 0 {
                                buffer.copy_within(start..pos, 0);
                                pos -= start;
                            }

                            if pos > buffer.len() - 1024 {
                                log::debug!(
                                    "current buffer used size {} bytes, extend buffer total size to {} bytes",
                                    pos,
                                    buffer.len() * 2
                                );
                                buffer.resize(buffer.len() * 2, 0);
                            }
                        }
                        Err(e) => {
                            log::error!("Error reading from stream: {}", e);
                            break;
                        }
                    }
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
                expr: expression,
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
    pub current_frame_id: i64,
    pub cache: DebuggerCache,
    pub breakpoints: HashMap<(String, i64), BreakPoint>,
    pub breakpoint_id: i64,
}
