//! 出厂数据快照：Blitz 不可达/冷启动时的兜底（深审 F1/F5）。
//!
//! 打包进应用的 `resources/mayhem-snapshot.json`，格式：
//! `{ "patch": "16.13", "source": "KR", "champions": { "5": <data blob>, ... } }`
//! data blob 结构与 Blitz 返回一致（augments/items/augment_trios）。

use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;

/// 进程级快照。setup 时从打包资源载入；未载入/缺失时为空。
static SNAPSHOT: OnceLock<SnapshotStore> = OnceLock::new();

#[derive(Debug, Default)]
pub struct SnapshotStore {
    by_champion: HashMap<u32, Value>,
    patch: String,
}

impl SnapshotStore {
    pub fn from_json(json: &str) -> Result<Self, String> {
        let root: Value =
            serde_json::from_str(json).map_err(|e| format!("snapshot 解析失败: {}", e))?;
        let patch = root
            .get("patch")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let mut by_champion = HashMap::new();
        if let Some(map) = root.get("champions").and_then(|v| v.as_object()) {
            for (key, blob) in map {
                if let Ok(id) = key.parse::<u32>() {
                    by_champion.insert(id, blob.clone());
                }
            }
        }
        Ok(Self { by_champion, patch })
    }

    /// 取某英雄的 data blob + 版本。
    pub fn get(&self, champion_id: u32) -> Option<(Value, String)> {
        self.by_champion
            .get(&champion_id)
            .map(|blob| (blob.clone(), self.patch.clone()))
    }

    pub fn len(&self) -> usize {
        self.by_champion.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_champion.is_empty()
    }
}

/// setup 时载入快照（best-effort，失败仅告警不阻断）。
pub fn init_snapshot(json: &str) {
    match SnapshotStore::from_json(json) {
        Ok(store) => {
            log::info!("✅ 载入出厂快照: {} 个英雄", store.len());
            let _ = SNAPSHOT.set(store);
        }
        Err(e) => log::warn!("出厂快照载入失败: {}", e),
    }
}

/// 从快照取某英雄数据（Blitz 不可达时的兜底）。
pub fn snapshot_get(champion_id: u32) -> Option<(Value, String)> {
    SNAPSHOT.get().and_then(|store| store.get(champion_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> SnapshotStore {
        SnapshotStore::from_json(include_str!("../../resources/mayhem-snapshot.json")).unwrap()
    }

    #[test]
    fn loads_and_gets_champion() {
        let store = sample();
        assert!(store.len() >= 1);
        let (blob, patch) = store.get(5).expect("样本含德邦(5)");
        assert!(blob.get("augments").is_some());
        assert!(!patch.is_empty());
    }

    #[test]
    fn missing_champion_none() {
        assert!(sample().get(99999).is_none());
    }

    #[test]
    fn malformed_is_error() {
        assert!(SnapshotStore::from_json("{ not json").is_err());
    }
}
