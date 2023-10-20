use crate::context::Context;

pub struct Runtime {
    context: Context
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            context: Context::new()
        }
    }

    pub fn use_context(&mut self) -> &mut Context {
        &mut self.context
    }
}