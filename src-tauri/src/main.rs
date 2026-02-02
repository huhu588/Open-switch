// ai-switch Tauri 入口点
// 防止 Windows 上出现额外的控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(err) = ai_switch::run() {
        eprintln!("启动失败: {}", err);
    }
}
