use crate::api::ApiClient;
use crate::auth;
use crate::config::Config;
use crate::error::Result;
use crate::style;

pub async fn run(profile: &str) -> Result<()> {
    crate::commands::print_scope_user();
    if let Ok(config) = Config::load() {
        if let Ok(base_url) = config.base_url_for(profile) {
            if let Ok(token) = auth::load_auth_token(profile) {
                if let Ok(client) = ApiClient::new(base_url, Some(token)) {
                    let _ = client.logout().await;
                }
            }
        }
    }
    let _ = auth::clear_umk(profile);
    let _ = auth::clear_auth(profile);
    let _ = auth::delete_session(profile);
    println!(
        "{}",
        style::paint(style::PEACH, &format!("Logged out ({})", profile))
    );
    Ok(())
}
