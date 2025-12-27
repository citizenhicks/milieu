use crate::auth;
use crate::error::{MilieuError, Result};
use crate::style;

pub fn show(profile: &str) -> Result<()> {
    crate::commands::print_scope_user(profile);
    let _user_id = auth::load_user_id(profile)?;
    match auth::load_phrase(profile)? {
        Some(phrase) => {
            println!(
                "{}",
                style::paint(
                    style::YELLOW,
                    "Recovery phrase (store this somewhere safe):"
                )
            );
            println!("{}", style::bold(style::LAVENDER, &phrase));
            Ok(())
        }
        None => Err(MilieuError::CommandFailed(
            "no recovery phrase found in keychain".to_string(),
        )),
    }
}

pub fn status(profile: &str) -> Result<()> {
    crate::commands::print_scope_user(profile);
    let _user_id = auth::load_user_id(profile)?;
    let exists = auth::load_phrase(profile)?.is_some();
    if exists {
        println!("{}", style::paint(style::GREEN, "phrase: present"));
    } else {
        println!("{}", style::paint(style::YELLOW, "phrase: missing"));
    }
    Ok(())
}
