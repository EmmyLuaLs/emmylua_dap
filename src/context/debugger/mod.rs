mod error;
mod proto;

use error::DebuggerError;
#[allow(unused)]
pub use proto::*;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::timeout;

type DebuggerResult<T> = Result<T, Box<dyn Error + Send>>;

#[derive(Debug)]
pub struct DebuggerConnection {
    stream: Option<Arc<Mutex<TcpStream>>>, // 增加 stream 字段
}

impl DebuggerConnection {
    pub fn new() -> Self {
        DebuggerConnection {
            stream: None, // 初始化为 None
        }
    }

    pub async fn connect(&mut self, addr: &str, timeout_secs: Option<u64>) -> DebuggerResult<()> {
        let addr: SocketAddr = addr.parse().map_err(|e| DebuggerError::from(e))?;

        let stream = if let Some(secs) = timeout_secs {
            let connect_future = TcpStream::connect(addr);
            match timeout(Duration::from_secs(secs), connect_future).await {
                Ok(result) => result.map_err(|e| DebuggerError::from(e))?,
                Err(_) => {
                    return Err(
                        DebuggerError::ConnectionError(format!("connect {} timeout", addr)).into(),
                    );
                }
            }
        } else {
            TcpStream::connect(addr)
                .await
                .map_err(|e| DebuggerError::from(e))?
        };

        self.stream = Some(Arc::new(Mutex::new(stream)));
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
        Ok(())
    }

    pub async fn send_message(&self, message: &[u8]) -> DebuggerResult<()> {
        if let Some(stream) = &self.stream {
            let mut stream = stream.lock().await;
            stream
                .write_all(message)
                .await
                .map_err(|e| DebuggerError::from(e))?;
            stream.flush().await.map_err(|e| DebuggerError::from(e))?;
            Ok(())
        } else {
            Err(DebuggerError::ConnectionError("没有建立连接".to_string()).into())
        }
    }

    pub async fn read_message(&self, buffer: &mut [u8]) -> DebuggerResult<usize> {
        if let Some(stream) = &self.stream {
            let mut stream = stream.lock().await;
            let bytes_read = stream
                .read(buffer)
                .await
                .map_err(|e| DebuggerError::from(e))?;
            Ok(bytes_read)
        } else {
            Err(DebuggerError::ConnectionError("没有建立连接".to_string()).into())
        }
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    pub async fn close(&mut self) {
        self.stream = None;
    }
}
