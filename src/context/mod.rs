use std::{
    io::Stdout,
    sync::{Arc, Mutex},
};

use dap::server::ServerOutput;

#[derive(Debug)]
pub struct EmmyLuaDebugContext {}

impl EmmyLuaDebugContext {
    pub fn new() -> Self {
        EmmyLuaDebugContext {}
    }
}
