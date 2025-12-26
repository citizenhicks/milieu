import { execSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { resolve } from "node:path";

const repoRoot = resolve(import.meta.dirname, "..", "..");
const binPath = resolve(repoRoot, "target", "debug", "milieu");

const commands = [
  { name: "milieu", args: ["--help"], title: "Overview" },
  { name: "milieu register", args: ["register", "--help"], title: "register" },
  { name: "milieu login", args: ["login", "--help"], title: "login" },
  { name: "milieu logout", args: ["logout", "--help"], title: "logout" },
  { name: "milieu init", args: ["init", "--help"], title: "init" },
  { name: "milieu clone", args: ["clone", "--help"], title: "clone" },
  { name: "milieu repos", args: ["repos", "--help"], title: "repos" },
  { name: "milieu branch", args: ["branch", "--help"], title: "branch" },
  { name: "milieu add", args: ["add", "--help"], title: "add" },
  { name: "milieu remove", args: ["remove", "--help"], title: "remove" },
  { name: "milieu push", args: ["push", "--help"], title: "push" },
  { name: "milieu pull", args: ["pull", "--help"], title: "pull" },
  { name: "milieu status", args: ["status", "--help"], title: "status" },
  { name: "milieu changes", args: ["changes", "--help"], title: "changes" },
  { name: "milieu log", args: ["log", "--help"], title: "log" },
  { name: "milieu checkout", args: ["checkout", "--help"], title: "checkout" },
  { name: "milieu doctor", args: ["doctor", "--help"], title: "doctor" },
  { name: "milieu phrase", args: ["phrase", "--help"], title: "phrase" },
  { name: "milieu sessions", args: ["sessions", "--help"], title: "sessions" },
];

function runHelp(args) {
  try {
    return execSync(`${binPath} ${args.join(" ")}`, {
      cwd: repoRoot,
      encoding: "utf8",
    }).trim();
  } catch (err) {
    throw new Error(`failed to run help for ${args.join(" ")}: ${err}`);
  }
}

function escapeHtml(text) {
  return text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll("`", "&#96;");
}

const features = [
  "End-to-end encrypted dotenv sync (.env and .env.* only)",
  "Branch-scoped env sets with git-like workflows (status, push, pull, changes)",
  "History + checkout for per-file rollback",
  "Recovery phrase + keychain-backed UMK",
  "Team access controls (read/write roles, invites)",
  "Session management and device tracking",
  "Self-hostable Cloudflare Worker + D1 API",
];

function slugify(value) {
  return value.toLowerCase().replace(/[^\w]+/g, "-").replace(/(^-|-$)/g, "");
}

const docSections = commands.map((entry) => {
  const output = escapeHtml(runHelp(entry.args));
  const id = slugify(entry.title);
  return {
    id,
    title: entry.title,
    html: `
    <section class="doc-section" id="${id}">
      <h2>${entry.title}</h2>
      <pre><code>${output}</code></pre>
    </section>
  `,
  };
});

const sections = [
  {
    id: "features",
    title: "Features",
    html: `
    <section class="doc-section" id="features">
      <h2>Features</h2>
      <ul class="feature-list">
        ${features.map((item) => `<li>${item}</li>`).join("\n")}
      </ul>
    </section>
  `,
  },
  ...docSections,
];

const html = `export const DOCS_HTML = String.raw\`
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <meta name="theme-color" content="#1e1e2e" />
  <meta name="color-scheme" content="dark" />
  <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent" />
  <title>milieu docs</title>
  <style>
    :root {
      --crust: #11111b;
      --mantle: #181825;
      --base: #1e1e2e;
      --surface-0: #313244;
      --surface-1: #45475a;
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
      background: radial-gradient(1200px 800px at 15% 5%, #1b1b2a 0%, var(--base) 35%, var(--mantle) 100%);
      color: var(--text);
      min-height: 100vh;
      padding: 32px 20px 80px;
      overflow-x: hidden;
    }
    .container {
      width: min(1200px, 100%);
      margin: 0 auto;
    }
    .layout {
      display: grid;
      grid-template-columns: minmax(200px, 260px) minmax(0, 1fr);
      gap: 24px;
    }
    .sidebar {
      position: sticky;
      top: 24px;
      align-self: start;
      padding: 16px;
      border-radius: 16px;
      background: rgba(17,17,27,0.7);
      border: 1px solid var(--surface-1);
    }
    .drawer-toggle {
      display: none;
      align-items: center;
      gap: 8px;
      padding: 8px 12px;
      border-radius: 999px;
      border: 1px solid var(--surface-1);
      background: rgba(17,17,27,0.7);
      color: var(--text);
      font-size: 13px;
      text-decoration: none;
    }
    .drawer-toggle:hover {
      color: var(--lavender);
      border-color: var(--surface-2);
    }
    .drawer-backdrop {
      display: none;
      position: fixed;
      inset: 0;
      background: rgba(13, 13, 20, 0.6);
      z-index: 40;
    }
    .drawer-panel {
      position: fixed;
      top: 0;
      right: 0;
      height: 100%;
      width: min(320px, 90vw);
      transform: translateX(100%);
      transition: transform 0.2s ease;
      background: rgba(17,17,27,0.95);
      border-left: 1px solid var(--surface-1);
      padding: 16px;
      z-index: 50;
      overflow: auto;
    }
    .drawer-panel.open {
      transform: translateX(0);
    }
    .drawer-backdrop.show {
      display: block;
    }
    .sidebar h3 {
      margin: 0 0 12px 0;
      color: var(--lavender);
      font-size: 16px;
      letter-spacing: 0.05em;
      text-transform: uppercase;
    }
    .search {
      width: 100%;
      padding: 10px 12px;
      border-radius: 10px;
      border: 1px solid var(--surface-1);
      background: var(--crust);
      color: var(--text);
      margin-bottom: 12px;
    }
    .nav-list {
      list-style: none;
      padding: 0;
      margin: 0;
      display: flex;
      flex-direction: column;
      gap: 6px;
      max-height: calc(100vh - 220px);
      overflow: auto;
    }
    .nav-list a {
      color: var(--subtext);
      text-decoration: none;
      font-size: 13px;
      padding: 6px 8px;
      border-radius: 8px;
      display: block;
      word-break: break-word;
    }
    .nav-list a.active,
    .nav-list a:hover {
      color: var(--text);
      background: rgba(49,50,68,0.6);
    }
    header {
      margin-bottom: 32px;
    }
    .header-row {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 16px;
      flex-wrap: wrap;
    }
    .back-link {
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
    }
    .back-link:hover {
      color: var(--lavender);
      border-color: var(--surface-2);
    }
    h1 {
      color: var(--lavender);
      font-size: 32px;
      margin: 0 0 8px 0;
    }
    p {
      margin: 0;
      color: var(--subtext);
    }
    .doc-section {
      margin-top: 28px;
      padding: 20px;
      border-radius: 16px;
      background: linear-gradient(180deg, rgba(49,50,68,0.7), rgba(30,30,46,0.85));
      border: 1px solid var(--surface-1);
      width: 100%;
    }
    h2 {
      margin: 0 0 12px 0;
      color: var(--mauve);
      font-size: 20px;
      letter-spacing: 0.03em;
    }
    pre {
      margin: 0;
      padding: 16px;
      border-radius: 12px;
      background: var(--crust);
      border: 1px solid var(--surface-0);
      color: var(--text);
      line-height: 1.6;
      overflow-x: auto;
      max-width: 100%;
      width: 100%;
      box-sizing: border-box;
    }
    code {
      white-space: pre;
    }
    a { color: var(--blue); text-decoration: none; }
    a:hover { text-decoration: underline; }
    .feature-list {
      margin: 0;
      padding-left: 18px;
      color: var(--subtext);
      line-height: 1.6;
    }
    @media (max-width: 900px) {
      .layout {
        grid-template-columns: 1fr;
      }
      .sidebar {
        display: none;
      }
      .drawer-toggle {
        display: inline-flex;
      }
    }
    @media (max-width: 640px) {
      body {
        padding: 24px 14px 60px;
      }
      .doc-section {
        padding: 14px;
        border-radius: 12px;
      }
      .search {
        font-size: 14px;
        width: 100%;
        max-width: 100%;
      }
      .nav-list {
        max-height: none;
        width: 100%;
      }
      .nav-list a {
        font-size: 12px;
        width: 100%;
      }
      pre {
        padding: 10px;
        font-size: 12px;
        white-space: pre-wrap;
        overflow-x: visible;
        word-break: break-word;
      }
    }
  </style>
</head>
<body>
  <div class="container">
    <header>
      <div class="header-row">
        <div>
          <h1>milieu docs</h1>
          <p>Command reference generated from clap help output.</p>
        </div>
        <div style="display:flex; gap:10px; flex-wrap:wrap;">
          <a class="back-link" href="/">← Back to main</a>
          <button class="drawer-toggle" type="button" aria-expanded="false" aria-controls="doc-drawer">Sections</button>
        </div>
      </div>
    </header>
    <div class="layout">
      <aside class="sidebar">
        <h3>Sections</h3>
        <input class="search" type="search" placeholder="Search sections..." aria-label="Search sections" />
        <ul class="nav-list">
          ${sections
            .map(
              (section) =>
                `<li><a href="#${section.id}" data-section="${section.id}">${section.title}</a></li>`,
            )
            .join("\n")}
        </ul>
      </aside>
      <main>
        ${sections.map((section) => section.html).join("\n")}
      </main>
    </div>
    <div class="drawer-backdrop" data-drawer-backdrop></div>
    <aside class="drawer-panel" id="doc-drawer" aria-label="Sections drawer">
      <h3>Sections</h3>
      <input class="search" type="search" placeholder="Search sections..." aria-label="Search sections" />
      <ul class="nav-list">
        ${sections
          .map(
            (section) =>
              `<li><a href="#${section.id}" data-section="${section.id}">${section.title}</a></li>`,
          )
          .join("\n")}
      </ul>
    </aside>
  </div>
  <script>
    const inputs = Array.from(document.querySelectorAll(".search"));
    const navLinks = Array.from(document.querySelectorAll(".nav-list a"));
    const sections = Array.from(document.querySelectorAll(".doc-section"));
    const drawer = document.querySelector(".drawer-panel");
    const backdrop = document.querySelector("[data-drawer-backdrop]");
    const toggle = document.querySelector(".drawer-toggle");

    function filterSections(query) {
      const q = query.trim().toLowerCase();
      sections.forEach((section) => {
        const match = section.textContent.toLowerCase().includes(q);
        section.style.display = match ? "" : "none";
      });
      navLinks.forEach((link) => {
        const target = document.getElementById(link.dataset.section);
        link.style.display = target && target.style.display !== "none" ? "" : "none";
      });
    }

    function setActive(id) {
      navLinks.forEach((link) => {
        link.classList.toggle("active", link.dataset.section === id);
      });
    }

    inputs.forEach((input) => {
      input.addEventListener("input", (event) => {
        filterSections(event.target.value);
      });
    });

    function closeDrawer() {
      drawer?.classList.remove("open");
      backdrop?.classList.remove("show");
      toggle?.setAttribute("aria-expanded", "false");
    }

    toggle?.addEventListener("click", () => {
      const isOpen = drawer?.classList.toggle("open");
      if (isOpen) {
        backdrop?.classList.add("show");
      } else {
        backdrop?.classList.remove("show");
      }
      toggle?.setAttribute("aria-expanded", isOpen ? "true" : "false");
    });

    backdrop?.addEventListener("click", closeDrawer);
    navLinks.forEach((link) => link.addEventListener("click", closeDrawer));

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            setActive(entry.target.id);
          }
        });
      },
      { rootMargin: "-30% 0px -60% 0px" },
    );

    sections.forEach((section) => observer.observe(section));
  </script>
</body>
</html>
\`;\n`;

const outPath = resolve(import.meta.dirname, "..", "src", "docs.ts");
writeFileSync(outPath, html);
console.log(`wrote ${outPath}`);
