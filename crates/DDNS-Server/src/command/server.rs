use tracing::info;

pub fn generate_api_key(username: &str) {
    let api_key = crate::command::utils::generate_api_key();
    info!("Generated API Key for user '{}': {}", username, api_key);
}
