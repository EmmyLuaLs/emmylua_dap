use dap::{requests::ScopesArguments, responses::ResponseBody};
use tokio_util::sync::CancellationToken;

use crate::context::{DapSnapShot, DebuggerCacheItem, DebuggerCacheRef};

use super::RequestResult;

pub async fn on_scopes_request(
    dap: DapSnapShot,
    scopes_arguments: ScopesArguments,
    _: CancellationToken,
) -> RequestResult {
    let mut data = dap.data.lock().await;
    data.current_frame_id = scopes_arguments.frame_id;
    let mut scopes = vec![];
    if let Some(stack) = data.stacks.get(scopes_arguments.frame_id as usize).cloned() {
        let ref_id = data.cache.allocate_cache_id();
        let stack_item = DebuggerCacheItem::Stack(DebuggerCacheRef::new(ref_id, stack.clone()).into());
        data.cache.add_cache(stack_item);
        let local_scope = dap::types::Scope {
            name: "Variables".to_string(),
            variables_reference: ref_id,
            expensive: false,
            presentation_hint: None,
            ..Default::default()
        };
        scopes.push(local_scope);
        
        let ref_id = data.cache.allocate_cache_id();
        let env_item = DebuggerCacheItem::Env(DebuggerCacheRef::new(ref_id, stack).into());
        data.cache.add_cache(env_item);
        let env_scope = dap::types::Scope {
            name: "Environment".to_string(),
            variables_reference: ref_id,
            expensive: false,
            presentation_hint: None,
            ..Default::default()
        };
        scopes.push(env_scope);
    } else {
        log::error!("Invalid frame id: {}", scopes_arguments.frame_id);
    }

    Ok(ResponseBody::Scopes(dap::responses::ScopesResponse {
        scopes,
    }))
}
