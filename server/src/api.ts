import { Env } from "./env";

type Json = Record<string, unknown>;

type LoginRequest = {
  email: string;
  password: string;
  host: string;
};

type RegisterRequest = LoginRequest;

type LoginResponse = {
  access_token: string;
  user_id: string;
  warning?: string;
};

type SessionInfo = {
  host: string;
  created_at: string;
  expires_at: string;
  token_suffix: string;
  active: boolean;
};

type RepoCreateRequest = {
  name: string;
};

type RepoResponse = {
  repo_id: string;
  name: string;
};

type RepoManifest = {
  version: number;
  repo_id: string;
  repo_name: string;
  active_branch: string;
  branches: { name: string; files: { path: string; tag?: string }[] }[];
};

type RegisterResponse = {
  user_id: string;
};

type UmkResponse = {
  encrypted_umk: string;
  kdf_params: Json;
  version: number;
  updated_at?: string;
};

type UmkRequest = {
  encrypted_umk: string;
  kdf_params: Json;
  version: number;
};

type ObjectRequest = {
  path: string;
  nonce: string;
  ciphertext: string;
  aad: string;
  ciphertext_hash: string;
  version?: number;
  created_at: string;
  schema_version: number;
};

type HistoryEntry = {
  version: number;
  created_at: string;
  ciphertext_hash: string | null;
};

type RepoAccessEntry = {
  email: string;
  role: string;
  status: "active" | "pending";
  invited_by?: string;
  created_at: string;
  public_key?: string;
  key_algorithm?: string;
};

type InviteInfo = {
  id: string;
  repo_id: string;
  repo_name: string;
  role: string;
  invited_by: string;
  created_at: string;
};

type UserKeyRequest = {
  public_key: string;
  algorithm: string;
};

type UserKeyResponse = UserKeyRequest & {
  created_at: string;
  updated_at: string;
};

type RepoKeyRequest = {
  wrapped_key: string;
  algorithm: string;
  email?: string;
};

type RepoKeyResponse = {
  wrapped_key: string;
  algorithm: string;
  created_at: string;
  updated_at: string;
};

const encoder = new TextEncoder();
const MAX_REPO_BYTES = 1 * 1024 * 1024;

export async function handleApiRequest(
  request: Request,
  env: Env,
): Promise<Response> {
  const url = new URL(request.url);
  const pathname = url.pathname;

  if (pathname === "/v1/auth/login" && request.method === "POST") {
    return handleLogin(request, env);
  }

    if (pathname === "/v1/auth/register" && request.method === "POST") {
      return handleRegister(request, env);
    }
    if (pathname === "/v1/auth/logout" && request.method === "POST") {
      return withAuth(request, env, async (userId) =>
        handleLogout(request, env, userId),
      );
    }

    if (pathname === "/v1/repos") {
      return withAuth(request, env, async (userId) => {
        if (request.method === "POST") {
          return handleCreateRepo(request, env, userId);
        }
        if (request.method === "GET") {
          return handleGetRepo(request, env, userId);
        }
        return json({ error: "method_not_allowed" }, 405);
      });
    }

    const repoDeleteMatch = pathname.match(/^\/v1\/repos\/([^/]+)$/);
    if (repoDeleteMatch) {
      const repoId = repoDeleteMatch[1];
      return withAuth(request, env, async (userId) => {
        if (request.method === "DELETE") {
          return handleDeleteRepo(env, userId, repoId);
        }
        return json({ error: "method_not_allowed" }, 405);
      });
    }

    const repoManifestMatch = pathname.match(
      /^\/v1\/repos\/([^/]+)\/manifest$/,
    );
    if (repoManifestMatch) {
      const repoId = repoManifestMatch[1];
      return withAuth(request, env, async (userId) => {
        if (request.method === "GET") {
          return handleGetManifest(env, userId, repoId);
        }
        if (request.method === "PUT") {
          return handlePutManifest(request, env, userId, repoId);
        }
        return json({ error: "method_not_allowed" }, 405);
      });
    }

    if (pathname === "/v1/users/me/umk") {
      return withAuth(request, env, async (userId) => {
        if (request.method === "GET") {
          return handleGetUmk(userId, env);
        }
        if (request.method === "PUT") {
          return handlePutUmk(request, userId, env);
        }
        return json({ error: "method_not_allowed" }, 405);
      });
    }

    if (pathname === "/v1/users/me/key") {
      return withAuth(request, env, async (userId) => {
        if (request.method === "GET") {
          return handleGetUserKey(userId, env);
        }
        if (request.method === "PUT") {
          return handlePutUserKey(request, userId, env);
        }
        return json({ error: "method_not_allowed" }, 405);
      });
    }

    if (pathname === "/v1/users/me/repos") {
      return withAuth(request, env, async (userId) => {
        if (request.method === "GET") {
          return handleGetRepos(userId, env);
        }
        return json({ error: "method_not_allowed" }, 405);
      });
    }

    if (pathname === "/v1/users/me/sessions") {
      return withAuth(request, env, async (userId) => {
        if (request.method === "GET") {
          return handleGetSessions(userId, env);
        }
        return json({ error: "method_not_allowed" }, 405);
      });
    }

    if (pathname === "/v1/users/me/invites") {
      return withAuth(request, env, async (userId) => {
        if (request.method === "GET") {
          return handleGetInvites(userId, env);
        }
        return json({ error: "method_not_allowed" }, 405);
      });
    }

    const inviteActionMatch = pathname.match(
      /^\/v1\/users\/me\/invites\/([^/]+)\/(accept|reject)$/,
    );
    if (inviteActionMatch) {
      const inviteId = inviteActionMatch[1];
      const action = inviteActionMatch[2];
      return withAuth(request, env, async (userId) => {
        if (request.method === "POST") {
          return handleInviteAction(env, userId, inviteId, action);
        }
        return json({ error: "method_not_allowed" }, 405);
      });
    }

    const repoMatch = pathname.match(
      /^\/v1\/repos\/([^/]+)\/branches\/([^/]+)\/objects(\/latest)?$/,
    );
    if (repoMatch) {
      const repoId = repoMatch[1];
      const branch = repoMatch[2];
      const isLatest = repoMatch[3] === "/latest";
      return withAuth(request, env, async (userId) => {
        if (request.method === "POST" && !isLatest) {
          return handlePostObject(request, env, userId, repoId, branch);
        }
        if (request.method === "GET" && isLatest) {
          const path = url.searchParams.get("path");
          if (!path) {
            return json({ error: "missing_path" }, 400);
          }
          return handleGetLatest(env, userId, repoId, branch, path);
        }
        return json({ error: "method_not_allowed" }, 405);
      });
    }

    const branchListMatch = pathname.match(/^\/v1\/repos\/([^/]+)\/branches$/);
    if (branchListMatch) {
      const repoId = branchListMatch[1];
      return withAuth(request, env, async (userId) => {
        if (request.method === "GET") {
          return handleListBranches(env, userId, repoId);
        }
        return json({ error: "method_not_allowed" }, 405);
      });
    }

    const accessMatch = pathname.match(/^\/v1\/repos\/([^/]+)\/access$/);
    if (accessMatch) {
      const repoId = accessMatch[1];
      return withAuth(request, env, async (userId) => {
        if (request.method === "GET") {
          return handleGetAccess(env, userId, repoId);
        }
        if (request.method === "POST") {
          return handleInviteAccess(request, env, userId, repoId);
        }
        if (request.method === "PATCH") {
          return handleUpdateAccess(request, env, userId, repoId);
        }
        if (request.method === "DELETE") {
          const email = url.searchParams.get("email");
          if (!email) return json({ error: "missing_email" }, 400);
          return handleRevokeAccess(env, userId, repoId, email);
        }
        return json({ error: "method_not_allowed" }, 405);
      });
    }

    const repoKeyMatch = pathname.match(/^\/v1\/repos\/([^/]+)\/key$/);
    if (repoKeyMatch) {
      const repoId = repoKeyMatch[1];
      return withAuth(request, env, async (userId) => {
        if (request.method === "GET") {
          return handleGetRepoKey(env, userId, repoId);
        }
        if (request.method === "PUT") {
          return handlePutRepoKey(request, env, userId, repoId);
        }
        return json({ error: "method_not_allowed" }, 405);
      });
    }

    const historyMatch = pathname.match(
      /^\/v1\/repos\/([^/]+)\/branches\/([^/]+)\/objects\/history$/,
    );
    if (historyMatch) {
      const repoId = historyMatch[1];
      const branch = historyMatch[2];
      return withAuth(request, env, async (userId) => {
        if (request.method === "GET") {
          const path = url.searchParams.get("path");
          if (!path) return json({ error: "missing_path" }, 400);
          return handleHistory(env, userId, repoId, branch, path);
        }
        return json({ error: "method_not_allowed" }, 405);
      });
    }

    const versionMatch = pathname.match(
      /^\/v1\/repos\/([^/]+)\/branches\/([^/]+)\/objects\/version$/,
    );
    if (versionMatch) {
      const repoId = versionMatch[1];
      const branch = versionMatch[2];
      return withAuth(request, env, async (userId) => {
        if (request.method === "GET") {
          const path = url.searchParams.get("path");
          const version = url.searchParams.get("version");
          if (!path || !version) return json({ error: "missing_params" }, 400);
          return handleGetVersion(
            env,
            userId,
            repoId,
            branch,
            path,
            Number(version),
          );
        }
        return json({ error: "method_not_allowed" }, 405);
      });
    }

  return json({ error: "not_found" }, 404);
}

async function handleLogin(request: Request, env: Env): Promise<Response> {
  const ip =
    request.headers.get("CF-Connecting-IP") ||
    request.headers.get("X-Forwarded-For") ||
    "unknown";
  const body = await request.json<LoginRequest>().catch(() => null);
  const email = normalizeEmail(body?.email);
  if (!body || !email || !body.password) {
    return json({ error: "invalid_request" }, 400);
  }
  const rateKey = `${ip}:${email}`;
  const limited = await isLoginRateLimited(env, rateKey);
  if (limited) {
    return json({ error: "rate_limited" }, 429);
  }
  const host =
    body.host && body.host.trim().length > 0 ? body.host.trim() : "unknown";

  const row = await env.DB.prepare(
    "SELECT id, password_hash, password_salt, password_iters FROM users WHERE email = ?",
  )
    .bind(email)
    .first<Record<string, string | number>>();

  if (!row) {
    return json({ error: "invalid_credentials" }, 401);
  }

  const ok = await verifyPassword(
    body.password,
    row.password_salt as string,
    row.password_iters as number,
    row.password_hash as string,
  );

  if (!ok) {
    await recordLoginAttempt(env, rateKey);
    return json({ error: "invalid_credentials" }, 401);
  }

  const token = await randomToken();
  const tokenHash = await hashToken(token);
  const tokenSuffix = token.length > 6 ? token.slice(-6) : token;
  const now = new Date();
  const ttlHours = Number(env.SESSION_TTL_HOURS || "720");
  const expiresAt = new Date(now.getTime() + ttlHours * 3600 * 1000);

  await expireHostSessions(env, row.id as string, host);
  await env.DB.prepare(
    "INSERT INTO sessions (token, token_suffix, user_id, host, created_at, expires_at) VALUES (?, ?, ?, ?, ?, ?)",
  )
    .bind(
      tokenHash,
      tokenSuffix,
      row.id as string,
      host,
      now.toISOString(),
      expiresAt.toISOString(),
    )
    .run();
  await clearLoginAttempts(env, rateKey);
  await pruneSessions(env, row.id as string, 12);

  const response: LoginResponse = {
    access_token: token,
    user_id: row.id as string,
    warning: "Beta testing: use at your own risk.",
  };
  return json(response, 200);
}

async function handleLogout(
  request: Request,
  env: Env,
  userId: string,
): Promise<Response> {
  const token = getBearerToken(request.headers.get("Authorization"));
  if (!token) return json({ error: "missing_auth" }, 401);
  const tokenHash = await hashToken(token);
  const now = new Date().toISOString();
  await env.DB.prepare(
    "UPDATE sessions SET expires_at = ? WHERE token = ? AND user_id = ?",
  )
    .bind(now, tokenHash, userId)
    .run();
  return json({ ok: true }, 200);
}

function rateConfig(env: Env): { limit: number; windowSeconds: number } {
  const limit = Number(env.LOGIN_RATE_LIMIT || "10");
  const windowSeconds = Number(env.LOGIN_RATE_WINDOW_SECONDS || "900");
  return { limit, windowSeconds };
}

async function isLoginRateLimited(env: Env, key: string): Promise<boolean> {
  const { limit, windowSeconds } = rateConfig(env);
  const row = await env.DB.prepare(
    "SELECT count, window_start FROM login_attempts WHERE key = ?",
  )
    .bind(key)
    .first<Record<string, string | number>>();

  if (!row) return false;

  const count = Number(row.count ?? 0);
  const windowStart = new Date(String(row.window_start));
  const windowEnd = windowStart.getTime() + windowSeconds * 1000;
  if (Date.now() > windowEnd) {
    await env.DB.prepare("DELETE FROM login_attempts WHERE key = ?")
      .bind(key)
      .run();
    return false;
  }

  return count >= limit;
}

async function recordLoginAttempt(env: Env, key: string): Promise<void> {
  const { windowSeconds } = rateConfig(env);
  const row = await env.DB.prepare(
    "SELECT count, window_start FROM login_attempts WHERE key = ?",
  )
    .bind(key)
    .first<Record<string, string | number>>();

  const now = new Date();
  if (!row) {
    await env.DB.prepare(
      "INSERT INTO login_attempts (key, count, window_start) VALUES (?, ?, ?)",
    )
      .bind(key, 1, now.toISOString())
      .run();
    return;
  }

  const windowStart = new Date(String(row.window_start));
  const windowEnd = windowStart.getTime() + windowSeconds * 1000;
  if (Date.now() > windowEnd) {
    await env.DB.prepare(
      "UPDATE login_attempts SET count = ?, window_start = ? WHERE key = ?",
    )
      .bind(1, now.toISOString(), key)
      .run();
    return;
  }

  await env.DB.prepare(
    "UPDATE login_attempts SET count = count + 1 WHERE key = ?",
  )
    .bind(key)
    .run();
}

async function clearLoginAttempts(env: Env, key: string): Promise<void> {
  await env.DB.prepare("DELETE FROM login_attempts WHERE key = ?")
    .bind(key)
    .run();
}

async function expireHostSessions(
  env: Env,
  userId: string,
  host: string,
): Promise<void> {
  const now = new Date().toISOString();
  await env.DB.prepare(
    "UPDATE sessions SET expires_at = ? WHERE user_id = ? AND host = ?",
  )
    .bind(now, userId, host)
    .run();
}

async function pruneSessions(
  env: Env,
  userId: string,
  limit: number,
): Promise<void> {
  const rows = await env.DB.prepare(
    `SELECT token
     FROM sessions
     WHERE user_id = ?
     ORDER BY created_at DESC`,
  )
    .bind(userId)
    .all<Record<string, string>>();

  const tokens = rows.results.map((row) => row.token);
  if (tokens.length <= limit) return;
  const toDelete = tokens.slice(limit);
  if (toDelete.length === 0) return;
  const placeholders = toDelete.map(() => "?").join(", ");
  await env.DB.prepare(`DELETE FROM sessions WHERE token IN (${placeholders})`)
    .bind(...toDelete)
    .run();
}

async function handleRegister(request: Request, env: Env): Promise<Response> {
  try {
    const body = await request.json<RegisterRequest>().catch(() => null);
    const email = normalizeEmail(body?.email);
    if (!body || !email || !body.password) {
      return json({ error: "invalid_request" }, 400);
    }

    const existing = await env.DB.prepare(
      "SELECT id FROM users WHERE email = ?",
    )
      .bind(email)
      .first<Record<string, string>>();

    if (existing) {
      return json({ error: "email_exists" }, 409);
    }

    const { hash, salt, iters } = await hashPassword(body.password);
    const userId = `usr_${crypto.randomUUID()}`;

    await env.DB.prepare(
      "INSERT INTO users (id, email, password_hash, password_salt, password_iters, created_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
      .bind(userId, email, hash, salt, iters, new Date().toISOString())
      .run();

    const response: RegisterResponse = { user_id: userId };
    return json(response, 200);
  } catch (err) {
    console.error("register_error", err);
    return json({ error: "internal_error", detail: String(err) }, 500);
  }
}

async function handleGetSessions(userId: string, env: Env): Promise<Response> {
  const rows = await env.DB.prepare(
    `SELECT host, created_at, expires_at, token_suffix
     FROM sessions
     WHERE user_id = ?
     ORDER BY created_at DESC`,
  )
    .bind(userId)
    .all<Record<string, string>>();

  const sessions: SessionInfo[] = (rows.results ?? []).map((row) => {
    const suffix = row.token_suffix ?? "";
    const expiresAt = new Date(row.expires_at ?? "");
    const active =
      Number.isFinite(expiresAt.getTime()) && expiresAt > new Date();
    return {
      host: row.host ?? "unknown",
      created_at: row.created_at ?? "",
      expires_at: row.expires_at ?? "",
      token_suffix: suffix,
      active,
    };
  });

  return json(sessions, 200);
}

async function handleCreateRepo(
  request: Request,
  env: Env,
  userId: string,
): Promise<Response> {
  const body = await request.json<RepoCreateRequest>().catch(() => null);
  if (!body || !body.name) {
    return json({ error: "invalid_request" }, 400);
  }
  if (!isValidRepoName(body.name)) {
    return json({ error: "invalid_repo_name" }, 400);
  }

  const existing = await env.DB.prepare(
    "SELECT id FROM repos WHERE owner_user_id = ? AND name = ?",
  )
    .bind(userId, body.name)
    .first<Record<string, string>>();

  if (existing) {
    return json({ error: "repo_exists" }, 409);
  }

  const repoId = `repo_${crypto.randomUUID()}`;
  const now = new Date().toISOString();

  const manifest: RepoManifest = {
    version: 1,
    repo_id: repoId,
    repo_name: body.name,
    active_branch: "dev",
    branches: [{ name: "dev", files: [] }],
  };

  await env.DB.prepare(
    "INSERT INTO repos (id, owner_user_id, name, manifest_json, created_at) VALUES (?, ?, ?, ?, ?)",
  )
    .bind(repoId, userId, body.name, JSON.stringify(manifest), now)
    .run();

  await env.DB.prepare(
    "INSERT INTO repo_links (user_id, repo_id, last_seen) VALUES (?, ?, ?)",
  )
    .bind(userId, repoId, now)
    .run();

  const response: RepoResponse = { repo_id: repoId, name: body.name };
  return json(response, 200);
}

async function handleGetRepo(
  request: Request,
  env: Env,
  userId: string,
): Promise<Response> {
  const url = new URL(request.url);
  const name = url.searchParams.get("name");
  if (!name) {
    return json({ error: "missing_name" }, 400);
  }

  const row = await env.DB.prepare(
    `SELECT repos.id as id, repos.name as name
     FROM repos
     LEFT JOIN repo_access ON repo_access.repo_id = repos.id AND repo_access.user_id = ?
     WHERE repos.name = ?
       AND (repos.owner_user_id = ? OR repo_access.user_id = ?)
     LIMIT 1`,
  )
    .bind(userId, name, userId, userId)
    .first<Record<string, string>>();

  if (!row) {
    return json({ error: "not_found" }, 404);
  }

  return json({ repo_id: row.id, name: row.name }, 200);
}

async function handleDeleteRepo(
  env: Env,
  userId: string,
  repoId: string,
): Promise<Response> {
  const repo = await ensureRepoOwned(env, userId, repoId);
  if (!repo) return json({ error: "repo_not_found" }, 404);

  await env.DB.prepare("DELETE FROM env_objects WHERE repo_id = ?")
    .bind(repoId)
    .run();
  await env.DB.prepare("DELETE FROM repo_keys WHERE repo_id = ?")
    .bind(repoId)
    .run();
  await env.DB.prepare("DELETE FROM repo_access WHERE repo_id = ?")
    .bind(repoId)
    .run();
  await env.DB.prepare("DELETE FROM repo_invites WHERE repo_id = ?")
    .bind(repoId)
    .run();
  await env.DB.prepare("DELETE FROM repo_links WHERE repo_id = ?")
    .bind(repoId)
    .run();
  await env.DB.prepare("DELETE FROM repos WHERE id = ?")
    .bind(repoId)
    .run();

  return json({ ok: true }, 200);
}

async function handleGetManifest(
  env: Env,
  userId: string,
  repoId: string,
): Promise<Response> {
  const repo = await ensureRepoAccess(env, userId, repoId, "read");
  if (!repo) {
    return json({ error: "repo_not_found" }, 404);
  }

  const row = await env.DB.prepare(
    "SELECT manifest_json FROM repos WHERE id = ?",
  )
    .bind(repoId)
    .first<Record<string, string>>();

  if (!row || !row.manifest_json) {
    return json({ error: "not_found" }, 404);
  }

  return json(JSON.parse(row.manifest_json), 200);
}

async function handlePutManifest(
  request: Request,
  env: Env,
  userId: string,
  repoId: string,
): Promise<Response> {
  const repo = await ensureRepoAccess(env, userId, repoId, "write");
  if (!repo) {
    return json({ error: "repo_not_found" }, 404);
  }

  const body = await request.json<RepoManifest>().catch(() => null);
  if (!body || !body.repo_id || !body.repo_name) {
    return json({ error: "invalid_request" }, 400);
  }
  if (body.repo_id !== repoId) {
    return json({ error: "repo_id_mismatch" }, 400);
  }

  await env.DB.prepare("UPDATE repos SET manifest_json = ? WHERE id = ?")
    .bind(JSON.stringify(body), repoId)
    .run();

  return json({ ok: true }, 200);
}

async function handleGetUmk(userId: string, env: Env): Promise<Response> {
  const row = await env.DB.prepare(
    "SELECT encrypted_umk, kdf_params, version, updated_at FROM umk_blobs WHERE user_id = ?",
  )
    .bind(userId)
    .first<Record<string, string | number>>();

  if (!row) {
    return json({ error: "not_found" }, 404);
  }

  const response: UmkResponse = {
    encrypted_umk: row.encrypted_umk as string,
    kdf_params: JSON.parse(row.kdf_params as string),
    version: Number(row.version),
    updated_at: row.updated_at as string,
  };

  return json(response, 200);
}

async function handlePutUmk(
  request: Request,
  userId: string,
  env: Env,
): Promise<Response> {
  const body = await request.json<UmkRequest>().catch(() => null);
  if (!body || !body.encrypted_umk || !body.kdf_params || !body.version) {
    return json({ error: "invalid_request" }, 400);
  }

  const now = new Date().toISOString();
  await env.DB.prepare(
    `INSERT INTO umk_blobs (user_id, encrypted_umk, kdf_params, version, updated_at)
     VALUES (?, ?, ?, ?, ?)
     ON CONFLICT(user_id) DO UPDATE SET
       encrypted_umk = excluded.encrypted_umk,
       kdf_params = excluded.kdf_params,
       version = excluded.version,
       updated_at = excluded.updated_at`,
  )
    .bind(
      userId,
      body.encrypted_umk,
      JSON.stringify(body.kdf_params),
      body.version,
      now,
    )
    .run();

  return json({ ok: true }, 200);
}

async function handleGetUserKey(
  userId: string,
  env: Env,
): Promise<Response> {
  const row = await env.DB.prepare(
    "SELECT public_key, algorithm, created_at, updated_at FROM user_keys WHERE user_id = ?",
  )
    .bind(userId)
    .first<Record<string, string>>();

  if (!row) {
    return json({ error: "not_found" }, 404);
  }

  const response: UserKeyResponse = {
    public_key: row.public_key,
    algorithm: row.algorithm,
    created_at: row.created_at,
    updated_at: row.updated_at,
  };
  return json(response, 200);
}

async function handlePutUserKey(
  request: Request,
  userId: string,
  env: Env,
): Promise<Response> {
  const body = await request.json<UserKeyRequest>().catch(() => null);
  if (!body || !body.public_key || !body.algorithm) {
    return json({ error: "invalid_request" }, 400);
  }

  const now = new Date().toISOString();
  await env.DB.prepare(
    `INSERT INTO user_keys (user_id, public_key, algorithm, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?)
     ON CONFLICT(user_id) DO UPDATE SET
       public_key = excluded.public_key,
       algorithm = excluded.algorithm,
       updated_at = excluded.updated_at`,
  )
    .bind(userId, body.public_key, body.algorithm, now, now)
    .run();

  return json({ ok: true }, 200);
}

async function handleGetRepoKey(
  env: Env,
  userId: string,
  repoId: string,
): Promise<Response> {
  const repo = await ensureRepoAccess(env, userId, repoId, "read");
  if (!repo) {
    return json({ error: "repo_not_found" }, 404);
  }

  const row = await env.DB.prepare(
    `SELECT wrapped_key, algorithm, created_at, updated_at
     FROM repo_keys
     WHERE repo_id = ? AND user_id = ?`,
  )
    .bind(repoId, userId)
    .first<Record<string, string>>();

  if (!row) {
    return json({ error: "not_found" }, 404);
  }

  const response: RepoKeyResponse = {
    wrapped_key: row.wrapped_key,
    algorithm: row.algorithm,
    created_at: row.created_at,
    updated_at: row.updated_at,
  };
  return json(response, 200);
}

async function handlePutRepoKey(
  request: Request,
  env: Env,
  userId: string,
  repoId: string,
): Promise<Response> {
  const body = await request.json<RepoKeyRequest>().catch(() => null);
  if (!body || !body.wrapped_key || !body.algorithm) {
    return json({ error: "invalid_request" }, 400);
  }

  const requesterEmail = await getUserEmail(env, userId);
  const targetEmail = normalizeEmail(body.email) ?? requesterEmail;
  if (!targetEmail) return json({ error: "invalid_email" }, 400);

  if (targetEmail !== requesterEmail) {
    const owned = await ensureRepoOwned(env, userId, repoId);
    if (!owned) return json({ error: "repo_not_found" }, 404);
  } else {
    const repo = await ensureRepoAccess(env, userId, repoId, "read");
    if (!repo) return json({ error: "repo_not_found" }, 404);
  }

  const userRow = await env.DB.prepare("SELECT id FROM users WHERE email = ?")
    .bind(targetEmail)
    .first<Record<string, string>>();
  if (!userRow) return json({ error: "user_not_found" }, 404);

  const now = new Date().toISOString();
  await env.DB.prepare(
    `INSERT INTO repo_keys (repo_id, user_id, wrapped_key, algorithm, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?)
     ON CONFLICT(repo_id, user_id) DO UPDATE SET
       wrapped_key = excluded.wrapped_key,
       algorithm = excluded.algorithm,
       updated_at = excluded.updated_at`,
  )
    .bind(repoId, userRow.id, body.wrapped_key, body.algorithm, now, now)
    .run();

  await env.DB.prepare(
    `INSERT INTO repo_key_events (id, repo_id, requester_user_id, requester_email, target_user_id, target_email, action, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?)`,
  )
    .bind(
      `rke_${crypto.randomUUID()}`,
      repoId,
      userId,
      requesterEmail ?? "",
      userRow.id,
      targetEmail,
      "upsert",
      now,
    )
    .run();

  return json({ ok: true }, 200);
}

async function handlePostObject(
  request: Request,
  env: Env,
  userId: string,
  repoId: string,
  branch: string,
): Promise<Response> {
  const repo = await ensureRepoAccess(env, userId, repoId, "write");
  if (!repo) {
    return json({ error: "repo_not_found" }, 404);
  }

  const body = await request.json<ObjectRequest>().catch(() => null);
  if (
    !body ||
    !body.path ||
    !body.nonce ||
    !body.ciphertext ||
    !body.aad ||
    !body.ciphertext_hash
  ) {
    return json({ error: "invalid_request" }, 400);
  }

  if (!isValidEnvPath(body.path)) {
    return json({ error: "invalid_path" }, 400);
  }

  if (body.path.includes("..") || body.path.startsWith("/")) {
    return json({ error: "invalid_path" }, 400);
  }

  let newSize = 0;
  try {
    newSize = base64ToBytes(body.ciphertext).length;
  } catch {
    return json({ error: "invalid_ciphertext" }, 400);
  }
  const currentSize = await latestObjectSize(env, repoId, branch, body.path);
  const totalLatest = await totalLatestSize(env, repoId);
  const nextTotal = totalLatest - currentSize + newSize;

  if (nextTotal > MAX_REPO_BYTES) {
    return json({ error: "repo_size_exceeded" }, 413);
  }

  const id = crypto.randomUUID();
  const createdAt = new Date().toISOString();
  const nextVersion = await nextVersionFor(env, repoId, branch, body.path);

  await env.DB.prepare(
    `INSERT INTO env_objects
     (id, repo_id, branch, path, nonce, ciphertext, aad, ciphertext_hash, version, created_at, client_created_at, schema_version)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
  )
    .bind(
      id,
      repoId,
      branch,
      body.path,
      body.nonce,
      body.ciphertext,
      body.aad,
      body.ciphertext_hash,
      nextVersion,
      createdAt,
      body.created_at ?? null,
      body.schema_version,
    )
    .run();

  await pruneHistory(env, repoId, branch, body.path, nextVersion);

  await env.DB.prepare(
    `INSERT INTO repo_links (user_id, repo_id, last_seen)
     VALUES (?, ?, ?)
     ON CONFLICT(user_id, repo_id) DO UPDATE SET last_seen = excluded.last_seen`,
  )
    .bind(userId, repoId, createdAt)
    .run();

  return json(
    {
      path: body.path,
      nonce: body.nonce,
      ciphertext: body.ciphertext,
      aad: body.aad,
      ciphertext_hash: body.ciphertext_hash,
      version: nextVersion,
      created_at: createdAt,
      schema_version: body.schema_version,
    },
    200,
  );
}

async function handleGetLatest(
  env: Env,
  _userId: string,
  repoId: string,
  branch: string,
  path: string,
): Promise<Response> {
  const repo = await ensureRepoAccess(env, _userId, repoId, "read");
  if (!repo) {
    return json({ error: "repo_not_found" }, 404);
  }
  if (!isValidEnvPath(path)) {
    return json({ error: "invalid_path" }, 400);
  }

  const row = await env.DB.prepare(
    `SELECT path, nonce, ciphertext, aad, ciphertext_hash, version, created_at, schema_version
     FROM env_objects
     WHERE repo_id = ? AND branch = ? AND path = ?
     ORDER BY created_at DESC
     LIMIT 1`,
  )
    .bind(repoId, branch, path)
    .first<Record<string, string | number>>();

  if (!row) {
    return json({ error: "not_found" }, 404);
  }

  return json(
    {
      path: row.path,
      nonce: row.nonce,
      ciphertext: row.ciphertext,
      aad: row.aad,
      ciphertext_hash: row.ciphertext_hash,
      version: Number(row.version),
      created_at: row.created_at,
      schema_version: Number(row.schema_version),
    },
    200,
  );
}

async function handleHistory(
  env: Env,
  userId: string,
  repoId: string,
  branch: string,
  path: string,
): Promise<Response> {
  const repo = await ensureRepoAccess(env, userId, repoId, "read");
  if (!repo) return json({ error: "repo_not_found" }, 404);
  if (!isValidEnvPath(path)) return json({ error: "invalid_path" }, 400);

  const rows = await env.DB.prepare(
    `SELECT version, created_at, ciphertext_hash
     FROM env_objects
     WHERE repo_id = ? AND branch = ? AND path = ?
     ORDER BY version DESC`,
  )
    .bind(repoId, branch, path)
    .all<Record<string, string | number>>();

  const entries: HistoryEntry[] = rows.results.map((row) => ({
    version: Number(row.version),
    created_at: row.created_at as string,
    ciphertext_hash: row.ciphertext_hash as string,
  }));

  return json(entries, 200);
}

async function handleGetVersion(
  env: Env,
  userId: string,
  repoId: string,
  branch: string,
  path: string,
  version: number,
): Promise<Response> {
  const repo = await ensureRepoAccess(env, userId, repoId, "read");
  if (!repo) return json({ error: "repo_not_found" }, 404);
  if (!isValidEnvPath(path)) return json({ error: "invalid_path" }, 400);

  const row = await env.DB.prepare(
    `SELECT path, nonce, ciphertext, aad, ciphertext_hash, version, created_at, schema_version
     FROM env_objects
     WHERE repo_id = ? AND branch = ? AND path = ? AND version = ?
     LIMIT 1`,
  )
    .bind(repoId, branch, path, version)
    .first<Record<string, string | number>>();

  if (!row) return json({ error: "not_found" }, 404);

  return json(
    {
      path: row.path,
      nonce: row.nonce,
      ciphertext: row.ciphertext,
      aad: row.aad,
      ciphertext_hash: row.ciphertext_hash,
      version: Number(row.version),
      created_at: row.created_at,
      schema_version: Number(row.schema_version),
    },
    200,
  );
}

async function handleGetRepos(userId: string, env: Env): Promise<Response> {
  const rows = await env.DB.prepare(
    `SELECT repos.id as repo_id,
            repos.name as name,
            owners.email as owner_email,
            COALESCE(repo_access.role, CASE WHEN repos.owner_user_id = ? THEN 'owner' END) as role,
            COALESCE(repo_links.last_seen, repos.created_at) as last_seen
     FROM repos
     JOIN users as owners ON owners.id = repos.owner_user_id
     LEFT JOIN repo_access ON repo_access.repo_id = repos.id AND repo_access.user_id = ?
     LEFT JOIN repo_links ON repo_links.repo_id = repos.id AND repo_links.user_id = ?
     WHERE repos.owner_user_id = ? OR repo_access.user_id = ?
     ORDER BY last_seen DESC`,
  )
    .bind(userId, userId, userId, userId, userId)
    .all<Record<string, string>>();

  return json(
    {
      repos: rows.results.map((row) => ({
        repo_id: row.repo_id,
        name: row.name,
        last_seen: row.last_seen,
        owner_email: row.owner_email,
        access: row.role ?? "read",
      })),
    },
    200,
  );
}

async function handleGetAccess(
  env: Env,
  userId: string,
  repoId: string,
): Promise<Response> {
  const repo = await ensureRepoOwned(env, userId, repoId);
  if (!repo) return json({ error: "repo_not_found" }, 404);

  const activeRows = await env.DB.prepare(
    `SELECT users.email as email,
            repo_access.role as role,
            repo_access.created_at as created_at,
            user_keys.public_key as public_key,
            user_keys.algorithm as key_algorithm
     FROM repo_access
     JOIN users ON users.id = repo_access.user_id
     LEFT JOIN user_keys ON user_keys.user_id = users.id
     WHERE repo_access.repo_id = ?
     ORDER BY users.email ASC`,
  )
    .bind(repoId)
    .all<Record<string, string>>();

  const pendingRows = await env.DB.prepare(
    `SELECT repo_invites.email as email, repo_invites.role as role, repo_invites.created_at as created_at,
            inviter.email as invited_by
     FROM repo_invites
     JOIN users as inviter ON inviter.id = repo_invites.invited_by_user_id
     WHERE repo_invites.repo_id = ? AND repo_invites.status = 'pending'
     ORDER BY repo_invites.created_at DESC`,
  )
    .bind(repoId)
    .all<Record<string, string>>();

  const entries: RepoAccessEntry[] = [
    ...activeRows.results.map((row) => ({
      email: row.email,
      role: row.role,
      status: "active" as const,
      created_at: row.created_at,
      public_key: row.public_key ?? undefined,
      key_algorithm: row.key_algorithm ?? undefined,
    })),
    ...pendingRows.results.map((row) => ({
      email: row.email,
      role: row.role,
      status: "pending" as const,
      invited_by: row.invited_by,
      created_at: row.created_at,
    })),
  ];

  return json({ entries }, 200);
}

async function handleInviteAccess(
  request: Request,
  env: Env,
  userId: string,
  repoId: string,
): Promise<Response> {
  const repo = await ensureRepoOwned(env, userId, repoId);
  if (!repo) return json({ error: "repo_not_found" }, 404);

  const body = await request
    .json<{ email?: string; role?: string }>()
    .catch(() => null);
  const email = normalizeEmail(body?.email);
  const role = body?.role;
  if (!email || !role) return json({ error: "invalid_request" }, 400);
  if (!isValidAccessRole(role)) return json({ error: "invalid_role" }, 400);

  const ownerEmail = await getUserEmail(env, userId);
  if (ownerEmail && ownerEmail === email) {
    return json({ error: "cannot_invite_owner" }, 400);
  }

  const access = await env.DB.prepare(
    "SELECT role FROM repo_access WHERE repo_id = ? AND user_id = (SELECT id FROM users WHERE email = ?)",
  )
    .bind(repoId, email)
    .first<Record<string, string>>();

  if (access) {
    return json({ error: "already_has_access" }, 409);
  }

  const now = new Date().toISOString();
  const existingInvite = await env.DB.prepare(
    "SELECT id FROM repo_invites WHERE repo_id = ? AND email = ? AND status = 'pending'",
  )
    .bind(repoId, email)
    .first<Record<string, string>>();

  if (existingInvite) {
    await env.DB.prepare(
      "UPDATE repo_invites SET role = ?, updated_at = ? WHERE id = ?",
    )
      .bind(role, now, existingInvite.id)
      .run();
    return json({ ok: true, invite_id: existingInvite.id }, 200);
  }

  const inviteId = `inv_${crypto.randomUUID()}`;
  await env.DB.prepare(
    `INSERT INTO repo_invites (id, repo_id, email, invited_by_user_id, role, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, 'pending', ?, ?)`,
  )
    .bind(inviteId, repoId, email, userId, role, now, now)
    .run();

  return json({ ok: true, invite_id: inviteId }, 200);
}

async function handleUpdateAccess(
  request: Request,
  env: Env,
  userId: string,
  repoId: string,
): Promise<Response> {
  const repo = await ensureRepoOwned(env, userId, repoId);
  if (!repo) return json({ error: "repo_not_found" }, 404);

  const body = await request
    .json<{ email?: string; role?: string }>()
    .catch(() => null);
  const email = normalizeEmail(body?.email);
  const role = body?.role;
  if (!email || !role) return json({ error: "invalid_request" }, 400);
  if (!isValidAccessRole(role)) return json({ error: "invalid_role" }, 400);

  const now = new Date().toISOString();
  const invite = await env.DB.prepare(
    "SELECT id FROM repo_invites WHERE repo_id = ? AND email = ? AND status = 'pending'",
  )
    .bind(repoId, email)
    .first<Record<string, string>>();

  if (invite) {
    await env.DB.prepare(
      "UPDATE repo_invites SET role = ?, updated_at = ? WHERE id = ?",
    )
      .bind(role, now, invite.id)
      .run();
    return json({ ok: true }, 200);
  }

  const userRow = await env.DB.prepare("SELECT id FROM users WHERE email = ?")
    .bind(email)
    .first<Record<string, string>>();
  if (!userRow) return json({ error: "user_not_found" }, 404);

  const access = await env.DB.prepare(
    "SELECT role FROM repo_access WHERE repo_id = ? AND user_id = ?",
  )
    .bind(repoId, userRow.id)
    .first<Record<string, string>>();
  if (!access) return json({ error: "access_not_found" }, 404);

  await env.DB.prepare(
    "UPDATE repo_access SET role = ?, updated_at = ? WHERE repo_id = ? AND user_id = ?",
  )
    .bind(role, now, repoId, userRow.id)
    .run();

  return json({ ok: true }, 200);
}

async function handleRevokeAccess(
  env: Env,
  userId: string,
  repoId: string,
  emailRaw: string,
): Promise<Response> {
  const repo = await ensureRepoOwned(env, userId, repoId);
  if (!repo) return json({ error: "repo_not_found" }, 404);

  const email = normalizeEmail(emailRaw);
  if (!email) return json({ error: "invalid_email" }, 400);

  const now = new Date().toISOString();
  const invite = await env.DB.prepare(
    "SELECT id FROM repo_invites WHERE repo_id = ? AND email = ? AND status = 'pending'",
  )
    .bind(repoId, email)
    .first<Record<string, string>>();

  if (invite) {
    await env.DB.prepare(
      "UPDATE repo_invites SET status = 'revoked', updated_at = ? WHERE id = ?",
    )
      .bind(now, invite.id)
      .run();
    return json({ ok: true }, 200);
  }

  const userRow = await env.DB.prepare("SELECT id FROM users WHERE email = ?")
    .bind(email)
    .first<Record<string, string>>();
  if (!userRow) return json({ error: "user_not_found" }, 404);

  const access = await env.DB.prepare(
    "SELECT role FROM repo_access WHERE repo_id = ? AND user_id = ?",
  )
    .bind(repoId, userRow.id)
    .first<Record<string, string>>();
  if (!access) return json({ error: "access_not_found" }, 404);

  await env.DB.prepare(
    "DELETE FROM repo_access WHERE repo_id = ? AND user_id = ?",
  )
    .bind(repoId, userRow.id)
    .run();
  await env.DB.prepare("DELETE FROM repo_links WHERE repo_id = ? AND user_id = ?")
    .bind(repoId, userRow.id)
    .run();

  return json({ ok: true }, 200);
}

async function handleGetInvites(userId: string, env: Env): Promise<Response> {
  const email = await getUserEmail(env, userId);
  if (!email) return json([], 200);

  const rows = await env.DB.prepare(
    `SELECT repo_invites.id as id,
            repo_invites.repo_id as repo_id,
            repos.name as repo_name,
            repo_invites.role as role,
            repo_invites.created_at as created_at,
            inviter.email as invited_by
     FROM repo_invites
     JOIN repos ON repos.id = repo_invites.repo_id
     JOIN users as inviter ON inviter.id = repo_invites.invited_by_user_id
     WHERE repo_invites.email = ? AND repo_invites.status = 'pending'
     ORDER BY repo_invites.created_at DESC`,
  )
    .bind(email)
    .all<Record<string, string>>();

  const invites: InviteInfo[] = rows.results.map((row) => ({
    id: row.id,
    repo_id: row.repo_id,
    repo_name: row.repo_name,
    role: row.role,
    invited_by: row.invited_by,
    created_at: row.created_at,
  }));

  return json(invites, 200);
}

async function handleInviteAction(
  env: Env,
  userId: string,
  inviteId: string,
  action: string,
): Promise<Response> {
  const email = await getUserEmail(env, userId);
  if (!email) return json({ error: "invalid_user" }, 400);

  const invite = await env.DB.prepare(
    `SELECT id, repo_id, role
     FROM repo_invites
     WHERE id = ? AND email = ? AND status = 'pending'`,
  )
    .bind(inviteId, email)
    .first<Record<string, string>>();

  if (!invite) return json({ error: "invite_not_found" }, 404);

  const now = new Date().toISOString();
  if (action === "reject") {
    await env.DB.prepare(
      "UPDATE repo_invites SET status = 'rejected', updated_at = ? WHERE id = ?",
    )
      .bind(now, inviteId)
      .run();
    return json({ ok: true }, 200);
  }

  if (action !== "accept") {
    return json({ error: "invalid_action" }, 400);
  }

  await env.DB.prepare(
    `INSERT INTO repo_access (repo_id, user_id, role, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?)
     ON CONFLICT(repo_id, user_id) DO UPDATE SET role = excluded.role, updated_at = excluded.updated_at`,
  )
    .bind(invite.repo_id, userId, invite.role, now, now)
    .run();

  await env.DB.prepare(
    "UPDATE repo_invites SET status = 'accepted', updated_at = ? WHERE id = ?",
  )
    .bind(now, inviteId)
    .run();

  await env.DB.prepare(
    `INSERT INTO repo_links (user_id, repo_id, last_seen)
     VALUES (?, ?, ?)
     ON CONFLICT(user_id, repo_id) DO UPDATE SET last_seen = excluded.last_seen`,
  )
    .bind(userId, invite.repo_id, now)
    .run();

  return json({ ok: true }, 200);
}

async function handleListBranches(
  env: Env,
  userId: string,
  repoId: string,
): Promise<Response> {
  const repo = await ensureRepoAccess(env, userId, repoId, "read");
  if (!repo) {
    return json({ error: "repo_not_found" }, 404);
  }

  const rows = await env.DB.prepare(
    "SELECT DISTINCT branch FROM env_objects WHERE repo_id = ? ORDER BY branch ASC",
  )
    .bind(repoId)
    .all<Record<string, string>>();

  return json({ branches: rows.results.map((row) => row.branch) }, 200);
}

async function withAuth(
  request: Request,
  env: Env,
  handler: (userId: string) => Promise<Response>,
): Promise<Response> {
  const token = getBearerToken(request.headers.get("Authorization"));
  if (!token) {
    return json({ error: "missing_auth" }, 401);
  }
  const tokenHash = await hashToken(token);

  const row = await env.DB.prepare(
    "SELECT user_id, expires_at FROM sessions WHERE token = ?",
  )
    .bind(tokenHash)
    .first<Record<string, string>>();

  if (!row) {
    return json({ error: "invalid_token" }, 401);
  }

  const expires = new Date(row.expires_at);
  if (Number.isNaN(expires.getTime()) || expires < new Date()) {
    return json({ error: "token_expired" }, 401);
  }

  return handler(row.user_id);
}

function getBearerToken(header: string | null): string | null {
  if (!header) return null;
  const [scheme, token] = header.split(" ");
  if (scheme !== "Bearer" || !token) return null;
  return token.trim();
}

async function randomToken(): Promise<string> {
  const bytes = new Uint8Array(32);
  crypto.getRandomValues(bytes);
  return bytesToBase64(bytes);
}

async function hashToken(token: string): Promise<string> {
  const digest = await crypto.subtle.digest(
    "SHA-256",
    encoder.encode(token),
  );
  return bytesToBase64(new Uint8Array(digest));
}

async function hashPassword(
  password: string,
): Promise<{ hash: string; salt: string; iters: number }> {
  const saltBytes = new Uint8Array(16);
  crypto.getRandomValues(saltBytes);
  const iters = 100_000;
  const derived = await pbkdf2(password, saltBytes, iters, 32);
  return {
    hash: bytesToBase64(derived),
    salt: bytesToBase64(saltBytes),
    iters,
  };
}

async function verifyPassword(
  password: string,
  saltB64: string,
  iters: number,
  expectedB64: string,
): Promise<boolean> {
  const salt = base64ToBytes(saltB64);
  const derived = await pbkdf2(password, salt, iters, 32);
  return timingSafeEqual(bytesToBase64(derived), expectedB64);
}

async function pbkdf2(
  password: string,
  salt: Uint8Array,
  iters: number,
  length: number,
): Promise<Uint8Array> {
  const key = await crypto.subtle.importKey(
    "raw",
    encoder.encode(password),
    "PBKDF2",
    false,
    ["deriveBits"],
  );
  const bits = await crypto.subtle.deriveBits(
    {
      name: "PBKDF2",
      hash: "SHA-256",
      salt,
      iterations: iters,
    },
    key,
    length * 8,
  );
  return new Uint8Array(bits);
}

function json(body: Json, status: number): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json" },
  });
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  for (const b of bytes) binary += String.fromCharCode(b);
  return btoa(binary);
}

function base64ToBytes(b64: string): Uint8Array {
  const binary = atob(b64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

function timingSafeEqual(a: string, b: string): boolean {
  if (a.length !== b.length) return false;
  let result = 0;
  for (let i = 0; i < a.length; i += 1) {
    result |= a.charCodeAt(i) ^ b.charCodeAt(i);
  }
  return result === 0;
}

function isValidEnvPath(path: string): boolean {
  if (!path || path.startsWith("/") || path.includes("\\")) return false;
  const parts = path.split("/");
  if (parts.some((part) => part === "" || part === "." || part === "..")) {
    return false;
  }
  const filename = parts[parts.length - 1];
  return filename === ".env" || filename.startsWith(".env.");
}

function isValidRepoName(name: string): boolean {
  return /^[A-Za-z0-9_-]+$/.test(name);
}

function normalizeEmail(email?: string | null): string | null {
  if (!email) return null;
  const trimmed = email.trim().toLowerCase();
  return trimmed.length > 0 ? trimmed : null;
}

function isValidAccessRole(role: string): boolean {
  return role === "read" || role === "write";
}

async function getUserEmail(env: Env, userId: string): Promise<string | null> {
  const row = await env.DB.prepare("SELECT email FROM users WHERE id = ?")
    .bind(userId)
    .first<Record<string, string>>();
  return row?.email ?? null;
}

async function ensureRepoOwned(
  env: Env,
  userId: string,
  repoId: string,
): Promise<{ id: string; name: string } | null> {
  const row = await env.DB.prepare(
    "SELECT id, name FROM repos WHERE id = ? AND owner_user_id = ?",
  )
    .bind(repoId, userId)
    .first<Record<string, string>>();

  if (!row) return null;
  return { id: row.id, name: row.name };
}

async function ensureRepoAccess(
  env: Env,
  userId: string,
  repoId: string,
  required: "read" | "write",
): Promise<{ id: string; name: string; role: string } | null> {
  const repo = await env.DB.prepare(
    "SELECT id, name, owner_user_id FROM repos WHERE id = ?",
  )
    .bind(repoId)
    .first<Record<string, string>>();
  if (!repo) return null;

  if (repo.owner_user_id === userId) {
    return { id: repo.id, name: repo.name, role: "owner" };
  }

  const access = await env.DB.prepare(
    "SELECT role FROM repo_access WHERE repo_id = ? AND user_id = ?",
  )
    .bind(repoId, userId)
    .first<Record<string, string>>();

  if (!access) return null;
  if (required === "write" && access.role !== "write") {
    return null;
  }
  return { id: repo.id, name: repo.name, role: access.role };
}

async function latestObjectSize(
  env: Env,
  repoId: string,
  branch: string,
  path: string,
): Promise<number> {
  const row = await env.DB.prepare(
    `SELECT length(ciphertext) as size
     FROM env_objects
     WHERE repo_id = ? AND branch = ? AND path = ?
     ORDER BY created_at DESC
     LIMIT 1`,
  )
    .bind(repoId, branch, path)
    .first<Record<string, number>>();

  return row?.size ?? 0;
}

async function totalLatestSize(env: Env, repoId: string): Promise<number> {
  const row = await env.DB.prepare(
    `WITH latest AS (
       SELECT ciphertext,
              ROW_NUMBER() OVER (PARTITION BY repo_id, branch, path ORDER BY created_at DESC) AS rn
       FROM env_objects
       WHERE repo_id = ?
     )
     SELECT COALESCE(SUM(length(ciphertext)), 0) AS total
     FROM latest
     WHERE rn = 1`,
  )
    .bind(repoId)
    .first<Record<string, number>>();

  return row?.total ?? 0;
}

async function nextVersionFor(
  env: Env,
  repoId: string,
  branch: string,
  path: string,
): Promise<number> {
  const row = await env.DB.prepare(
    `SELECT MAX(version) as max_version
     FROM env_objects
     WHERE repo_id = ? AND branch = ? AND path = ?`,
  )
    .bind(repoId, branch, path)
    .first<Record<string, number>>();

  const maxVersion = Number(row?.max_version ?? 0);
  return maxVersion + 1;
}

async function pruneHistory(
  env: Env,
  repoId: string,
  branch: string,
  path: string,
  latestVersion: number,
): Promise<void> {
  const limit = 12;
  const minVersion = latestVersion - (limit - 1);
  if (minVersion <= 0) return;
  await env.DB.prepare(
    `DELETE FROM env_objects
     WHERE repo_id = ? AND branch = ? AND path = ? AND version < ?`,
  )
    .bind(repoId, branch, path, minVersion)
    .run();
}
