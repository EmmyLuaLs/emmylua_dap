use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum MessageCMD {
    Unknown,

    InitReq,
    InitRsp,

    ReadyReq,
    ReadyRsp,

    AddBreakPointReq,
    AddBreakPointRsp,

    RemoveBreakPointReq,
    RemoveBreakPointRsp,

    ActionReq,
    ActionRsp,

    EvalReq,
    EvalRsp,

    // lua -> ide
    BreakNotify,
    AttachedNotify,

    StartHookReq,
    StartHookRsp,

    LogNotify,
}

impl MessageCMD {
    pub fn get_rsp_cmd(&self) -> MessageCMD {
        match self {
            MessageCMD::InitReq => MessageCMD::InitRsp,
            MessageCMD::ReadyReq => MessageCMD::ReadyRsp,
            MessageCMD::AddBreakPointReq => MessageCMD::AddBreakPointRsp,
            MessageCMD::RemoveBreakPointReq => MessageCMD::RemoveBreakPointRsp,
            MessageCMD::ActionReq => MessageCMD::ActionRsp,
            MessageCMD::EvalReq => MessageCMD::EvalRsp,
            _ => MessageCMD::Unknown,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    InitReq(InitReq),
    InitRsp(InitRsp),

    ReadyReq(ReadyReq),
    ReadyRsp(ReadyRsp),

    AddBreakPointReq(AddBreakPointReq),
    AddBreakPointRsp(AddBreakPointRsp),

    RemoveBreakPointReq(RemoveBreakPointReq),
    RemoveBreakPointRsp(RemoveBreakPointRsp),

    ActionReq(ActionReq),
    ActionRsp(ActionRsp),

    EvalReq(EvalReq),
    EvalRsp(EvalRsp),

    BreakNotify(BreakNotify),
    AttachedNotify(AttachedNotify),

    StartHookReq(StartHookReq),
    StartHookRsp(StartHookRsp),

    LogNotify(LogNotify),
}

impl Message {
    pub fn get_cmd(&self) -> MessageCMD {
        match self {
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
        }
    }
}

// value type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ValueType {
    TNIL,
    TBOOLEAN,
    TLIGHTUSERDATA,
    TNUMBER,
    TSTRING,
    TTABLE,
    TFUNCTION,
    TUSERDATA,
    TTHREAD,
    GROUP,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableNameType {
    NString,
    NNumber,
    NComplex,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variable {
    pub name: String,
    pub name_type: ValueType,
    pub value: String,
    pub value_type: ValueType,
    pub value_type_name: String,
    pub cache_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Variable>>,
}

// 调用栈结构
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stack {
    pub file: String,
    pub line: i32,
    pub function_name: String,
    pub level: i32,
    pub local_variables: Vec<Variable>,
    pub upvalue_variables: Vec<Variable>,
}

// 断点结构
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BreakPoint {
    pub file: String,
    pub line: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_condition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_message: Option<String>,
}

// 初始化请求
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitReq {
    pub emmy_helper: String,
    pub ext: Vec<String>,
}

// 初始化响应
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitRsp {
    pub version: String,
}

// 添加断点请求
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddBreakPointReq {
    pub break_points: Vec<BreakPoint>,
    pub clear: bool,
}

// 添加断点响应
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddBreakPointRsp {}

// 删除断点请求
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveBreakPointReq {
    pub break_points: Vec<BreakPoint>,
}

// 删除断点响应
#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveBreakPointRsp {}

// 调试动作枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebugAction {
    Break,
    Continue,
    StepOver,
    StepIn,
    StepOut,
    Stop,
}

// 调试动作请求
#[derive(Debug, Serialize, Deserialize)]
pub struct ActionReq {
    pub action: DebugAction,
}

// 调试动作响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ActionRsp {}

// 中断通知
#[derive(Debug, Serialize, Deserialize)]
pub struct BreakNotify {
    pub stacks: Vec<Stack>,
}

// 求值请求
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalReq {
    pub seq: i32,
    pub expr: String,
    pub stack_level: i32,
    pub depth: i32,
    pub cache_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_value: Option<bool>,
}

// 求值响应
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalRsp {
    pub seq: i32,
    pub success: bool,
    pub error: String,
    pub value: Variable,
}

// Ready请求
#[derive(Debug, Serialize, Deserialize)]
pub struct ReadyReq {}

// Ready响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ReadyRsp {}

// 附加通知
#[derive(Debug, Serialize, Deserialize)]
pub struct AttachedNotify {}

// 开始Hook请求
#[derive(Debug, Serialize, Deserialize)]
pub struct StartHookReq {}

// 开始Hook响应
#[derive(Debug, Serialize, Deserialize)]
pub struct StartHookRsp {}

// 日志通知
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogNotify {
    pub message: String,
}
