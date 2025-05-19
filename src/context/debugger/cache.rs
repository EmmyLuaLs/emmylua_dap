use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use super::{DebuggerConnection, Stack, ValueType, Variable};

#[derive(Debug)]
pub struct DebuggerCache {
    cache_id: i64,
    caches: HashMap<i64, DebuggerCacheItem>,
}

impl DebuggerCache {
    pub fn new() -> Self {
        DebuggerCache {
            cache_id: 1,
            caches: HashMap::new(),
        }
    }
}

impl Default for DebuggerCache {
    fn default() -> Self {
        Self::new()
    }
}

impl DebuggerCache {
    pub fn get_cache(&self, id: i64) -> Option<DebuggerCacheItem> {
        self.caches.get(&id).cloned()
    }

    pub fn get_cache_ref(&self, id: i64) -> Option<&DebuggerCacheItem> {
        self.caches.get(&id)
    }

    pub fn add_cache(&mut self, item: DebuggerCacheItem) -> i64 {
        let cache_id = item.get_ref_id();
        self.caches.insert(cache_id, item);
        self.cache_id
    }

    pub fn allocate_cache_id(&mut self) -> i64 {
        let cache_id = self.cache_id;
        self.cache_id += 1;
        cache_id
    }
}

#[derive(Debug, Clone)]
pub struct DebuggerCacheRef<T> {
    pub id: i64,
    pub item: T,
}

impl<T> DebuggerCacheRef<T> {
    pub fn new(id: i64, item: T) -> Self {
        DebuggerCacheRef { id, item }
    }
}

#[derive(Debug, Clone)]
pub struct DebuggerVariable {
    pub var: Variable,
    pub parent_ref_id: i64,
}

impl DebuggerVariable {
    pub fn get_expr(&self, cache: &DebuggerCache) -> String {
        let mut arr: Vec<String> = vec![];
        let mut n: Option<&DebuggerVariable> = Some(self);
        while let Some(var) = n {
            if var.var.value_type != ValueType::GROUP {
                arr.push(var.var.name.clone());
            }

            if var.parent_ref_id != 0 {
                let parent = cache.get_cache_ref(var.parent_ref_id);
                match parent {
                    Some(DebuggerCacheItem::Variable(var_ref)) => {
                        n = Some(&var_ref.item);
                    }
                    _ => {
                        n = None;
                    }
                }
            } else {
                n = None;
            }
        }
        arr.reverse();
        arr.join(".")
    }
}

#[derive(Debug, Clone)]
pub enum DebuggerCacheItem {
    Stack(Arc<DebuggerCacheRef<Stack>>),
    Env(Arc<DebuggerCacheRef<Stack>>),
    Variable(Arc<DebuggerCacheRef<DebuggerVariable>>),
}

impl DebuggerCacheItem {
    pub fn get_ref_id(&self) -> i64 {
        match self {
            DebuggerCacheItem::Stack(stack) => stack.id,
            DebuggerCacheItem::Env(stack) => stack.id,
            DebuggerCacheItem::Variable(var_ref) => var_ref.id,
        }
    }

    pub fn to_dap_variable(&self) -> dap::types::Variable {
        match self {
            DebuggerCacheItem::Stack(_) => {
                unreachable!("Stack should not be converted to dap variable")
            }
            DebuggerCacheItem::Env(_) => {
                unreachable!("Env should not be converted to dap variable")
            }
            DebuggerCacheItem::Variable(var_ref) => {
                let var = &var_ref.item.var;
                let mut ref_id = 0;
                let mut value = var.value.clone();
                match var.value_type {
                    ValueType::TSTRING => {
                        value = format!("\"{}\"", value);
                    }
                    ValueType::TTABLE | ValueType::TUSERDATA | ValueType::GROUP => {
                        ref_id = var_ref.id;
                    }
                    _ => {}
                }

                let mut name = var.name.clone();
                match var.name_type {
                    ValueType::TSTRING => {}
                    _ => {
                        name = format!("[{}]", name);
                    }
                }

                dap::types::Variable {
                    name,
                    value,
                    variables_reference: ref_id,
                    ..Default::default()
                }
            }
        }
    }

    pub async fn compute_children(
        &self,
        cache: &mut DebuggerCache,
        debugger_conn: Arc<Mutex<DebuggerConnection>>,
    ) -> Vec<dap::types::Variable> {
        match self {
            DebuggerCacheItem::Stack(stack_ref) => {
                let mut variables = vec![];
                variables.extend(stack_ref.item.local_variables.iter());
                variables.extend(stack_ref.item.upvalue_variables.iter());

                let mut result_variables = vec![];
                for variable in variables {
                    let var_ref_id = cache.allocate_cache_id();
                    let var_ref = DebuggerCacheRef::new(
                        var_ref_id,
                        DebuggerVariable {
                            var: variable.clone(),
                            parent_ref_id: stack_ref.id,
                        },
                    );
                    let var_item = DebuggerCacheItem::Variable(Arc::new(var_ref));
                    result_variables.push(var_item.to_dap_variable());
                    cache.add_cache(var_item);
                }

                result_variables
            }
            DebuggerCacheItem::Env(_) => {
                vec![]
            }
            DebuggerCacheItem::Variable(var_ref) => {
                let mut children = var_ref.item.var.children.clone();
                if var_ref.item.var.value_type != ValueType::GROUP {
                    let mut debugger_conn = debugger_conn.lock().await;
                    let eval_rsp_result = debugger_conn
                        .eval_expr(
                            var_ref.item.get_expr(cache),
                            var_ref.item.var.cache_id as i64,
                            2,
                            -1,
                        )
                        .await;

                    match eval_rsp_result {
                        Ok(eval_rsp) => {
                            if eval_rsp.success {
                                children = eval_rsp.value.unwrap().children;
                            }
                        }
                        Err(err) => {
                            log::error!("Error evaluating expression: {}", err);
                            return vec![];
                        }
                    }
                }

                // todo compute children
                if let Some(children) = children {
                    let mut result_variables = vec![];
                    for child in children {
                        let child_ref_id = cache.allocate_cache_id();
                        let child_ref = DebuggerCacheRef::new(
                            child_ref_id,
                            DebuggerVariable {
                                var: child,
                                parent_ref_id: var_ref.id,
                            },
                        );
                        let child_item = DebuggerCacheItem::Variable(Arc::new(child_ref));
                        result_variables.push(child_item.to_dap_variable());
                        cache.add_cache(child_item);
                    }
                    result_variables
                } else {
                    vec![]
                }
            }
        }
    }
}
