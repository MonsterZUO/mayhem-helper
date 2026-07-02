//! 海克斯大乱斗（ARAM: Mayhem）数据模型 + Tauri 命令。
//!
//! 组合 U1(Blitz 原始数据) + U2(海克斯元数据)，整形为前端可直接渲染的推荐：
//! 海克斯按胜率排序、核心出装、top 三连组合。数据驱动（只看 win_rate），不看英雄静态标签。

use crate::data::augments::{AugmentMetaStore, AugmentRarity};
use crate::data::{blitz, snapshot};
use serde::Serialize;
use serde_json::Value;
use std::sync::OnceLock;

/// 数据来源标识。当前仅外服 KR 代理（见 ADR-0001）。
const DATA_SOURCE: &str = "KR";
/// 快照兜底时的来源标识。
const DATA_SOURCE_SNAPSHOT: &str = "KR·出厂";
/// cdragon 元数据不可达时的空占位表。
static EMPTY_AUGMENT_STORE: OnceLock<AugmentMetaStore> = OnceLock::new();
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

/// 三连海克斯组合。Blitz 对 trio 只给档位(tier 1-5, 1=最优)，不给具体胜率。
#[derive(Debug, Clone, Serialize)]
pub struct AugmentTrio {
    pub ids: [u32; 3],
    pub names: [String; 3],
    /// 胜率档位，1=最优、5=最差。
    pub win_rate_tier: u64,
    /// 选取率档位，1=最热。
    pub pick_rate_tier: u64,
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

/// 兼容字符串或数字的 u64 取值（含浮点串/浮点数如 "1234.0" → 1234）。
fn as_u64(v: &Value) -> u64 {
    match v {
        Value::Number(n) => n
            .as_u64()
            .or_else(|| n.as_f64().map(|f| f as u64))
            .unwrap_or(0),
        Value::String(s) => s
            .parse::<u64>()
            .ok()
            .or_else(|| s.parse::<f64>().ok().map(|f| f as u64))
            .unwrap_or(0),
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

    // shape_trios 内部已按档位排好序（trio 无具体 win_rate，只有 tier）
    let mut trios = shape_trios(data.get("augment_trios"), store);
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
    let mut out: Vec<AugmentTrio> = map
        .iter()
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
                win_rate_tier: field_u64(stat, "win_rate_tier"),
                pick_rate_tier: field_u64(stat, "pick_rate_tier"),
                num_games: field_u64(stat, "num_games"),
            })
        })
        .collect();
    // 按胜率档位升序(1=最优)；缺失档位(0)视为最差沉底；同档按对局数降序(更可靠)
    out.sort_by(|a, b| {
        tier_key(a.win_rate_tier)
            .cmp(&tier_key(b.win_rate_tier))
            .then(b.num_games.cmp(&a.num_games))
    });
    out
}

/// 档位排序键：有效档位 1-5 原样，缺失(0)映射为最大值沉底。
fn tier_key(tier: u64) -> u64 {
    if tier == 0 {
        u64::MAX
    } else {
        tier
    }
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
/// 三层取数：实时 Blitz → 出厂快照（Blitz 不可达兜底，深审 F1/F5）。
#[tauri::command]
pub async fn get_mayhem_champion(champion_id: u32) -> Result<MayhemChampion, String> {
    let client = reqwest::Client::new();
    // 元数据 best-effort：cdragon 不可达时用空占位（海克斯显示为 id），不阻断
    let store = match augment_store(&client).await {
        Ok(store) => store,
        Err(e) => {
            log::warn!("海克斯元数据不可达，用占位: {}", e);
            EMPTY_AUGMENT_STORE.get_or_init(AugmentMetaStore::default)
        }
    };

    match blitz::fetch_mayhem_champion(&client, &champion_id.to_string()).await {
        Ok(raw) => Ok(shape_mayhem(
            &raw.data,
            &raw.patch,
            champion_id,
            store,
            TOP_TRIOS,
        )),
        Err(live_err) => match snapshot::snapshot_get(champion_id) {
            Some((data, patch)) => {
                log::warn!("Blitz 不可达，回退出厂快照: {}", live_err);
                let mut champ = shape_mayhem(&data, &patch, champion_id, store, TOP_TRIOS);
                champ.source = DATA_SOURCE_SNAPSHOT.to_string();
                Ok(champ)
            }
            None => Err(format!(
                "Blitz 不可达且无该英雄出厂快照: {}",
                live_err
            )),
        },
    }
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
    fn trios_topn_and_tier_monotonic() {
        let m = shape_mayhem(&blitz_data(), "16.13", 5, &store(), 8);
        assert!(!m.trios.is_empty());
        assert!(m.trios.len() <= 8);
        // 按 win_rate_tier 升序（1=最优）单调
        for w in m.trios.windows(2) {
            assert!(
                tier_key(w[0].win_rate_tier) <= tier_key(w[1].win_rate_tier),
                "三连未按档位升序"
            );
        }
        // top1 必是最优档(fixture 有 70 个 tier1)
        assert_eq!(m.trios[0].win_rate_tier, 1, "top1 应为最优档");
    }

    #[test]
    fn trios_sorted_by_tier_not_key_order() {
        // key 字典序 "1:1:1" < "9:9:9"，但让 9:9:9 档位更优(1)——验证按档位而非按 key 截断
        let data = serde_json::json!({
            "augment_trios": {
                "1:1:1": {"win_rate_tier":"5","pick_rate_tier":"3","num_games":"100"},
                "9:9:9": {"win_rate_tier":"1","pick_rate_tier":"2","num_games":"50"},
                "5:5:5": {"win_rate_tier":"3","pick_rate_tier":"3","num_games":"80"}
            }
        });
        let m = shape_mayhem(&data, "x", 5, &store(), 8);
        assert_eq!(m.trios[0].ids, [9, 9, 9], "应按档位取最优，非按 key 字典序");
        assert_eq!(m.trios[0].win_rate_tier, 1);
    }

    #[test]
    fn trios_same_tier_break_by_num_games() {
        let data = serde_json::json!({
            "augment_trios": {
                "1:1:1": {"win_rate_tier":"1","pick_rate_tier":"1","num_games":"50"},
                "2:2:2": {"win_rate_tier":"1","pick_rate_tier":"1","num_games":"200"}
            }
        });
        let m = shape_mayhem(&data, "x", 5, &store(), 8);
        assert_eq!(m.trios[0].ids, [2, 2, 2], "同档按对局数降序");
    }

    #[test]
    fn value_parsing_edges() {
        assert!((as_f64(&serde_json::json!("3.9E-4")) - 3.9e-4).abs() < 1e-12);
        assert_eq!(as_f64(&serde_json::json!("")), 0.0);
        assert_eq!(as_f64(&Value::Null), 0.0);
        assert_eq!(as_u64(&serde_json::json!("148729")), 148729);
        assert_eq!(as_u64(&serde_json::json!("1234.0")), 1234, "浮点串应转整数不清零");
        assert_eq!(as_u64(&serde_json::json!(1234.0)), 1234, "浮点数应转整数不清零");
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
