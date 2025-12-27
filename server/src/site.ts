import { DOCS_HTML } from "./docs";
import { DEMO_MP4_BASE64 } from "./demo";
import {
  ANDROID_CHROME_192_BASE64,
  ANDROID_CHROME_512_BASE64,
  APPLE_TOUCH_ICON_BASE64,
  FAVICON_16_BASE64,
  FAVICON_32_BASE64,
  FAVICON_ICO_BASE64,
} from "./favicons";

export async function handleSiteRequest(request: Request): Promise<Response> {
  const url = new URL(request.url);
  const pathname = url.pathname;

  if (pathname === "/" && request.method === "GET") {
    return serveLanding();
  }
  if (
    (pathname === "/docs" || pathname === "/docs/") &&
    request.method === "GET"
  ) {
    return serveDocs();
  }
  if (pathname === "/demo.mp4" && request.method === "GET") {
    return serveDemo();
  }
  if (pathname === "/favicon.ico" && request.method === "GET") {
    return serveFavicon(FAVICON_ICO_BASE64);
  }
  if (pathname === "/favicon-16x16.png" && request.method === "GET") {
    return servePng(FAVICON_16_BASE64);
  }
  if (pathname === "/favicon-32x32.png" && request.method === "GET") {
    return servePng(FAVICON_32_BASE64);
  }
  if (pathname === "/apple-touch-icon.png" && request.method === "GET") {
    return servePng(APPLE_TOUCH_ICON_BASE64);
  }
  if (pathname === "/android-chrome-192x192.png" && request.method === "GET") {
    return servePng(ANDROID_CHROME_192_BASE64);
  }
  if (pathname === "/android-chrome-512x512.png" && request.method === "GET") {
    return servePng(ANDROID_CHROME_512_BASE64);
  }

  return new Response("not_found", { status: 404 });
}

function serveLanding(): Response {
  const GITHUB_SVG = `<svg class="icon" viewBox="0 0 24 24" role="img" aria-hidden="true">
    <path d="M12 .5C5.73.5.75 5.48.75 11.75c0 4.87 3.16 9 7.55 10.46.55.1.75-.24.75-.53v-2.03c-3.07.67-3.72-1.28-3.72-1.28-.5-1.27-1.22-1.6-1.22-1.6-1-.68.07-.67.07-.67 1.1.08 1.68 1.13 1.68 1.13.99 1.69 2.59 1.2 3.22.92.1-.72.38-1.2.69-1.48-2.45-.28-5.02-1.22-5.02-5.45 0-1.2.43-2.18 1.13-2.95-.11-.28-.49-1.41.1-2.94 0 0 .92-.3 3.01 1.13.87-.24 1.8-.36 2.73-.36.93 0 1.86.12 2.73.36 2.1-1.43 3.01-1.13 3.01-1.13.59 1.53.21 2.66.1 2.94.7.77 1.13 1.75 1.13 2.95 0 4.24-2.58 5.17-5.04 5.45.39.34.73 1 .73 2.02v2.99c0 .29.2.63.75.53 4.39-1.46 7.55-5.59 7.55-10.46C23.25 5.48 18.27.5 12 .5z"/>
  </svg>`;
  const BOOK_SVG = `<svg class="icon" viewBox="0 0 24 24" role="img" aria-hidden="true">
    <path d="M5 3.5C5 2.67 5.67 2 6.5 2H20a1 1 0 0 1 1 1v15.5a2.5 2.5 0 0 1-2.5 2.5H6.5A2.5 2.5 0 0 0 4 23V5.5C4 4.12 5.12 3 6.5 3H20v2H6.5C6.22 5 6 5.22 6 5.5V19a3.49 3.49 0 0 1 0-7H20V4H6.5A1.5 1.5 0 0 0 5 5.5V3.5z"/>
  </svg>`;
  const X_SVG = `<svg class="icon" viewBox="0 0 24 24" role="img" aria-hidden="true">
    <path d="M18.9 2H22l-7.1 8.1L23 22h-6.5l-5.1-6.8L5.6 22H2l7.6-8.7L1 2h6.6l4.6 6.1L18.9 2Zm-1.2 18h1.8L6.4 4H4.4l13.3 16Z"/>
  </svg>`;

  const html = `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <meta name="theme-color" content="#1e1e2e" />
  <meta name="color-scheme" content="dark" />
  <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent" />
  <link rel="icon" href="/favicon.ico" sizes="any" />
  <link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png" />
  <link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png" />
  <link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png" />
  <title>milieu</title>
  <style>
    :root {
      --crust: #11111b;
      --mantle: #181825;
      --base: #1e1e2e;
      --surface-0: #313244;
      --surface-1: #45475a;
      --surface-2: #585b70;
      --text: #cdd6f4;
      --subtext: #a6adc8;
      --lavender: #b4befe;
      --mauve: #cba6f7;
      --blue: #89b4fa;
      --teal: #94e2d5;
      --green: #a6e3a1;
      --yellow: #f9e2af;
      --peach: #fab387;
      --red: #f38ba8;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
      background: radial-gradient(1200px 800px at 20% 10%, #1b1b2a 0%, var(--base) 40%, var(--mantle) 100%);
      color: var(--text);
      min-height: 100vh;
      padding: 32px 20px 80px;
      overflow-x: hidden;
    }
    a { color: var(--blue); text-decoration: none; }
    a:hover { text-decoration: underline; }
    .icon-link {
      display: inline-flex;
      align-items: center;
      gap: 10px;
      color: var(--text);
      text-decoration: none;
    }
    .icon-link:hover { color: var(--lavender); }
    .icon {
      width: 22px;
      height: 22px;
      fill: currentColor;
    }
    .container {
      width: min(1100px, 100%);
      margin: 0 auto;
      display: flex;
      flex-direction: column;
      gap: 40px;
    }
    .hero {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
      gap: 28px;
      align-items: center;
    }
    .hero h1 {
      margin: 0;
      font-size: clamp(28px, 4vw, 40px);
      color: var(--lavender);
      letter-spacing: 0.04em;
    }
    .hero-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 16px;
      flex-wrap: wrap;
      margin-bottom: 12px;
    }
    .topbar {
      display: flex;
      align-items: center;
      gap: 10px;
    }
    .topbar a {
      display: inline-flex;
      align-items: center;
      gap: 8px;
      padding: 8px 12px;
      border-radius: 999px;
      border: 1px solid var(--surface-1);
      background: rgba(17,17,27,0.7);
      color: var(--text);
      font-size: 13px;
      text-decoration: none;
      white-space: nowrap;
    }
    .topbar a:hover {
      color: var(--lavender);
      border-color: var(--surface-2);
    }
    .topbar .icon {
      width: 16px;
      height: 16px;
      fill: currentColor;
    }
    .hero p {
      margin: 0 0 16px 0;
      color: var(--subtext);
      line-height: 1.6;
    }
    .badge-row {
      display: flex;
      flex-wrap: wrap;
      gap: 10px;
      margin: 16px 0 20px;
    }
    .badge-row a {
      display: inline-flex;
      align-items: center;
    }
    .badge-row img {
      height: 22px;
    }
    .callout {
      border: 1px solid rgba(243, 139, 168, 0.6);
      background: rgba(243, 139, 168, 0.1);
      padding: 12px 16px;
      border-radius: 12px;
      color: var(--text);
      font-size: 14px;
    }
    .terminal {
      background: var(--crust);
      border: 1px solid var(--surface-1);
      border-radius: 16px;
      padding: 20px;
      box-shadow: 0 20px 60px rgba(0,0,0,0.35);
      position: relative;
    }
    .terminal::before {
      content: "";
      position: absolute;
      top: 14px;
      left: 16px;
      width: 10px;
      height: 10px;
      border-radius: 50%;
      background: var(--red);
      box-shadow: 18px 0 0 var(--peach), 36px 0 0 var(--green);
    }
    .terminal pre {
      margin: 0;
      padding-top: 24px;
      white-space: pre-wrap;
      line-height: 1.6;
    }
    .prompt { color: var(--mauve); }
    .cmd { color: var(--blue); }
    .ok { color: var(--green); }
    .note { color: var(--teal); }
    .muted { color: var(--subtext); }
    .section {
      background: linear-gradient(180deg, rgba(49,50,68,0.7), rgba(30,30,46,0.85));
      border: 1px solid var(--surface-1);
      border-radius: 18px;
      padding: 24px;
    }
    .section h2 {
      margin: 0 0 12px 0;
      color: var(--mauve);
      font-size: 22px;
    }
    .section p {
      color: var(--subtext);
      margin: 0 0 12px 0;
      line-height: 1.6;
    }
    .demo-section .card {
      display: block;
      margin: 0 auto;
      max-width: 820px;
    }
    .demo-video {
      width: 100%;
      max-width: 100%;
      border-radius: 12px;
      border: 1px solid var(--surface-1);
      background: var(--crust);
      display: block;
    }
    .logo-title {
      display: inline-flex;
      align-items: center;
      gap: 12px;
    }
    .logo-title img {
      width: 32px;
      height: 32px;
    }
    .grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
      gap: 16px;
    }
    .card {
      background: rgba(17,17,27,0.7);
      border: 1px solid var(--surface-1);
      border-radius: 14px;
      padding: 16px;
      align-items: center;
    }
    .card h3 {
      margin: 0 0 8px 0;
      color: var(--lavender);
      font-size: 16px;
    }
    .pill-row {
      display: flex;
      flex-wrap: wrap;
      gap: 10px;
      margin-top: 12px;
    }
    .pill {
      border: 1px solid var(--surface-1);
      border-radius: 999px;
      padding: 6px 12px;
      background: rgba(49,50,68,0.6);
      color: var(--text);
      font-size: 12px;
    }
    .install pre {
      margin: 0;
      padding: 16px;
      border-radius: 12px;
      background: var(--crust);
      border: 1px solid var(--surface-0);
      color: var(--text);
    }
    .list {
      margin: 0;
      padding-left: 18px;
      color: var(--subtext);
      line-height: 1.6;
    }
    .mermaid {
      background: rgba(17,17,27,0.7);
      border: 1px solid var(--surface-1);
      border-radius: 12px;
      padding: 16px;
      overflow-x: auto;
    }
    .footer {
      text-align: center;
      color: var(--subtext);
      font-size: 12px;
      margin-top: 12px;
    }
    @keyframes blink {
      0%, 45% { opacity: 1; }
      46%, 100% { opacity: 0; }
    }
    .cursor {
      display: inline-block;
      width: 10px;
      height: 1em;
      background: var(--green);
      margin-left: 4px;
      animation: blink 1s infinite;
      vertical-align: -0.1em;
    }
    @media (max-width: 640px) {
      body { padding: 24px 16px 60px; }
    }
  </style>
</head>
<body>
  <div class="container">
    <section class="hero">
      <div>
        <div class="hero-header">
          <h1 class="logo-title">
            <img src="/favicon-32x32.png" alt="milieu icon" />
            milieu
          </h1>
          <div class="topbar">
            <a href="/docs">${BOOK_SVG}Docs</a>
            <a href="https://github.com/citizenhicks/milieu">${GITHUB_SVG}GitHub</a>
            <a href="https://x.com/citizenhicks">${X_SVG}Twitter</a>
          </div>
        </div>
        <p>Git‑like dotenv sync with end‑to‑end encryption. Keep secrets out of git, keep them portable across machines.</p>
        <p><code>/docs</code> for the full CLI reference.</p>
        <div class="badge-row">
          <a href="https://github.com/citizenhicks/milieu/actions/workflows/ci.yml" aria-label="CI status">
            <img src="https://github.com/citizenhicks/milieu/actions/workflows/ci.yml/badge.svg" alt="CI status">
          </a>
          <a href="https://github.com/citizenhicks/milieu/releases" aria-label="Release">
            <img src="https://img.shields.io/github/v/release/citizenhicks/milieu?display_name=tag" alt="Release">
          </a>
          <a href="https://github.com/citizenhicks/homebrew-milieu" aria-label="Homebrew tap">
            <img src="https://img.shields.io/badge/homebrew-citizenhicks%2Fmilieu-orange?logo=homebrew" alt="Homebrew tap">
          </a>
        </div>
        <div class="callout">
          <strong>Beta:</strong> this project is in public beta. Use at your own risk.
        </div>
        <div class="pill-row">
          <span class="pill">E2EE by default</span>
          <span class="pill">Keychain-backed</span>
          <span class="pill">Open source</span>
          <span class="pill">.env* only</span>
        </div>
      </div>
      <div class="terminal">
        <pre><span class="prompt">citizenhicks@snowwhite</span> <span class="muted">~</span> <span class="prompt">❯</span> <span class="cmd">milieu status</span>
<span class="ok">SCOPE: repo milieu</span>
<span class="prompt">branch: *dev</span>
  File       Local   Remote  Diff
  ---------  ------  ------  --------------
  .env       v3(m)   v2      modified_local
  .env.prod  v1      v1      no_change

<span class="prompt">citizenhicks@snowwhite</span> <span class="muted">~</span> <span class="prompt">❯</span> <span class="cmd">milieu push</span>
<span class="ok">pushed .env (+2 -1)</span><span class="cursor"></span></pre>
      </div>
    </section>

    <section class="section demo-section">
      <h2>Demo</h2>
      <div class="card">
        <video class="demo-video" src="/demo.mp4" autoplay loop muted playsinline controls></video>
      </div>
    </section>

    <section class="section install">
      <h2>Install</h2>
      <p>Homebrew (macOS + Linux):</p>
      <pre><code>brew tap citizenhicks/milieu
brew install milieu</code></pre>
    </section>

    <section class="section">
      <h2>Features</h2>
      <div class="grid">
        <div class="card">
          <h3>Branch-scoped dotenvs</h3>
          <p>Multiple encrypted env sets per repo with git-like workflows: push, pull, status, and diff.</p>
        </div>
        <div class="card">
          <h3>History & rollback</h3>
          <p>Per-file version history with checkout and change inspection.</p>
        </div>
        <div class="card">
          <h3>Team access</h3>
          <p>Invite collaborators with read/write roles and audit sessions per device.</p>
        </div>
        <div class="card">
          <h3>Secure by default</h3>
          <p>Argon2id + XChaCha20-Poly1305, UMK stored in OS keychain, rate-limited logins.</p>
        </div>
        <div class="card">
          <h3>Self-hostable API</h3>
          <p>Cloudflare Worker + D1 backend, or point the CLI at your own base URL.</p>
        </div>
        <div class="card">
          <h3>Zero plaintext server</h3>
          <p>Only ciphertext and metadata are stored remotely.</p>
        </div>
      </div>
    </section>

    <section class="section">
      <h2>Why Milieu</h2>
      <div class="grid">
        <div class="card">
          <h3>Encrypted storage</h3>
          <p>Secrets are encrypted locally and only ciphertext hits the server. UMK is protected by the OS keychain.</p>
        </div>
        <div class="card">
          <h3>Git-like workflow</h3>
          <p>Branch‑scoped env sets. Push, pull, status, changes — feels like git, without committing secrets.</p>
        </div>
        <div class="card">
          <h3>Open source</h3>
          <p>
            <a class="icon-link" href="https://github.com/citizenhicks/milieu" aria-label="GitHub">
              ${GITHUB_SVG}
              citizenhicks/milieu
            </a>
          </p>
        </div>
      </div>
    </section>

    <section class="section">
      <h2>Facts & architecture</h2>
      <div class="grid">
        <div class="card">
          <h3>Crypto</h3>
          <p>Argon2id for passphrase KDF, XChaCha20‑Poly1305 for file encryption, AAD binds repo + branch + path.</p>
        </div>
        <div class="card">
          <h3>Command surface</h3>
          <p>Full CLI with branch‑scoped commands and repo‑wide status. See <a href="/docs">milieu.sh/docs</a>.</p>
        </div>
        <div class="card">
          <h3>Server</h3>
          <p>Cloudflare Worker + D1. Sessions + encrypted objects only. Easy to self‑host with your own base URL.</p>
        </div>
      </div>
    </section>

    <section class="section">
      <h2>Encryption flow</h2>
      <p>Milieu uses a per‑repo shared key wrapped for each collaborator with X25519 + HKDF. Files are encrypted with XChaCha20‑Poly1305 and bound to repo/branch/path via AAD.</p>
      <pre class="mermaid">
sequenceDiagram
  autonumber
  participant U as User
  participant C as Milieu CLI
  participant K as Keychain
  participant S as API
  participant D as D1

  rect rgba(203, 166, 247, 0.08)
  note over U,C: User onboarding
  U->>C: login / init / clone
  C->>C: derive UMK from recovery phrase
  C->>C: derive user keypair from UMK (X25519)
  C->>S: PUT /v1/users/me/key (public key)
  S->>D: store user_keys
  end

  rect rgba(166, 227, 161, 0.08)
  note over U,C: Repo bootstrap
  U->>C: init repo
  C->>C: generate repo key (32 bytes)
  C->>C: wrap repo key for owner (X25519 + HKDF)
  C->>S: PUT /v1/repos/:id/key (wrapped_key)
  S->>D: store repo_keys (per user)
  C->>K: store repo key locally
  end

  rect rgba(249, 226, 175, 0.08)
  note over U,C: Collaboration
  U->>C: invite collaborator
  C->>S: POST /v1/repos/:id/access
  S->>D: create invite
  U->>C: share repo key
  C->>S: GET /v1/repos/:id/access (public keys)
  C->>C: wrap repo key for each collaborator
  C->>S: PUT /v1/repos/:id/key (wrapped_key, email)
  S->>D: store repo_keys
  end

  rect rgba(148, 226, 213, 0.08)
  note over U,C: Key rotation
  U->>C: user rotate-keys
  C->>C: generate new phrase + UMK
  C->>S: PUT /v1/users/me/umk
  S->>D: store umk_blobs
  C->>S: PUT /v1/users/me/key (new public key)
  S->>D: update user_keys
  C->>S: PUT /v1/repos/:id/key (rewrapped keys)
  S->>D: update repo_keys
  end

  rect rgba(137, 180, 250, 0.08)
  note over U,C: Write path
  U->>C: push .env
  C->>C: aad = v2|repo|branch|path|tag
  C->>C: encrypt file with repo key (XChaCha20‑Poly1305)
  C->>S: POST /v1/repos/:id/branches/:b/objects
  S->>D: store ciphertext only
  end

  rect rgba(180, 190, 254, 0.08)
  note over U,C: Read path
  U->>C: pull .env
  C->>S: GET /v1/repos/:id/key (wrapped_key)
  C->>C: unwrap with private key (X25519 + HKDF)
  C->>C: decrypt file with repo key
  end
      </pre>
      <div class="notice">
        <strong>Recovery phrase collision risk:</strong> phrases are generated from 128 bits of entropy (12‑word BIP39), so the chance of two users ever getting the same phrase is about 1 in 2^128 (~3.4e38). Collisions are astronomically unlikely.
      </div>
    </section>

    <section class="section">
      <h2>Contribution</h2>
      <p>Pull requests welcome. If you want to work on CLI features, server behavior, or docs, open an issue first so we can align on scope.</p>
      <ul class="list">
        <li>Fork the repo and create a branch.</li>
        <li>Run the CLI locally and verify flows.</li>
        <li>Keep changes focused and add tests if needed.</li>
      </ul>
    </section>

      <div class="footer">
        milieu.sh · encrypted dotenv sync ·
        <a href="https://x.com/citizenhicks">x.com/citizenhicks</a>
      </div>
  </div>
  <script type="module">
    import mermaid from "https://unpkg.com/mermaid@10/dist/mermaid.esm.min.mjs";
    mermaid.initialize({ startOnLoad: true, theme: "dark" });
  </script>
</body>
</html>`;

  return new Response(html, {
    status: 200,
    headers: {
      "content-type": "text/html; charset=utf-8",
      "cache-control": "public, max-age=300",
    },
  });
}

function serveDocs(): Response {
  return new Response(DOCS_HTML, {
    status: 200,
    headers: {
      "content-type": "text/html; charset=utf-8",
      "cache-control": "public, max-age=300",
    },
  });
}

function serveDemo(): Response {
  const binary = atob(DEMO_MP4_BASE64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return new Response(bytes, {
    status: 200,
    headers: {
      "content-type": "video/mp4",
      "cache-control": "public, max-age=300",
    },
  });
}

function servePng(base64: string): Response {
  return new Response(base64ToBytes(base64), {
    status: 200,
    headers: {
      "content-type": "image/png",
      "cache-control": "public, max-age=604800",
    },
  });
}

function serveFavicon(base64: string): Response {
  return new Response(base64ToBytes(base64), {
    status: 200,
    headers: {
      "content-type": "image/x-icon",
      "cache-control": "public, max-age=604800",
    },
  });
}

function base64ToBytes(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}
