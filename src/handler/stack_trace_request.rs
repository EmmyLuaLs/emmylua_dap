use dap::{requests::StackTraceArguments, responses::ResponseBody, types::Source};
use tokio_util::sync::CancellationToken;

use crate::context::{DapSnapShot, DebuggerData, Stack};

use super::RequestResult;

pub async fn on_stack_trace_request(
    dap: DapSnapShot,
    _: StackTraceArguments,
    _: CancellationToken,
) -> RequestResult {
    let mut data = dap.data.lock().await;

    let mut stack_frames = vec![];
    let stacks = data.stacks.iter().map(StackInfo::from).collect::<Vec<_>>();
    for stack in stacks {
        let file_string = stack.file.clone();
        if stack.line <= 0 {
            continue;
        }

        // todo find real file
        let file_path = find_file_path(&mut data, file_string.clone()).await?;
        let stack_frame = dap::types::StackFrame {
            id: stack.level as i64,
            name: stack.function_name.clone(),
            source: Some(Source {
                name: Some(file_string),
                path: file_path,
                ..Default::default()
            }),
            line: stack.line as i64,
            ..Default::default()
        };
        stack_frames.push(stack_frame);
    }

    let total_frames = stack_frames.len() as i64;
    Ok(ResponseBody::StackTrace(
        dap::responses::StackTraceResponse {
            stack_frames,
            total_frames: Some(total_frames),
        },
    ))
}

pub struct StackInfo {
    pub level: i32,
    pub function_name: String,
    pub file: String,
    pub line: i32,
}

impl StackInfo {
    pub fn from(stack: &Stack) -> Self {
        StackInfo {
            level: stack.level,
            function_name: stack.function_name.clone(),
            file: stack.file.clone(),
            line: stack.line,
        }
    }
}

async fn find_file_path(
    data: &mut DebuggerData,
    chunkname: String,
) -> Result<Option<String>, Box<dyn std::error::Error + Send>> {
    if let Some(real_file_path) = data.file_cache.get(&chunkname) {
        return Ok(real_file_path.clone());
    }

    // For Lua files, the chunkname might be a file path or have special formatting
    // Try to clean it up and find the actual file
    let basic_file = if chunkname.starts_with('@') {
        // Remove the @ prefix which is common in Lua chunk names
        chunkname[1..].to_string()
    } else {
        chunkname.clone()
    };

    let mut file_paths = vec![];
    for ext in &data.extension {
        if basic_file.ends_with(ext) {
            file_paths.push(basic_file.clone());
        }
    }

    if file_paths.is_empty() {
        for ext in &data.extension {
            file_paths.push(format!("{}{}", basic_file, ext));
        }
    }

    for file_path in &file_paths {
        if let Ok(metadata) = tokio::fs::metadata(file_path).await {
            if metadata.is_file() {
                data.file_cache
                    .insert(chunkname.clone(), Some(file_path.clone()));
                return Ok(Some(file_path.clone()));
            }
        }
    }

    Ok(None)
}
