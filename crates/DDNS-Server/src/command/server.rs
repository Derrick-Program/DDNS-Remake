use crate::command::AppState;
use crate::command::CommandResult;
use anyhow::Result;
use comfy_table::ContentArrangement;
use comfy_table::Table;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use std::sync::Arc;
use tracing::{debug, error, info};
pub fn generate_api_key(username: &str, ctx: &Arc<AppState>) -> Result<CommandResult> {
    let api_key = crate::command::utils::generate_api_key();
    let mut db = ctx.db_service.clone();
    let Some(user) = db.find_user_by_username(username)? else {
        error!("User not found");
        return Ok(CommandResult::Continue);
    };
    debug!("{:#?}", user);
    info!("Generated API Key for user '{}': {}", username, api_key);
    Ok(CommandResult::Continue)
}

pub fn list_users(ctx: &Arc<AppState>) -> Result<CommandResult> {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(40)
        .set_header(vec!["UserName"]);
    let mut db = ctx.db_service.clone();
    let all_users = db.get_all_users()?;
    for user in &all_users {
        table.add_row(vec![user.to_string()]);
    }
    info!("顯示所有使用者表格\n{table}");
    Ok(CommandResult::Continue)
}

pub fn add_user(username: &str, password: &str, ctx: &Arc<AppState>) -> Result<CommandResult> {
    {
        let mut db = ctx.db_service.clone();
        if db.find_user_by_username(username)?.is_some() {
            error!("User already exists!");
            return Ok(CommandResult::Continue);
        };
    }
    let p_hash = crate::command::utils::hash_token(password);
    let mut db = ctx.db_service.clone();
    let new_user = db.create_user(username, &p_hash)?;
    info!("User {} added", new_user.username);
    Ok(CommandResult::Continue)
}
pub fn remove_user(username: &str, ctx: &Arc<AppState>) -> Result<CommandResult> {
    let mut db = ctx.db_service.clone();
    let Some(user) = db.find_user_by_username(username)? else {
        error!("User not found");
        return Ok(CommandResult::Continue);
    };
    let n = db.delete_user_by_username(&user.username)?;
    if n == 1 {
        info!("User {username} deleted");
    }
    Ok(CommandResult::Continue)
}

pub fn list_device(ctx: &Arc<AppState>) -> Result<CommandResult> {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(40)
        .set_header(vec!["DeviceName"]);
    let mut db = ctx.db_service.clone();
    let all_devices = db.get_all_devices()?;
    for device in &all_devices {
        table.add_row(vec![device.to_string()]);
    }
    info!("顯示所有裝置表格\n{table}");
    Ok(CommandResult::Continue)
}

pub fn add_device(
    device_name: &str,
    owner_name: &str,
    ctx: &Arc<AppState>,
) -> Result<CommandResult> {
    Ok(CommandResult::Continue)
}
