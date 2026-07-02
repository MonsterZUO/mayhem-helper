//! 海克斯大乱斗（ARAM: Mayhem）数据模型 + Tauri 命令。
//!
//! 组合 U1(Blitz 原始数据) + U2(海克斯元数据)，整形为前端可直接渲染的推荐：
//! 海克斯按胜率排序、核心出装、top 三连组合。数据驱动（只看 win_rate），不看英雄静态标签。

use crate::data::augments::{AugmentMetaStore, AugmentRarity};
use crate::data::blitz;
use serde::Serialize;
use serde_json::Value;

/// 数据来源标识。当前仅外服 KR 代理（见 ADR-0001）。
const DATA_SOURCE: &str = "KR";
/// 默认返回 top 三连组合数。
const TOP_TRIOS: usize = 8;

static AUGMENT_STORE: tokio::sync::OnceCell<AugmentMetaStore> = tokio::sync::OnceCell::const_new();

/// 单个海克斯的推荐条目。
#[derive(Debug, Clone, Serialize)]
pub struct RankedAugment {
    pub id: u32,
    pub name: String,
    pub icon_url: String,
    pub rarity: AugmentRarity,
    pub win_rate: f64,
    pub pick_rate: f64,
    pub num_games: u64,
}

/// 核心出装条目。
#[derive(Debug, Clone, Serialize)]
pub struct RankedItem {
    pub id: u32,
    pub win_rate: f64,
    pub pick_rate: f64,
    pub num_games: u64,
}

/// 三连海克斯组合。
#[derive(Debug, Clone, Serialize)]
pub struct AugmentTrio {
    pub ids: [u32; 3],
    pub names: [String; 3],
    pub win_rate: f64,
    pub num_games: u64,
}

/// 某英雄的完整海克斯大乱斗推荐。
#[derive(Debug, Clone, Serialize)]
pub struct MayhemChampion {
    pub champion_id: u32,
    pub patch: String,
    pub source: String,
    pub win_rate: f64,
    pub pick_rate: f64,
    /// 按 win_rate 降序，rarity 字段供前端分组。
    pub augments: Vec<RankedAugment>,
    pub core_items: Vec<RankedItem>,
    pub trios: Vec<AugmentTrio>,
}

/// 兼容字符串或数字的 f64 取值（Blitz 数值多为字符串 "0.5"）。
fn as_f64(v: &Value) -> f64 {
    match v {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => s.parse().unwrap_or(0.0),
        _ => 0.0,
    }
}

/// 兼容字符串或数字的 u64 取值。
fn as_u64(v: &Value) -> u64 {
    match v {
        Value::Number(n) => n.as_u64().unwrap_or(0),
        Value::String(s) => s.parse().unwrap_or(0),
        _ => 0,
    }
}

fn field_f64(obj: &Value, key: &str) -> f64 {
    obj.get(key).map(as_f64).unwrap_or(0.0)
}

fn field_u64(obj: &Value, key: &str) -> u64 {
    obj.get(key).map(as_u64).unwrap_or(0)
}

/// 纯整形函数：Blitz data blob + 元数据 → MayhemChampion。可单测。
pub fn shape_mayhem(
    data: &Value,
    patch: &str,
    champion_id: u32,
    store: &AugmentMetaStore,
    top_trios: usize,
) -> MayhemChampion {
    let mut augments = shape_augments(data.get("augments"), store);
    augments.sort_by(|a, b| b.win_rate.total_cmp(&a.win_rate));

    let mut core_items = shape_items(data.get("items"));
    core_items.sort_by(|a, b| b.win_rate.total_cmp(&a.win_rate));

    let mut trios = shape_trios(data.get("augment_trios"), store);
    trios.sort_by(|a, b| b.win_rate.total_cmp(&a.win_rate));
    trios.truncate(top_trios);

    MayhemChampion {
        champion_id,
        patch: patch.to_string(),
        source: DATA_SOURCE.to_string(),
        win_rate: field_f64(data, "win_rate"),
        pick_rate: field_f64(data, "pick_rate"),
        augments,
        core_items,
        trios,
    }
}

fn shape_augments(augments: Option<&Value>, store: &AugmentMetaStore) -> Vec<RankedAugment> {
    let Some(map) = augments.and_then(|v| v.as_object()) else {
        return Vec::new();
    };
    map.iter()
        .filter_map(|(id_str, stat)| {
            let id = id_str.parse::<u32>().ok()?;
            let meta = store.resolve(id);
            Some(RankedAugment {
                id,
                name: meta.name,
                icon_url: meta.icon_url,
                rarity: meta.rarity,
                win_rate: field_f64(stat, "win_rate"),
                pick_rate: field_f64(stat, "pick_rate"),
                num_games: field_u64(stat, "num_games"),
            })
        })
        .collect()
}

fn shape_items(items: Option<&Value>) -> Vec<RankedItem> {
    let Some(map) = items.and_then(|v| v.as_object()) else {
        return Vec::new();
    };
    map.iter()
        .filter_map(|(id_str, stat)| {
            let id = id_str.parse::<u32>().ok()?;
            Some(RankedItem {
                id,
                win_rate: field_f64(stat, "win_rate"),
                pick_rate: field_f64(stat, "pick_rate"),
                num_games: field_u64(stat, "num_games"),
            })
        })
        .collect()
}

fn shape_trios(trios: Option<&Value>, store: &AugmentMetaStore) -> Vec<AugmentTrio> {
    let Some(map) = trios.and_then(|v| v.as_object()) else {
        return Vec::new();
    };
    map.iter()
        .filter_map(|(key, stat)| {
            let ids = parse_trio_key(key)?;
            let names = [
                store.resolve(ids[0]).name,
                store.resolve(ids[1]).name,
                store.resolve(ids[2]).name,
            ];
            Some(AugmentTrio {
                ids,
                names,
                win_rate: field_f64(stat, "win_rate"),
                num_games: field_u64(stat, "num_games"),
            })
        })
        .collect()
}

/// 解析 "1020:1138:2102" → [1020,1138,2102]。
fn parse_trio_key(key: &str) -> Option<[u32; 3]> {
    let mut parts = key.split(':');
    let a = parts.next()?.parse().ok()?;
    let b = parts.next()?.parse().ok()?;
    let c = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some([a, b, c])
}

/// 获取（缓存）海克斯元数据表。
async fn augment_store(client: &reqwest::Client) -> Result<&'static AugmentMetaStore, String> {
    // ponytail: 进程级缓存，cdragon 数据版本内稳定
    AUGMENT_STORE
        .get_or_try_init(|| AugmentMetaStore::fetch(client))
        .await
}

/// Tauri 命令：取某英雄的海克斯大乱斗推荐。
#[tauri::command]
pub async fn get_mayhem_champion(champion_id: u32) -> Result<MayhemChampion, String> {
    let client = reqwest::Client::new();
    let raw = blitz::fetch_mayhem_champion(&client, &champion_id.to_string()).await?;
    let store = augment_store(&client).await?;
    Ok(shape_mayhem(
        &raw.data,
        &raw.patch,
        champion_id,
        store,
        TOP_TRIOS,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> AugmentMetaStore {
        AugmentMetaStore::from_cdragon_json(include_str!(
            "../../data/augments/fixtures/cherry-augments-zh_cn.json"
        ))
        .unwrap()
    }

    fn blitz_data() -> Value {
        let raw: Value = serde_json::from_str(include_str!(
            "../../data/blitz/fixtures/mayhem_champion_5.json"
        ))
        .unwrap();
        let data_str = raw["data"]["executeDatabricksQuery"]["payload"]["result"]["dataArray"][0]
            [1]
            .as_str()
            .unwrap();
        serde_json::from_str(data_str).unwrap()
    }

    #[test]
    fn augments_sorted_desc_by_win_rate() {
        let m = shape_mayhem(&blitz_data(), "16.13", 5, &store(), 8);
        assert!(!m.augments.is_empty());
        for w in m.augments.windows(2) {
            assert!(w[0].win_rate >= w[1].win_rate, "海克斯未按胜率降序");
        }
        // 数据驱动：top 海克斯带真实元数据
        assert!(!m.augments[0].name.is_empty());
    }

    #[test]
    fn trios_parsed_and_topn() {
        let m = shape_mayhem(&blitz_data(), "16.13", 5, &store(), 8);
        assert!(!m.trios.is_empty());
        assert!(m.trios.len() <= 8);
        for w in m.trios.windows(2) {
            assert!(w[0].win_rate >= w[1].win_rate, "三连未按胜率降序");
        }
        // trio key "a:b:c" 解析为 3 个 id
        assert_eq!(m.trios[0].ids.len(), 3);
    }

    #[test]
    fn parses_string_numbers() {
        // Blitz win_rate 是字符串 "0.5"
        let m = shape_mayhem(&blitz_data(), "16.13", 5, &store(), 8);
        assert!(m.win_rate > 0.0 && m.win_rate < 1.0);
    }

    #[test]
    fn trio_key_parsing() {
        assert_eq!(parse_trio_key("1:2:3"), Some([1, 2, 3]));
        assert_eq!(parse_trio_key("1:2"), None);
        assert_eq!(parse_trio_key("1:2:3:4"), None);
        assert_eq!(parse_trio_key("x:2:3"), None);
    }

    #[test]
    fn empty_sections_dont_panic() {
        let data = serde_json::json!({ "win_rate": "0.5" });
        let m = shape_mayhem(&data, "16.13", 5, &store(), 8);
        assert!(m.augments.is_empty() && m.core_items.is_empty() && m.trios.is_empty());
    }
}
