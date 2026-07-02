//! 海克斯（Augment）元数据：id → 中文名 / 图标 URL / 稀有度。
//!
//! 源 = CommunityDragon `cherry-augments.json`（全集查表，含海克斯大乱斗的 `ARAM_` 海克斯）。
//! 实测：Blitz 返回的 augment id 100% 命中该文件全集。

mod metadata;

pub use metadata::{AugmentMeta, AugmentRarity, AugmentMetaStore};
