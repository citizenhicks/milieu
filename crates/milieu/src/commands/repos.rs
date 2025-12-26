use crate::api::{ApiClient, InviteInfo, RepoAccessEntry, RepoResponse};
use crate::auth;
use crate::config::Config;
use crate::error::Result;
use crate::keys;
use crate::style;
use atty::Stream;

pub async fn list(profile: &str) -> Result<()> {
    crate::commands::print_scope_user();
    let config = Config::load()?;
    let base_url = config.base_url_for(profile)?;

    let token = auth::load_auth_token(profile)?;
    let client = ApiClient::new(base_url, Some(token))?;

    let repos = client.get_repos().await?;
    if repos.is_empty() {
        println!(
            "{}",
            style::paint(style::YELLOW, "No repos linked yet.")
        );
        return Ok(());
    }

    let mut rows = Vec::new();
    let mut id_width = "Repo ID".len();
    let mut name_width = "Name".len();
    let mut owner_width = "Owner".len();
    let mut access_width = "Access".len();
    for repo in repos {
        id_width = id_width.max(repo.repo_id.len());
        name_width = name_width.max(repo.name.len());
        if let Some(owner) = &repo.owner_email {
            owner_width = owner_width.max(owner.len());
        }
        if let Some(access) = &repo.access {
            access_width = access_width.max(access.len());
        }
        rows.push(repo);
    }

    println!(
        "{}  {}  {}  {}  {}",
        style::bold(style::MAUVE, &format!("{:<width$}", "Repo ID", width = id_width)),
        style::bold(style::MAUVE, &format!("{:<width$}", "Name", width = name_width)),
        style::bold(style::MAUVE, &format!("{:<width$}", "Access", width = access_width)),
        style::bold(style::MAUVE, &format!("{:<width$}", "Owner", width = owner_width)),
        style::bold(style::MAUVE, "Last Seen")
    );
    println!(
        "{}  {}  {}  {}  {}",
        style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = id_width)),
        style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = name_width)),
        style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = access_width)),
        style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = owner_width)),
        style::paint(style::SUBTEXT1, "---------")
    );
    for repo in rows {
        println!(
            "{}  {}  {}  {}  {}",
            style::paint(style::TEXT, &format!("{:<width$}", repo.repo_id, width = id_width)),
            style::paint(style::TEXT, &format!("{:<width$}", repo.name, width = name_width)),
            style::paint(
                style::TEXT,
                &format!(
                    "{:<width$}",
                    repo.access.as_deref().unwrap_or("read"),
                    width = access_width
                )
            ),
            style::paint(
                style::TEXT,
                &format!(
                    "{:<width$}",
                    repo.owner_email.as_deref().unwrap_or("-"),
                    width = owner_width
                )
            ),
            style::paint(style::SUBTEXT1, &repo.last_seen)
        );
    }

    Ok(())
}

pub async fn manage_list(profile: &str, repo_name: &str) -> Result<()> {
    crate::commands::print_scope_user();
    let (client, repo) = client_and_repo(profile, repo_name).await?;
    let entries = client.get_repo_access(&repo.repo_id).await?;

    if entries.is_empty() {
        println!("{}", style::paint(style::SUBTEXT1, "no collaborators yet"));
        return Ok(());
    }

    print_access_table(&entries);
    Ok(())
}

pub async fn manage_add(profile: &str, repo_name: &str, email: &str, role: &str) -> Result<()> {
    crate::commands::print_scope_user();
    let (client, repo) = client_and_repo(profile, repo_name).await?;
    client.invite_repo_access(&repo.repo_id, email, role).await?;
    println!(
        "{}",
        style::paint(
            style::GREEN,
            &format!("invited {} ({}) to {}", email, role, repo.name)
        )
    );
    Ok(())
}

pub async fn manage_set(profile: &str, repo_name: &str, email: &str, role: &str) -> Result<()> {
    crate::commands::print_scope_user();
    let (client, repo) = client_and_repo(profile, repo_name).await?;
    client.update_repo_access(&repo.repo_id, email, role).await?;
    println!(
        "{}",
        style::paint(
            style::GREEN,
            &format!("updated {} to {} on {}", email, role, repo.name)
        )
    );
    Ok(())
}

pub async fn manage_remove(profile: &str, repo_name: &str, email: &str) -> Result<()> {
    crate::commands::print_scope_user();
    let (client, repo) = client_and_repo(profile, repo_name).await?;
    client.revoke_repo_access(&repo.repo_id, email).await?;
    println!(
        "{}",
        style::paint(
            style::PEACH,
            &format!("removed {} from {}", email, repo.name)
        )
    );
    Ok(())
}

pub async fn manage_delete(profile: &str, repo_name: &str) -> Result<()> {
    crate::commands::print_scope_user();
    let (client, repo) = client_and_repo(profile, repo_name).await?;

    let first = crate::commands::prompt(&format!(
        "delete remote repo '{}' (this cannot be undone)? [y/N] ",
        repo.name
    ))?;
    if first.to_lowercase().as_str() != "y" {
        println!("{}", style::paint(style::SUBTEXT1, "aborted"));
        return Ok(());
    }

    let confirm = crate::commands::prompt(&format!(
        "type repo name '{}' to confirm delete: ",
        repo.name
    ))?;
    if confirm.trim() != repo.name {
        println!("{}", style::paint(style::SUBTEXT1, "confirmation did not match; aborted"));
        return Ok(());
    }

    client.delete_repo(&repo.repo_id).await?;
    println!(
        "{}",
        style::paint(style::PEACH, &format!("deleted remote repo '{}'", repo.name))
    );
    Ok(())
}

pub async fn manage_invites(profile: &str) -> Result<()> {
    crate::commands::print_scope_user();
    let client = client_only(profile).await?;
    let invites = client.get_invites().await?;
    if invites.is_empty() {
        println!("{}", style::paint(style::SUBTEXT1, "no pending invites"));
        return Ok(());
    }

    print_invites(&invites);

    if !atty::is(Stream::Stdin) {
        return Ok(());
    }

    for invite in invites {
        let prompt = format!(
            "accept invite to {} from {} as {}? [a/r/s] ",
            invite.repo_name, invite.invited_by, invite.role
        );
        let answer = crate::commands::prompt(&prompt)?;
        match answer.to_lowercase().as_str() {
            "a" | "accept" => {
                client.accept_invite(&invite.id).await?;
                println!(
                    "{}",
                    style::paint(style::GREEN, &format!("accepted {}", invite.repo_name))
                );
            }
            "r" | "reject" => {
                client.reject_invite(&invite.id).await?;
                println!(
                    "{}",
                    style::paint(style::PEACH, &format!("rejected {}", invite.repo_name))
                );
            }
            _ => {
                println!(
                    "{}",
                    style::paint(style::SUBTEXT1, "skipped")
                );
            }
        }
    }
    Ok(())
}

pub async fn manage_share(profile: &str, repo_name: &str) -> Result<()> {
    crate::commands::print_scope_user();
    let (client, repo) = client_and_repo(profile, repo_name).await?;
    let _ = keys::ensure_user_keypair(profile, &client).await?;
    let repo_key = keys::get_or_fetch_repo_key(profile, &client, &repo.repo_id).await?;

    let entries = client.get_repo_access(&repo.repo_id).await?;
    if entries.is_empty() {
        println!("{}", style::paint(style::SUBTEXT1, "no collaborators yet"));
        return Ok(());
    }

    let current_email = auth::load_email(profile)?.unwrap_or_default();
    let mut shared = 0;
    let mut missing = Vec::new();
    for entry in entries {
        if entry.status != "active" {
            continue;
        }
        if entry.email == current_email {
            continue;
        }
        match entry.public_key.as_deref() {
            Some(public_key) => {
                let wrapped = keys::wrap_repo_key_for_user(public_key, &repo_key).await?;
                client
                    .put_repo_key(
                        &repo.repo_id,
                        &wrapped,
                        entry
                            .key_algorithm
                            .as_deref()
                            .unwrap_or("x25519-hkdf-xchacha20poly1305"),
                        Some(&entry.email),
                    )
                    .await?;
                shared += 1;
            }
            None => {
                missing.push(entry.email);
            }
        }
    }

    println!(
        "{}",
        style::paint(
            style::GREEN,
            &format!("shared repo key with {} collaborator(s)", shared)
        )
    );
    if !missing.is_empty() {
        let mut message = String::from("missing public keys:");
        for email in missing {
            message.push_str(&format!("\n  - {}", email));
        }
        println!("{}", style::paint(style::YELLOW, &message));
    }
    Ok(())
}

pub async fn manage_accept(profile: &str, invite_id: &str) -> Result<()> {
    crate::commands::print_scope_user();
    let client = client_only(profile).await?;
    client.accept_invite(invite_id).await?;
    println!("{}", style::paint(style::GREEN, "invite accepted"));
    Ok(())
}

pub async fn manage_reject(profile: &str, invite_id: &str) -> Result<()> {
    crate::commands::print_scope_user();
    let client = client_only(profile).await?;
    client.reject_invite(invite_id).await?;
    println!("{}", style::paint(style::PEACH, "invite rejected"));
    Ok(())
}

fn print_access_table(entries: &[RepoAccessEntry]) {
    let mut email_width = "Email".len();
    let mut role_width = "Role".len();
    let mut status_width = "Status".len();
    let mut invited_width = "Invited By".len();

    for entry in entries {
        email_width = email_width.max(entry.email.len());
        role_width = role_width.max(entry.role.len());
        status_width = status_width.max(entry.status.len());
        if let Some(invited_by) = &entry.invited_by {
            invited_width = invited_width.max(invited_by.len());
        }
    }

    println!(
        "{}  {}  {}  {}",
        style::bold(style::MAUVE, &format!("{:<width$}", "Email", width = email_width)),
        style::bold(style::MAUVE, &format!("{:<width$}", "Role", width = role_width)),
        style::bold(style::MAUVE, &format!("{:<width$}", "Status", width = status_width)),
        style::bold(style::MAUVE, &format!("{:<width$}", "Invited By", width = invited_width)),
    );
    println!(
        "{}  {}  {}  {}",
        style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = email_width)),
        style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = role_width)),
        style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = status_width)),
        style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = invited_width)),
    );

    for entry in entries {
        let invited_by = entry.invited_by.as_deref().unwrap_or("-");
        let line = format!(
            "{:<email_width$}  {:<role_width$}  {:<status_width$}  {:<invited_width$}",
            entry.email,
            entry.role,
            entry.status,
            invited_by,
            email_width = email_width,
            role_width = role_width,
            status_width = status_width,
            invited_width = invited_width
        );
        let color = if entry.status == "pending" {
            style::PEACH
        } else {
            style::TEXT
        };
        println!("{}", style::paint(color, &line));
    }
}

fn print_invites(invites: &[InviteInfo]) {
    let mut repo_width = "Repo".len();
    let mut role_width = "Role".len();
    let mut inviter_width = "Invited By".len();
    for invite in invites {
        repo_width = repo_width.max(invite.repo_name.len());
        role_width = role_width.max(invite.role.len());
        inviter_width = inviter_width.max(invite.invited_by.len());
    }

    println!(
        "{}  {}  {}  {}",
        style::bold(style::MAUVE, &format!("{:<width$}", "Repo", width = repo_width)),
        style::bold(style::MAUVE, &format!("{:<width$}", "Role", width = role_width)),
        style::bold(style::MAUVE, &format!("{:<width$}", "Invited By", width = inviter_width)),
        style::bold(style::MAUVE, "Invite ID"),
    );
    println!(
        "{}  {}  {}  {}",
        style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = repo_width)),
        style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = role_width)),
        style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = inviter_width)),
        style::paint(style::SUBTEXT1, "---------"),
    );
    for invite in invites {
        println!(
            "{}  {}  {}  {}",
            style::paint(style::TEXT, &format!("{:<width$}", invite.repo_name, width = repo_width)),
            style::paint(style::TEXT, &format!("{:<width$}", invite.role, width = role_width)),
            style::paint(style::TEXT, &format!("{:<width$}", invite.invited_by, width = inviter_width)),
            style::paint(style::SUBTEXT1, &invite.id),
        );
    }
}

async fn client_only(profile: &str) -> Result<ApiClient> {
    let config = Config::load()?;
    let base_url = config.base_url_for(profile)?;
    let token = auth::load_auth_token(profile)?;
    ApiClient::new(base_url, Some(token))
}

async fn client_and_repo(profile: &str, repo_name: &str) -> Result<(ApiClient, RepoResponse)> {
    let client = client_only(profile).await?;
    let repo = client.get_repo_by_name(repo_name).await?;
    Ok((client, repo))
}
