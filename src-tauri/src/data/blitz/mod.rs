//! Blitz Datalake：海克斯大乱斗（ARAM: Mayhem）外服(KR)战绩数据源。
//!
//! U1 输出原始数据 blob（augments/items/augment_trios），U3 据此整形排序。

mod decode;
mod query;

pub use decode::decode_databricks_rows;

use serde_json::Value;

/// 某英雄的原始海克斯大乱斗数据 + 数据版本。
pub struct MayhemRaw {
    /// 含 `augments` / `items` / `augment_trios` 的对象。
    pub data: Value,
    /// 数据对应的版本号（如 "16.13"），供 UI 标注来源/版本。
    pub patch: String,
}

/// 拉取并解析某英雄的海克斯大乱斗数据。
pub async fn fetch_mayhem_champion(
    client: &reqwest::Client,
    champion_id: &str,
) -> Result<MayhemRaw, String> {
    let payload = query::fetch_mayhem_payload(client, champion_id).await?;
    let rows = decode_databricks_rows(&payload)?;

    let row = rows
        .into_iter()
        .next()
        .ok_or_else(|| format!("Blitz 无 champion_id={} 的海克斯大乱斗数据", champion_id))?;

    let data = row.get("data").cloned().unwrap_or(Value::Null);
    if !data.is_object() {
        return Err("Blitz data 列解析异常：非对象（疑似响应结构变更）".to_string());
    }
    let patch = row
        .get("patch")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(MayhemRaw { data, patch })
}
