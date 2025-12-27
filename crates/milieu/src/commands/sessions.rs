use crate::api::ApiClient;
use crate::auth;
use crate::config::Config;
use crate::error::Result;
use crate::style;

pub async fn list(profile: &str) -> Result<()> {
    crate::commands::print_scope_user(profile);
    let config = Config::load()?;
    let base_url = config.base_url_for(profile)?;

    let token = auth::load_auth_token(profile)?;
    let client = ApiClient::new(&base_url, Some(token))?;

    let sessions = client.get_sessions().await?;
    if sessions.is_empty() {
        println!("{}", style::paint(style::YELLOW, "No active sessions."));
        return Ok(());
    }

    let mut host_width = "Host".len();
    let mut token_width = "Token".len();
    let mut status_width = "Status".len();
    for session in &sessions {
        host_width = host_width.max(session.host.len());
        token_width = token_width.max(session.token_suffix.len() + 1);
        let status = if session.active { "active" } else { "expired" };
        status_width = status_width.max(status.len());
    }

    println!(
        "{}  {}  {}  {}  {}",
        style::bold(style::MAUVE, &format!("{:<width$}", "Host", width = host_width)),
        style::bold(style::MAUVE, &format!("{:<width$}", "Token", width = token_width)),
        style::bold(style::MAUVE, &format!("{:<width$}", "Status", width = status_width)),
        style::bold(style::MAUVE, "Created"),
        style::bold(style::MAUVE, "Expires")
    );
    println!(
        "{}  {}  {}  {}  {}",
        style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = host_width)),
        style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = token_width)),
        style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = status_width)),
        style::paint(style::SUBTEXT1, "-------"),
        style::paint(style::SUBTEXT1, "-------")
    );

    for session in sessions {
        let token = format!("…{}", session.token_suffix);
        let status = if session.active { "active" } else { "expired" };
        let status_color = if session.active { style::GREEN } else { style::PEACH };
        println!(
            "{}  {}  {}  {}  {}",
            style::paint(style::TEXT, &format!("{:<width$}", session.host, width = host_width)),
            style::paint(style::TEXT, &format!("{:<width$}", token, width = token_width)),
            style::paint(status_color, &format!("{:<width$}", status, width = status_width)),
            style::paint(style::SUBTEXT1, &session.created_at),
            style::paint(style::SUBTEXT1, &session.expires_at)
        );
    }

    Ok(())
}
