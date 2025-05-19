use dap::{
    requests::EvaluateArguments,
    responses::{EvaluateResponse, ResponseBody},
};
use tokio_util::sync::CancellationToken;

use crate::context::{DapSnapShot, DebuggerCacheItem, DebuggerCacheRef, DebuggerVariable};

use super::RequestResult;

pub async fn on_evaluate_request(
    dap: DapSnapShot,
    evaluate_arguments: EvaluateArguments,
    _: CancellationToken,
) -> RequestResult {
    let frame_id = evaluate_arguments.frame_id.unwrap_or(-1);
    let expression = evaluate_arguments.expression;

    let mut debugger_conn = dap.debugger_conn.lock().await;
    let eval_rsp = debugger_conn.eval_expr(expression, 0, 1, frame_id).await?;

    if eval_rsp.success {
        let mut data = dap.data.lock().await;
        let value = eval_rsp.value;
        let ref_id = data.cache.allocate_cache_id();
        let variable_item = DebuggerCacheItem::Variable(
            DebuggerCacheRef::new(
                ref_id,
                DebuggerVariable {
                    var: value,
                    parent_ref_id: 0,
                },
            )
            .into(),
        );
        let variable = variable_item.to_dap_variable();
        data.cache.add_cache(variable_item);

        Ok(ResponseBody::Evaluate(EvaluateResponse {
            result: variable.value,
            type_field: variable.type_field,
            variables_reference: ref_id,
            ..Default::default()
        }))
    } else {
        Ok(ResponseBody::Evaluate(EvaluateResponse {
            result: eval_rsp.error,
            type_field: Some("string".to_string()),
            variables_reference: 0,
            ..Default::default()
        }))
    }
}
