use crate::error::{MilieuError, Result};
use crate::manifest::Manifest;
use crate::style;

pub mod push;
pub mod doctor;
pub mod clone;
pub mod add;
pub mod changes;
pub mod checkout;
pub mod init;
pub mod login;
pub mod log;
pub mod phrase;
pub mod register;
pub mod logout;
pub mod pull;
pub mod repos;
pub mod sessions;
pub mod branches;
pub mod status;
pub mod remove;

pub fn print_scope_user(profile: &str) {
    let label = match crate::auth::load_email(profile) {
        Ok(Some(email)) => format!("SCOPE: user {}", email),
        _ => "SCOPE: user".to_string(),
    };
    println!("{}", style::bold(style::MAUVE, &label));
}

pub fn print_scope_repo(manifest: &Manifest) {
    println!(
        "{}",
        style::bold(style::MAUVE, &format!("SCOPE: repo {}", manifest.repo_name))
    );
}

pub fn print_scope_branch(manifest: &Manifest, branch: &str) {
    println!(
        "{}",
        style::bold(
            style::MAUVE,
            &format!("SCOPE: repo {} -> branch {}", manifest.repo_name, branch)
        )
    );
}

pub fn prompt(text: &str) -> Result<String> {
    use std::io::{self, Write};
    let mut stdout = io::stdout();
    write!(stdout, "{}", text)?;
    stdout.flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

pub fn prompt_password(text: &str) -> Result<String> {
    rpassword::prompt_password(text).map_err(|e| MilieuError::Io(e))
}
