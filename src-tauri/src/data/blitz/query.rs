//! Blitz Datalake GraphQL 请求。
//!
//! 端点与查询名经本机实测确认（champion_id=5 免鉴权返回 augments/items/augment_trios）。
//! 参考工具 lolpro 无 License，此处不搬其代码，仅复现实测得到的端点事实。

use serde_json::{json, Value};

pub const BLITZ_DATALAKE_URL: &str = "https://datalake.v2.iesdev.com/graphql";

/// 命名 Databricks 查询：海克斯大乱斗按英雄聚合战绩。
const ARAM_MAYHEM_QUERY: &str = concat!(
    "query aramMayhemChampionStats($champion_id: String!) { ",
    "executeDatabricksQuery(game: LEAGUE queryName: \"prod_aram_mayhem_champion\" ",
    "params: [{ name: \"champion_id\", value: $champion_id }]) { payload } }"
);

/// 请求 Blitz Datalake，返回 `executeDatabricksQuery.payload` 原始 Value。
pub async fn fetch_mayhem_payload(
    client: &reqwest::Client,
    champion_id: &str,
) -> Result<Value, String> {
    let body = json!({
        "query": ARAM_MAYHEM_QUERY,
        "variables": { "champion_id": champion_id },
    });

    log::info!("🌐 请求 Blitz Datalake: champion_id={}", champion_id);
    let resp = client
        .post(BLITZ_DATALAKE_URL)
        .header("Content-Type", "application/json")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Blitz 网络请求失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Blitz 请求失败: HTTP {}", resp.status()));
    }

    let mut data: Value = resp
        .json()
        .await
        .map_err(|e| format!("Blitz JSON 解析失败: {}", e))?;

    if let Some(errors) = data.get("errors") {
        return Err(format!("Blitz GraphQL 错误: {}", errors));
    }

    data.pointer_mut("/data/executeDatabricksQuery/payload")
        .map(|v| v.take())
        .ok_or_else(|| "Blitz 响应缺 executeDatabricksQuery.payload".to_string())
}
