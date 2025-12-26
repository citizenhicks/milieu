use crate::api::{ApiClient, RegisterRequest};
use crate::commands::{prompt, prompt_password};
use crate::config::Config;
use crate::error::Result;
use crate::style;

pub async fn run(profile: &str) -> Result<()> {
    crate::commands::print_scope_user();
    let config = Config::load()?;
    let base_url = config.base_url_for(profile)?;

    let email = prompt("Email: ")?;
    let password = prompt_password("Password: ")?;
    println!(
        "{}",
        style::paint(
            style::SUBTEXT1,
            "note: milieu does not validate email or password format"
        )
    );

    let client = ApiClient::new(base_url, None)?;
    let _ = client
        .register(&RegisterRequest {
            email: email.clone(),
            password,
        })
        .await?;

    println!("{}", style::paint(style::GREEN, &format!("registered {}", email)));
    Ok(())
}
