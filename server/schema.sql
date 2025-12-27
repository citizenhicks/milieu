-- Users and auth
CREATE TABLE IF NOT EXISTS users (
  id TEXT PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  password_salt TEXT NOT NULL,
  password_iters INTEGER NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS repos (
  id TEXT PRIMARY KEY,
  owner_user_id TEXT NOT NULL,
  name TEXT NOT NULL,
  manifest_json TEXT,
  created_at TEXT NOT NULL,
  UNIQUE (owner_user_id, name),
  FOREIGN KEY (owner_user_id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS sessions (
  token TEXT PRIMARY KEY,
  token_suffix TEXT NOT NULL DEFAULT '',
  user_id TEXT NOT NULL,
  host TEXT NOT NULL,
  created_at TEXT NOT NULL,
  expires_at TEXT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Login rate limiting
CREATE TABLE IF NOT EXISTS login_attempts (
  key TEXT PRIMARY KEY,
  count INTEGER NOT NULL,
  window_start TEXT NOT NULL
);

-- UMK blobs
CREATE TABLE IF NOT EXISTS umk_blobs (
  user_id TEXT PRIMARY KEY,
  encrypted_umk TEXT NOT NULL,
  kdf_params TEXT NOT NULL,
  version INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

-- User public keys for repo key wrapping
CREATE TABLE IF NOT EXISTS user_keys (
  user_id TEXT PRIMARY KEY,
  public_key TEXT NOT NULL,
  algorithm TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Repo keys wrapped per user
CREATE TABLE IF NOT EXISTS repo_keys (
  repo_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  wrapped_key TEXT NOT NULL,
  algorithm TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (repo_id, user_id),
  FOREIGN KEY (repo_id) REFERENCES repos(id),
  FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Audit log for repo key writes
CREATE TABLE IF NOT EXISTS repo_key_events (
  id TEXT PRIMARY KEY,
  repo_id TEXT NOT NULL,
  requester_user_id TEXT NOT NULL,
  requester_email TEXT NOT NULL,
  target_user_id TEXT NOT NULL,
  target_email TEXT NOT NULL,
  action TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (repo_id) REFERENCES repos(id),
  FOREIGN KEY (requester_user_id) REFERENCES users(id),
  FOREIGN KEY (target_user_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS repo_key_events_repo
  ON repo_key_events (repo_id, created_at DESC);

-- Encrypted env objects
CREATE TABLE IF NOT EXISTS env_objects (
  id TEXT PRIMARY KEY,
  repo_id TEXT NOT NULL,
  branch TEXT NOT NULL,
  path TEXT NOT NULL,
  nonce TEXT NOT NULL,
  ciphertext TEXT NOT NULL,
  aad TEXT NOT NULL,
  ciphertext_hash TEXT NOT NULL,
  version INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  client_created_at TEXT,
  schema_version INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS env_objects_lookup
  ON env_objects (repo_id, branch, path, created_at DESC);

CREATE TABLE IF NOT EXISTS repo_links (
  user_id TEXT NOT NULL,
  repo_id TEXT NOT NULL,
  last_seen TEXT NOT NULL,
  PRIMARY KEY (user_id, repo_id),
  FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Repo sharing
CREATE TABLE IF NOT EXISTS repo_access (
  repo_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  role TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (repo_id, user_id),
  FOREIGN KEY (repo_id) REFERENCES repos(id),
  FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS repo_invites (
  id TEXT PRIMARY KEY,
  repo_id TEXT NOT NULL,
  email TEXT NOT NULL,
  invited_by_user_id TEXT NOT NULL,
  role TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (repo_id) REFERENCES repos(id),
  FOREIGN KEY (invited_by_user_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS repo_invites_email
  ON repo_invites (email, status);
