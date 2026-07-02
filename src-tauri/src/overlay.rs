//! 游戏内浮层窗口管理：显隐切换、点击穿透。
//!
//! 浮层是 OS 置顶窗（非注入），需游戏无边框模式方可覆盖显示（见 ADR-0002）。
//! 默认点击穿透，不拦截游戏内点击（深审 F4）。

use tauri::{AppHandle, Manager, WebviewWindow};

pub const OVERLAY_LABEL: &str = "overlay";

fn overlay_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(OVERLAY_LABEL)
}

/// 切换浮层显隐。显示时不抢焦点，避免无边框游戏最小化（F4）。
pub fn toggle_overlay(app: &AppHandle) {
    let Some(window) = overlay_window(app) else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        let _ = window.show();
        let _ = window.set_always_on_top(true);
    }
}

/// 前端手动切换浮层显隐。
#[tauri::command]
pub fn toggle_overlay_cmd(app: AppHandle) {
    toggle_overlay(&app);
}

/// 设置浮层点击穿透。默认 true（不拦截游戏点击）；hover 交互区时前端临时设 false。
#[tauri::command]
pub fn set_overlay_click_through(app: AppHandle, ignore: bool) -> Result<(), String> {
    if let Some(window) = overlay_window(&app) {
        window
            .set_ignore_cursor_events(ignore)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
