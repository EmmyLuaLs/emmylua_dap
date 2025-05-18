use dap::{
    requests::VariablesArguments,
    responses::{ResponseBody, VariablesResponse},
};
use tokio_util::sync::CancellationToken;

use crate::context::DapSnapShot;

use super::RequestResult;

pub async fn on_variable_request(
    dap: DapSnapShot,
    variable_argument: VariablesArguments,
    _: CancellationToken,
) -> RequestResult {
    let mut data = dap.data.lock().await;
    let cache_item = data.cache.get_cache(variable_argument.variables_reference);
    let cache = &mut data.cache;
    let variables = match cache_item {
        Some(item) => item.compute_children(cache, dap.debugger_conn).await,
        None => {
            vec![]
        }
    };

    Ok(ResponseBody::Variables(VariablesResponse { variables }))
}
