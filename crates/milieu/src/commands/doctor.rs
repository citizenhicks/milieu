use crate::auth;
use crate::config::Config;
use crate::error::{MilieuError, Result};
use crate::keychain;
use crate::repo::{manifest_path, project_root};
use crate::style;

pub fn run(profile: &str) -> Result<()> {
    crate::commands::print_scope_user();
    println!("{}", style::bold(style::MAUVE, "Milieu doctor"));

    match Config::load() {
        Ok(config) => {
            println!("{}", style::paint(style::GREEN, "config: ok"));
            match config.base_url_for(profile) {
                Ok(_) => println!("{}", style::paint(style::GREEN, "base_url: ok")),
                Err(_) => println!(
                    "{}",
                    style::paint(
                        style::YELLOW,
                        "base_url: missing (set in ~/.config/milieu/config.toml)"
                    )
                ),
            }
        }
        Err(err) => println!(
            "{}",
            style::paint(style::RED, &format!("config: error ({})", err))
        ),
    }

    match keychain_check() {
        Ok(()) => println!("{}", style::paint(style::GREEN, "keychain: ok")),
        Err(err) => {
            println!(
                "{}",
                style::paint(style::RED, &format!("keychain: error ({})", err))
            );
            println!(
                "{}",
                style::paint(
                    style::YELLOW,
                    "hint: on Linux, enable a Secret Service provider such as GNOME Keyring or KWallet"
                )
            );
        }
    }

    match auth::load_auth_token(profile) {
        Ok(_) => println!("{}", style::paint(style::GREEN, "auth token: ok")),
        Err(_) => println!(
            "{}",
            style::paint(style::YELLOW, "auth token: missing (run `milieu login`)")
        ),
    }

    match auth::load_user_id(profile) {
        Ok(_user_id) => match auth::load_umk(profile) {
            Ok(_) => println!("{}", style::paint(style::GREEN, "umk: ok")),
            Err(_) => println!(
                "{}",
                style::paint(style::YELLOW, "umk: missing (run `milieu login`)")
            ),
        },
        Err(_) => println!(
            "{}",
            style::paint(style::YELLOW, "user id: missing (run `milieu login`)")
        ),
    }

    match project_root() {
        Ok(root) => {
            println!(
                "{}",
                style::paint(
                    style::GREEN,
                    &format!("project: ok ({})", root.display())
                )
            );
            let manifest = manifest_path()?;
            if manifest.exists() {
                println!("{}", style::paint(style::GREEN, "manifest: ok"));
            } else {
                println!(
                    "{}",
                    style::paint(style::YELLOW, "manifest: missing (run `milieu init`)"),
                );
            }
        }
        Err(err) => println!(
            "{}",
            style::paint(style::RED, &format!("project: error ({})", err)),
        ),
    }

    Ok(())
}

fn keychain_check() -> Result<()> {
    let key = "doctor_temp";
    keychain::set_secret(key, "ok")?;
    let value = keychain::get_secret(key)?;
    if value.as_deref() != Some("ok") {
        return Err(MilieuError::CommandFailed("keychain read mismatch".to_string()));
    }
    keychain::delete_secret(key)?;
    Ok(())
}
