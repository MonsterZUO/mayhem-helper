//! 出装写入：构建本工具的 item-set + 合并回写（不覆盖用户自定义）。

use super::types::{
    ItemBlockEntry, ItemEntry, ItemSetEntry, HOWLING_ABYSS_MAP_ID, MAYHEM_SET_TITLE_PREFIX,
};
use crate::lcu::request::{lcu_get, lcu_put_no_content};
use crate::lcu::types::SummonerInfo;
use reqwest::Client;
use serde_json::{json, Value};

/// 构建本英雄的海克斯大乱斗出装（纯函数）。关键：`associatedMaps:[12]`(嚎哭深渊)。
pub fn build_mayhem_item_set(champion_id: u32, item_ids: &[u32]) -> ItemSetEntry {
    let items = item_ids
        .iter()
        .map(|id| ItemEntry {
            id: id.to_string(),
            count: 1,
        })
        .collect();
    ItemSetEntry {
        title: format!("{} {}", MAYHEM_SET_TITLE_PREFIX, champion_id),
        set_type: "custom".to_string(),
        map: "any".to_string(),
        mode: "any".to_string(),
        associated_champions: vec![champion_id as i64],
        associated_maps: vec![HOWLING_ABYSS_MAP_ID],
        blocks: vec![ItemBlockEntry {
            block_type: "海克斯大乱斗核心出装".to_string(),
            items,
        }],
    }
}

/// 合并：保留用户既有 sets、替换本工具旧 set、追加新 set（纯函数）。
/// 用 Value 操作，避免臆测/丢弃用户 set 的未知字段。
pub fn merge_item_sets(existing: &Value, our_set: Value, account_id: u64, timestamp: u64) -> Value {
    let mut sets: Vec<Value> = existing
        .get("itemSets")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // 去掉本工具上次写入的 set（按标题前缀识别），保留其余（用户自定义）
    sets.retain(|s| {
        !s.get("title")
            .and_then(|t| t.as_str())
            .map(|t| t.starts_with(MAYHEM_SET_TITLE_PREFIX))
            .unwrap_or(false)
    });
    sets.push(our_set);

    json!({ "accountId": account_id, "timestamp": timestamp, "itemSets": sets })
}

/// 运行时：GET 现有 sets → 合并 → PUT 回写。需活体客户端（Windows 验）。
pub async fn apply_mayhem_item_set(
    client: &Client,
    summoner: &SummonerInfo,
    champion_id: u32,
    item_ids: &[u32],
    timestamp: u64,
) -> Result<(), String> {
    let account_id: u64 = summoner
        .account_id
        .parse()
        .map_err(|_| format!("accountId 非数字: {}", summoner.account_id))?;
    let path = format!(
        "/lol-item-sets/v1/item-sets/{}/sets",
        summoner.summoner_id
    );

    // 首次可能无 sets：读失败/空时按空处理，不阻断
    let existing: Value = lcu_get(client, &path)
        .await
        .unwrap_or_else(|_| json!({ "itemSets": [] }));

    let our_set = serde_json::to_value(build_mayhem_item_set(champion_id, item_ids))
        .map_err(|e| format!("序列化 item-set 失败: {}", e))?;
    let body = merge_item_sets(&existing, our_set, account_id, timestamp);

    // LCU 该端点写入成功常返回空 body，用 no_content 避免 EOF 假失败（ce-code-review）
    lcu_put_no_content(client, &path, body).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_set_with_howling_abyss_map() {
        let set = build_mayhem_item_set(5, &[3153, 3157]);
        assert_eq!(set.associated_champions, vec![5]);
        assert_eq!(set.associated_maps, vec![12], "应关联嚎哭深渊(12)非竞技场");
        assert!(set.title.starts_with(MAYHEM_SET_TITLE_PREFIX));
        assert_eq!(set.blocks[0].items.len(), 2);
        assert_eq!(set.blocks[0].items[0].id, "3153", "item id 为字符串");
    }

    #[test]
    fn merge_preserves_user_sets_and_replaces_own() {
        let existing = json!({
            "itemSets": [
                { "title": "我的自定义出装", "blocks": [] },
                { "title": "[海克斯大乱斗] 5", "blocks": [] }
            ]
        });
        let our = serde_json::to_value(build_mayhem_item_set(5, &[3153])).unwrap();
        let merged = merge_item_sets(&existing, our, 12345, 0);
        let sets = merged["itemSets"].as_array().unwrap();
        // 用户 set 保留、本工具旧 set 被替换(仍只一个)、共 2 个
        assert_eq!(sets.len(), 2);
        assert!(sets.iter().any(|s| s["title"] == "我的自定义出装"));
        let ours: Vec<_> = sets
            .iter()
            .filter(|s| {
                s["title"]
                    .as_str()
                    .unwrap_or("")
                    .starts_with(MAYHEM_SET_TITLE_PREFIX)
            })
            .collect();
        assert_eq!(ours.len(), 1, "本工具 set 不重复");
        assert_eq!(merged["accountId"], 12345);
    }

    #[test]
    fn merge_into_empty() {
        let merged = merge_item_sets(&json!({ "itemSets": [] }), json!({"title":"[海克斯大乱斗] 1"}), 1, 0);
        assert_eq!(merged["itemSets"].as_array().unwrap().len(), 1);
    }
}
