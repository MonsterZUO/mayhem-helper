//! 从 CommunityDragon cherry-augments 构建 augment id→元数据表。

use serde::Serialize;
use std::collections::HashMap;

const CDRAGON_ZH_CN_URL: &str = "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/zh_cn/v1/cherry-augments.json";
const CDRAGON_ASSET_BASE: &str =
    "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default";

/// 海克斯稀有度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AugmentRarity {
    Silver,
    Gold,
    Prismatic,
    Unknown,
}

impl AugmentRarity {
    /// cherry-augments 的 `rarity` 字段（kSilver/kGold/kPrismatic）→ 枚举。
    fn from_cdragon(raw: &str) -> Self {
        match raw {
            "kSilver" => Self::Silver,
            "kGold" => Self::Gold,
            "kPrismatic" => Self::Prismatic,
            _ => Self::Unknown,
        }
    }
}

/// 单个海克斯的展示元数据。
#[derive(Debug, Clone, Serialize)]
pub struct AugmentMeta {
    pub id: u32,
    pub name: String,
    pub icon_url: String,
    pub rarity: AugmentRarity,
}

/// id→元数据表。未知 id 返回占位，不报错。
#[derive(Debug, Clone, Default)]
pub struct AugmentMetaStore {
    by_id: HashMap<u32, AugmentMeta>,
}

impl AugmentMetaStore {
    /// 从 cherry-augments JSON 文本构建。
    pub fn from_cdragon_json(json: &str) -> Result<Self, String> {
        let list: Vec<serde_json::Value> =
            serde_json::from_str(json).map_err(|e| format!("cherry-augments 解析失败: {}", e))?;

        let mut by_id = HashMap::with_capacity(list.len());
        for entry in &list {
            let Some(id) = entry.get("id").and_then(|v| v.as_u64()) else {
                continue;
            };
            let id = id as u32;
            let name = entry
                .get("nameTRA")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let icon_path = entry
                .get("augmentSmallIconPath")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let rarity = entry
                .get("rarity")
                .and_then(|v| v.as_str())
                .map(AugmentRarity::from_cdragon)
                .unwrap_or(AugmentRarity::Unknown);

            by_id.insert(
                id,
                AugmentMeta {
                    id,
                    name,
                    icon_url: icon_url_from_path(icon_path),
                    rarity,
                },
            );
        }
        Ok(Self { by_id })
    }

    /// 在线拉取 zh_cn cherry-augments 并构建。
    pub async fn fetch(client: &reqwest::Client) -> Result<Self, String> {
        let resp = client
            .get(CDRAGON_ZH_CN_URL)
            .send()
            .await
            .map_err(|e| format!("cherry-augments 请求失败: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("cherry-augments HTTP {}", resp.status()));
        }
        let text = resp
            .text()
            .await
            .map_err(|e| format!("cherry-augments 读取失败: {}", e))?;
        Self::from_cdragon_json(&text)
    }

    /// 解析 id；未知 id 返回占位（名=id 字符串、无图标、稀有度 Unknown）。
    pub fn resolve(&self, id: u32) -> AugmentMeta {
        self.by_id.get(&id).cloned().unwrap_or_else(|| AugmentMeta {
            id,
            name: id.to_string(),
            icon_url: String::new(),
            rarity: AugmentRarity::Unknown,
        })
    }

    pub fn contains(&self, id: u32) -> bool {
        self.by_id.contains_key(&id)
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}

/// `augmentSmallIconPath` → CommunityDragon 图标 URL。
/// 实测变换：小写、`/lol-game-data/` 换 cdragon base、塌缩 `/assets/assets/`→`/assets/`。
fn icon_url_from_path(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    let lower = path.to_lowercase();
    let rel = lower.strip_prefix("/lol-game-data/").unwrap_or(&lower);
    format!("{}/{}", CDRAGON_ASSET_BASE, rel).replace("/assets/assets/", "/assets/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> AugmentMetaStore {
        AugmentMetaStore::from_cdragon_json(include_str!(
            "fixtures/cherry-augments-zh_cn.json"
        ))
        .unwrap()
    }

    #[test]
    fn resolves_known_augment() {
        let meta = store().resolve(2095);
        assert_eq!(meta.name, "掷骰狂人");
        assert_eq!(meta.rarity, AugmentRarity::Prismatic);
        assert!(
            meta.icon_url
                .ends_with("/assets/ux/kiwi/augments/icons/highroller_small.png"),
            "icon URL 变换错: {}",
            meta.icon_url
        );
        assert!(!meta.icon_url.contains("/assets/assets/"), "未塌缩双 assets");
    }

    #[test]
    fn unknown_id_returns_placeholder() {
        let meta = store().resolve(99999999);
        assert_eq!(meta.name, "99999999");
        assert_eq!(meta.rarity, AugmentRarity::Unknown);
        assert!(meta.icon_url.is_empty());
    }

    /// Covers F3：Blitz 返回的全部 augment id 必须命中 cherry-augments 全集，
    /// 否则会静默映射错位（审查发现的隐蔽真问题）。
    #[test]
    fn blitz_augment_ids_fully_covered() {
        let store = store();
        let blitz: serde_json::Value = serde_json::from_str(include_str!(
            "../blitz/fixtures/mayhem_champion_5.json"
        ))
        .unwrap();
        let data_str = blitz["data"]["executeDatabricksQuery"]["payload"]["result"]["dataArray"]
            [0][1]
            .as_str()
            .unwrap();
        let data: serde_json::Value = serde_json::from_str(data_str).unwrap();
        let augment_ids = data["augments"].as_object().unwrap();

        let missing: Vec<&String> = augment_ids
            .keys()
            .filter(|k| {
                k.parse::<u32>()
                    .map(|id| !store.contains(id))
                    .unwrap_or(true)
            })
            .collect();
        assert!(
            missing.is_empty(),
            "{} 个 Blitz augment id 未命中 cherry-augments: {:?}",
            missing.len(),
            &missing[..missing.len().min(10)]
        );
    }
}
