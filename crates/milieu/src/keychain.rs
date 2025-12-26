use crate::error::{Result, MilieuError};
use keyring::Entry;

const SERVICE: &str = "milieu";

pub fn set_secret(key: &str, value: &str) -> Result<()> {
    let entry = Entry::new(SERVICE, key)?;
    entry.set_password(value)?;
    Ok(())
}

pub fn get_secret(key: &str) -> Result<Option<String>> {
    let entry = Entry::new(SERVICE, key)?;
    match entry.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(MilieuError::Keyring(err)),
    }
}

pub fn delete_secret(key: &str) -> Result<()> {
    let entry = Entry::new(SERVICE, key)?;
    match entry.delete_password() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(MilieuError::Keyring(err)),
    }
}
