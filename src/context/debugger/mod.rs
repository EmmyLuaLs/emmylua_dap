mod error;
mod proto;

use error::DebuggerError;
#[allow(unused)]
pub use proto::*;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::timeout;

type DebuggerResult<T> = Result<T, Box<dyn Error + Send>>;

#[derive(Debug)]
pub struct DebuggerConnection {
    stream: Option<Arc<Mutex<TcpStream>>>,
    reader_task: Option<JoinHandle<()>>,
    response_senders: Arc<Mutex<HashMap<MessageCMD, mpsc::Sender<Message>>>>,
}

impl DebuggerConnection {
    pub fn new() -> Self {
        DebuggerConnection {
            stream: None,
            reader_task: None,
            response_senders: Arc::new(Mutex::new(HashMap::new())),
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

        self.stream = Some(Arc::new(Mutex::new(stream)));
        self.start_reader_task();
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

        self.stream = Some(Arc::new(Mutex::new(stream)));
        self.start_reader_task();
        Ok(())
    }
    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    pub async fn close(&mut self) {
        if let Some(handle) = self.reader_task.take() {
            handle.abort();
        }
        self.stream = None;
    }

    fn start_reader_task(&mut self) {
        if self.reader_task.is_some() {
            return;
        }

        if let Some(stream) = &self.stream {
            let stream_clone = stream.clone();
            let senders = self.response_senders.clone();

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
                            break;
                        }
                        Ok(n) => {
                            pos += n;

                            // 尝试解析缓冲区中的所有消息
                            // 这里需要根据协议格式修改，假设每条消息以换行符结尾
                            let mut start = 0;
                            for i in 0..pos {
                                if buffer[i] == b'\n' {
                                    if let Ok(msg_str) = std::str::from_utf8(&buffer[start..i]) {
                                        if let Ok(message) =
                                            serde_json::from_str::<Message>(msg_str)
                                        {
                                            Self::dispatch_message(message, &senders).await;
                                        }
                                    }
                                    start = i + 1;
                                }
                            }

                            // 处理完整消息后移动剩余数据到缓冲区开头
                            if start > 0 {
                                buffer.copy_within(start..pos, 0);
                                pos -= start;
                            }

                            if pos > buffer.len() - 1024 {
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

    /// 分发消息到对应的接收器
    async fn dispatch_message(
        message: Message,
        senders: &Arc<Mutex<HashMap<MessageCMD, mpsc::Sender<Message>>>>,
    ) {
        let cmd = match &message {
            Message::InitReq(_) => MessageCMD::InitReq,
            Message::InitRsp(_) => MessageCMD::InitRsp,
            Message::ReadyReq(_) => MessageCMD::ReadyReq,
            Message::ReadyRsp(_) => MessageCMD::ReadyRsp,
            Message::AddBreakPointReq(_) => MessageCMD::AddBreakPointReq,
            Message::AddBreakPointRsp(_) => MessageCMD::AddBreakPointRsp,
            Message::RemoveBreakPointReq(_) => MessageCMD::RemoveBreakPointReq,
            Message::RemoveBreakPointRsp(_) => MessageCMD::RemoveBreakPointRsp,
            Message::ActionReq(_) => MessageCMD::ActionReq,
            Message::ActionRsp(_) => MessageCMD::ActionRsp,
            Message::EvalReq(_) => MessageCMD::EvalReq,
            Message::EvalRsp(_) => MessageCMD::EvalRsp,
            Message::BreakNotify(_) => MessageCMD::BreakNotify,
            Message::AttachedNotify(_) => MessageCMD::AttachedNotify,
            Message::StartHookReq(_) => MessageCMD::StartHookReq,
            Message::StartHookRsp(_) => MessageCMD::StartHookRsp,
            Message::LogNotify(_) => MessageCMD::LogNotify,
        };

        let senders_guard = senders.lock().await;
        if let Some(sender) = senders_guard.get(&cmd) {
            let _ = sender.send(message).await;
        }
    }

    /// 注册回调接收特定类型的消息
    pub async fn register_callback(&self, cmd: MessageCMD) -> Option<mpsc::Receiver<Message>> {
        if !self.is_connected() {
            return None;
        }

        let (tx, rx) = mpsc::channel(32); // 创建容量为32的通道

        let mut senders_guard = self.response_senders.lock().await;
        senders_guard.insert(cmd, tx);

        Some(rx)
    }

    /// 发送消息
    pub async fn send_message(&self, message: &Message) -> DebuggerResult<()> {
        if let Some(stream) = &self.stream {
            let mut stream_guard = stream.lock().await;

            let json = serde_json::to_string(&message)
                .map_err(|e| DebuggerError::SerializationError(format!("序列化消息失败: {}", e)))?;

            // 发送消息，加上换行符作为消息分隔符
            stream_guard
                .write_all((json + "\n").as_bytes())
                .await
                .map_err(|e| DebuggerError::IoError(e).into())?;

            stream_guard
                .flush()
                .await
                .map_err(|e| DebuggerError::IoError(e).into())?;

            Ok(())
        } else {
            Err(DebuggerError::ConnectionError("没有建立连接".to_string()).into())
        }
    }
}
