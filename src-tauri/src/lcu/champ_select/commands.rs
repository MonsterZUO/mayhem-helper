use crate::{http_client, lcu};
use std::collections::HashMap;

#[tauri::command]
pub async fn get_champselect_team_players_info(
) -> Result<HashMap<String, lcu::types::MatchStatistics>, String> {
    let client = http_client::get_lcu_client();
    lcu::champ_select::service::get_champselect_team_players_info(client).await
}

#[tauri::command]
pub async fn get_champ_select_session() -> Result<serde_json::Value, String> {
    let client = http_client::get_lcu_client();
    lcu::champ_select::service::get_champ_select_session_raw(client).await
}

#[tauri::command]
pub async fn get_champ_select_session_typed() -> Result<lcu::types::ChampSelectSession, String> {
    let client = http_client::get_lcu_client();
    lcu::champ_select::service::get_champ_select_session(client).await
}

/// 选英雄阶段取本地玩家当前英雄 id（供海克斯大乱斗推荐自动跟随）。
/// 未在选英雄/未分配时返回 Ok(None)。
#[tauri::command]
pub async fn get_current_champion_id() -> Result<Option<u32>, String> {
    let client = http_client::get_lcu_client();
    let session = lcu::champ_select::service::get_champ_select_session(client).await?;
    Ok(lcu::champ_select::service::local_player_champion_id(&session))
}

#[tauri::command]
pub async fn pick_champion(
    action_id: u64,
    champion_id: u64,
    completed: bool,
) -> Result<String, String> {
    let client = http_client::get_lcu_client();
    match lcu::champ_select::service::pick_champion(client, action_id, champion_id, completed).await
    {
        Ok(()) => {
            let action_type = if completed { "锁定" } else { "预选" };
            let message = format!(
                "{}英雄成功 (ActionID: {}, ChampionID: {})",
                action_type, action_id, champion_id
            );
            log::info!("[Commands] {}", message);
            Ok(message)
        }
        Err(e) => {
            let action_type = if completed { "锁定" } else { "预选" };
            let error_msg = format!("{}英雄失败: {}", action_type, e);
            log::error!("[Commands] {}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn ban_champion(action_id: u64, champion_id: u64) -> Result<String, String> {
    let client = http_client::get_lcu_client();
    match lcu::champ_select::service::ban_champion(client, action_id, champion_id).await {
        Ok(()) => {
            let message = format!(
                "禁用英雄成功 (ActionID: {}, ChampionID: {})",
                action_id, champion_id
            );
            log::info!("[Commands] {}", message);
            Ok(message)
        }
        Err(e) => {
            let error_msg = format!("禁用英雄失败: {}", e);
            log::error!("[Commands] {}", error_msg);
            Err(error_msg)
        }
    }
}
