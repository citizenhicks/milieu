use crate::api::ApiClient;
use crate::auth;
use crate::commands::prompt;
use crate::config::Config;
use crate::error::Result;
use crate::style;
use atty::Stream;

pub async fn run(profile: &str) -> Result<()> {
    crate::commands::print_scope_user(profile);
    let mut keep_local = false;
    if atty::is(Stream::Stdin) {
        let answer = prompt("Keep login details in keychain? [y/N] ")?;
        let answer = answer.trim().to_lowercase();
        keep_local = answer == "y" || answer == "yes";
    }
    if let Ok(config) = Config::load() {
        if let Ok(base_url) = config.base_url_for(profile) {
            if let Ok(token) = auth::load_auth_token(profile) {
                if let Ok(client) = ApiClient::new(&base_url, Some(token)) {
                    let _ = client.logout().await;
                }
            }
        }
    }
    if keep_local {
        let _ = auth::clear_auth(profile);
        println!(
            "{}",
            style::paint(
                style::PEACH,
                "Logged out. Kept recovery phrase + local keychain data."
            )
        );
        println!(
            "{}",
            style::paint(
                style::SUBTEXT1,
                "Next login only needs email + password. Remove it later manually or `milieu logout` and answer no."
            )
        );
    } else {
        let _ = auth::clear_umk(profile);
        let _ = auth::clear_auth(profile);
        let _ = auth::delete_session(profile);
        println!(
            "{}",
            style::paint(style::PEACH, &format!("Logged out ({})", profile))
        );
    }
    Ok(())
}
