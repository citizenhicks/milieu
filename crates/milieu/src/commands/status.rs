use crate::api::ApiClient;
use crate::auth;
use crate::config::Config;
use crate::crypto::{aad_for, decrypt_bytes};
use crate::error::Result;
use crate::keys;
use crate::manifest::Manifest;
use crate::repo::{manifest_path, project_root, validate_env_path};
use crate::style;
use blake3::{Hash, Hasher};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub async fn run(profile: &str, json: bool) -> Result<()> {
    let manifest = Manifest::load(&manifest_path()?)?;
    crate::commands::print_scope_repo(&manifest);

    let config = Config::load()?;
    let mut base_url = config.base_url_for(profile)?;
    if let Some(remote) = &manifest.remote {
        if let Some(url) = &remote.base_url {
            base_url = url.clone();
        }
    }

    let token = auth::load_auth_token(profile)?;
    let client = ApiClient::new(&base_url, Some(token))?;
    let repo_key = keys::get_or_fetch_repo_key(profile, &client, &manifest.repo_id).await?;

    let tracked_all: HashSet<String> = manifest
        .branches
        .iter()
        .flat_map(|branch| branch.files.iter().map(|entry| entry.path().to_string()))
        .collect();

    let untracked = find_untracked(&tracked_all)?;
    let remote_manifest = match client.get_manifest(&manifest.repo_id).await {
        Ok(value) => Some(value),
        Err(err) => {
            println!(
                "{}",
                style::paint(
                    style::PEACH,
                    &format!("warning: remote manifest unavailable ({})", err)
                )
            );
            None
        }
    };

    if json {
        let mut entries = Vec::new();
        for branch in &manifest.branches {
            if branch.files.is_empty() {
                entries.push(serde_json::json!({
                    "branch": branch.name,
                    "tracked": false,
                    "files": [],
                }));
                continue;
            }
            for entry in &branch.files {
                let path = entry.path();
                validate_env_path(path)?;
                let local = local_status(path)?;
                let remote = client
                    .get_latest(&manifest.repo_id, &branch.name, path)
                    .await?;
                let diff = change_kind(
                    &local,
                    remote.as_ref(),
                    &repo_key,
                    &manifest,
                    &branch.name,
                    entry,
                );
                entries.push(serde_json::json!({
                    "branch": branch.name,
                    "path": path,
                    "tag": entry.tag(),
                    "local": local.label(),
                    "remote": remote.as_ref().map(|_| "present").unwrap_or("missing"),
                    "status": change_kind_str(diff),
                    "local_version": entry.last_synced_version,
                    "remote_version": remote.as_ref().and_then(|obj| obj.version),
                }));
            }
        }
        println!("{}", serde_json::to_string_pretty(&entries)?);
        return Ok(());
    }

    if !untracked.is_empty() {
        println!("{}", style::bold(style::MAUVE, "untracked:"));
        for path in &untracked {
            println!("{}", style::paint(style::SKY, &format!("  ? {}", path)));
        }
        println!(
            "{}",
            style::paint(style::SUBTEXT1, "run `milieu add <file>` to track")
        );
    }

    for branch in &manifest.branches {
        let mut entries = Vec::new();
        let mut remote_only_manifest = Vec::new();

        if let Some(remote_manifest) = &remote_manifest {
            if let Some(remote_branch) = remote_manifest
                .branches
                .iter()
                .find(|b| b.name == branch.name)
            {
                for file in &remote_branch.files {
                    let path = file.path().to_string();
                    if !branch.files.iter().any(|f| f.path() == path) {
                        remote_only_manifest.push(path);
                    }
                }
            }
        }

        for entry in &branch.files {
            let path = entry.path();
            validate_env_path(path)?;
            let local = local_status(path)?;
            let remote = client
                .get_latest(&manifest.repo_id, &branch.name, path)
                .await?;
            let diff = change_kind(
                &local,
                remote.as_ref(),
                &repo_key,
                &manifest,
                &branch.name,
                entry,
            );
            let entry_status = StatusEntry {
                path: path.to_string(),
                kind: diff,
                local_version: entry.last_synced_version,
                remote_version: remote.as_ref().and_then(|obj| obj.version),
            };

            entries.push(entry_status);
        }

        for path in &remote_only_manifest {
            let remote = client
                .get_latest(&manifest.repo_id, &branch.name, path)
                .await?;
            let entry_status = StatusEntry {
                path: path.to_string(),
                kind: ChangeKind::NewRemote,
                local_version: None,
                remote_version: remote.as_ref().and_then(|obj| obj.version),
            };
            entries.push(entry_status);
        }

        let label = if branch.name == manifest.active_branch {
            format!("branch: *{}", branch.name)
        } else {
            format!("branch: {}", branch.name)
        };
        println!("{}", style::bold(style::MAUVE, &label));

        if entries.is_empty() {
            println!("{}", style::paint(style::SUBTEXT1, "  (no tracked files)"));
            continue;
        }

        let mut file_width = "File".len();
        let mut local_width = "Local".len();
        let mut remote_width = "Remote".len();
        let mut diff_width = "Diff".len();

        let rows: Vec<_> = entries
            .iter()
            .map(|entry| {
                let local = format_version(entry.kind, entry.local_version, true);
                let remote = format_version(entry.kind, entry.remote_version, false);
                let diff = change_kind_str(entry.kind).to_string();
                file_width = file_width.max(entry.path.len());
                local_width = local_width.max(local.len());
                remote_width = remote_width.max(remote.len());
                diff_width = diff_width.max(diff.len());
                (entry, local, remote, diff)
            })
            .collect();

        println!(
            "  {}  {}  {}  {}",
            style::bold(style::MAUVE, &format!("{:<width$}", "File", width = file_width)),
            style::bold(style::MAUVE, &format!("{:<width$}", "Local", width = local_width)),
            style::bold(style::MAUVE, &format!("{:<width$}", "Remote", width = remote_width)),
            style::bold(style::MAUVE, &format!("{:<width$}", "Diff", width = diff_width)),
        );
        println!(
            "  {}  {}  {}  {}",
            style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = file_width)),
            style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = local_width)),
            style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = remote_width)),
            style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = diff_width)),
        );

        for (entry, local, remote, diff) in rows {
            let color = status_color(entry.kind);
            let line = format!(
                "  {:<file_width$}  {:<local_width$}  {:<remote_width$}  {:<diff_width$}",
                entry.path,
                local,
                remote,
                diff,
                file_width = file_width,
                local_width = local_width,
                remote_width = remote_width,
                diff_width = diff_width
            );
            println!("{}", style::paint(color, &line));
        }

        if entries.iter().any(|e| matches!(e.kind, ChangeKind::NewRemote | ChangeKind::ModifiedRemote | ChangeKind::ModifiedBoth)) {
            println!(
                "{}",
                style::paint(
                    style::SUBTEXT1,
                    "remote has new changes; run `milieu pull` before pushing"
                )
            );
        }
        println!(
            "{}",
            style::paint(
                style::SUBTEXT1,
                "run `milieu changes <file> --branch <name>` to view diffs"
            )
        );
    }

    Ok(())
}

fn local_status(path: &str) -> Result<LocalStatus> {
    match fs::read(path) {
        Ok(data) => {
            let mut hasher = Hasher::new();
            hasher.update(&data);
            let hash = hasher.finalize();
            Ok(LocalStatus::Present { hash })
        }
        Err(_) => Ok(LocalStatus::Missing),
    }
}

#[derive(Clone)]
enum LocalStatus {
    Present { hash: Hash },
    Missing,
}

impl LocalStatus {
    fn label(&self) -> &'static str {
        match self {
            LocalStatus::Present { .. } => "present",
            LocalStatus::Missing => "missing",
        }
    }
}

#[derive(Copy, Clone)]
enum ChangeKind {
    Clean,
    NewLocal,
    NewRemote,
    ModifiedLocal,
    ModifiedRemote,
    ModifiedBoth,
    ModifiedUnknown,
    None,
}

struct StatusEntry {
    path: String,
    kind: ChangeKind,
    local_version: Option<u32>,
    remote_version: Option<u32>,
}

fn change_kind(
    local: &LocalStatus,
    remote: Option<&crate::api::ObjectResponse>,
    repo_key: &[u8; 32],
    manifest: &Manifest,
    branch: &str,
    entry: &crate::manifest::FileEntry,
) -> ChangeKind {
    let base_hash = entry
        .last_synced_hash
        .as_deref()
        .and_then(|hex| blake3::Hash::from_hex(hex).ok());

    match (local, remote) {
        (LocalStatus::Missing, None) => ChangeKind::None,
        (LocalStatus::Missing, Some(_)) => ChangeKind::NewRemote,
        (LocalStatus::Present { .. }, None) => ChangeKind::NewLocal,
        (LocalStatus::Present { hash: local_hash }, Some(remote_obj)) => {
            let schema_version = remote_obj.schema_version;
            let aad = aad_for(schema_version, &manifest.repo_id, branch, entry.path(), entry.tag());
            let remote_hash = match decrypt_bytes(repo_key, &aad, &remote_obj.nonce, &remote_obj.ciphertext) {
                Ok(plaintext) => blake3::hash(&plaintext),
                Err(_) => return ChangeKind::ModifiedUnknown,
            };

            if let Some(base) = base_hash {
                let local_changed = local_hash != &base;
                let remote_changed = remote_hash != base;

                match (local_changed, remote_changed) {
                    (false, false) => ChangeKind::Clean,
                    (true, false) => ChangeKind::ModifiedLocal,
                    (false, true) => ChangeKind::ModifiedRemote,
                    (true, true) => {
                        if local_hash == &remote_hash {
                            ChangeKind::Clean
                        } else {
                            ChangeKind::ModifiedBoth
                        }
                    }
                }
            } else if local_hash == &remote_hash {
                ChangeKind::Clean
            } else {
                ChangeKind::ModifiedUnknown
            }
        }
    }
}

fn change_kind_str(kind: ChangeKind) -> &'static str {
    match kind {
        ChangeKind::Clean => "no_change",
        ChangeKind::NewLocal => "new_local",
        ChangeKind::NewRemote => "new_remote",
        ChangeKind::ModifiedLocal => "modified_local",
        ChangeKind::ModifiedRemote => "modified_remote",
        ChangeKind::ModifiedBoth => "modified_both",
        ChangeKind::ModifiedUnknown => "modified_unknown",
        ChangeKind::None => "no_change",
    }
}

fn format_version(kind: ChangeKind, version: Option<u32>, is_local: bool) -> String {
    let base = match version {
        Some(v) => format!("v{}", v),
        None => {
            if is_local && matches!(kind, ChangeKind::NewLocal) {
                "v1".to_string()
            } else {
                "-".to_string()
            }
        }
    };
    if is_local
        && matches!(
            kind,
            ChangeKind::NewLocal
                | ChangeKind::ModifiedLocal
                | ChangeKind::ModifiedBoth
                | ChangeKind::ModifiedUnknown
        )
    {
        format!("{}(m)", base)
    } else {
        base
    }
}

fn status_color(kind: ChangeKind) -> style::Rgb {
    match kind {
        ChangeKind::Clean | ChangeKind::None => style::GREEN,
        ChangeKind::NewLocal => style::GREEN,
        ChangeKind::NewRemote => style::PEACH,
        ChangeKind::ModifiedLocal => style::YELLOW,
        ChangeKind::ModifiedRemote => style::PEACH,
        ChangeKind::ModifiedBoth => style::RED,
        ChangeKind::ModifiedUnknown => style::YELLOW,
    }
}

// path validation centralized in repo::validate_env_path

fn find_untracked(tracked: &HashSet<String>) -> Result<Vec<String>> {
    let root = project_root()?;
    let mut out = Vec::new();
    collect_env_files(&root, &root, tracked, &mut out)?;
    out.sort();
    Ok(out)
}

fn collect_env_files(
    root: &Path,
    dir: &Path,
    tracked: &HashSet<String>,
    out: &mut Vec<String>,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if path.is_dir() {
            if name == ".milieu" || name == ".git" || name == "target" || name == "node_modules" {
                continue;
            }
            collect_env_files(root, &path, tracked, out)?;
            continue;
        }

        if name == ".env" || name.starts_with(".env.") {
            let rel = path.strip_prefix(root).unwrap_or(&path);
            let rel_str = rel.to_string_lossy().to_string();
            if !tracked.contains(&rel_str) {
                out.push(rel_str);
            }
        }
    }
    Ok(())
}
