//! LCU 出装（item-set）写入 schema（**非** op.gg 形状的 `types.rs::ItemSet`）。
//!
//! LCU `PUT /lol-item-sets/v1/item-sets/{summonerId}/sets` body 为 wrapper：
//! `{ accountId, timestamp, itemSets:[{title, associatedChampions:[int], associatedMaps:[int], blocks}] }`。

use serde::Serialize;

/// 嚎哭深渊地图 id（海克斯大乱斗所在，非斗魂竞技场 CHERRY）。
pub const HOWLING_ABYSS_MAP_ID: i64 = 12;
/// 本工具生成的出装标题前缀，用于识别/替换自己写入的 set（不动用户自定义）。
pub const MAYHEM_SET_TITLE_PREFIX: &str = "[海克斯大乱斗]";

/// 单件装备。
#[derive(Debug, Clone, Serialize)]
pub struct ItemEntry {
    /// LCU 要求 item id 为字符串。
    pub id: String,
    pub count: u32,
}

/// 装备块（商店里一组）。
#[derive(Debug, Clone, Serialize)]
pub struct ItemBlockEntry {
    #[serde(rename = "type")]
    pub block_type: String,
    pub items: Vec<ItemEntry>,
}

/// 一份出装。
#[derive(Debug, Clone, Serialize)]
pub struct ItemSetEntry {
    pub title: String,
    #[serde(rename = "type")]
    pub set_type: String,
    pub map: String,
    pub mode: String,
    #[serde(rename = "associatedChampions")]
    pub associated_champions: Vec<i64>,
    #[serde(rename = "associatedMaps")]
    pub associated_maps: Vec<i64>,
    pub blocks: Vec<ItemBlockEntry>,
}
