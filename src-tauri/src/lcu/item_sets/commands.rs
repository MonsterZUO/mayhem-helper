use crate::{http_client, lcu};
use std::time::{SystemTime, UNIX_EPOCH};

/// 把某英雄的海克斯大乱斗核心出装写入客户端预设出装（局内商店可见）。
/// item_ids 由前端从 `get_mayhem_champion` 的 core_items 传入。
#[tauri::command]
pub async fn apply_mayhem_item_set(champion_id: u32, item_ids: Vec<u32>) -> Result<String, String> {
    let client = http_client::get_lcu_client();
    let summoner = lcu::summoner::service::get_current_summoner(client).await?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    lcu::item_sets::service::apply_mayhem_item_set(
        client,
        &summoner,
        champion_id,
        &item_ids,
        timestamp,
    )
    .await?;
    Ok(format!("已导入 {} 件出装", item_ids.len()))
}
