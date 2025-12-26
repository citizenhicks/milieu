export const DOCS_HTML = String.raw`
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
          <li><a href="#features" data-section="features">Features</a></li>
<li><a href="#overview" data-section="overview">Overview</a></li>
<li><a href="#register" data-section="register">register</a></li>
<li><a href="#login" data-section="login">login</a></li>
<li><a href="#logout" data-section="logout">logout</a></li>
<li><a href="#init" data-section="init">init</a></li>
<li><a href="#clone" data-section="clone">clone</a></li>
<li><a href="#repos" data-section="repos">repos</a></li>
<li><a href="#branch" data-section="branch">branch</a></li>
<li><a href="#add" data-section="add">add</a></li>
<li><a href="#remove" data-section="remove">remove</a></li>
<li><a href="#push" data-section="push">push</a></li>
<li><a href="#pull" data-section="pull">pull</a></li>
<li><a href="#status" data-section="status">status</a></li>
<li><a href="#changes" data-section="changes">changes</a></li>
<li><a href="#log" data-section="log">log</a></li>
<li><a href="#checkout" data-section="checkout">checkout</a></li>
<li><a href="#doctor" data-section="doctor">doctor</a></li>
<li><a href="#phrase" data-section="phrase">phrase</a></li>
<li><a href="#sessions" data-section="sessions">sessions</a></li>
        </ul>
      </aside>
      <main>
        
    <section class="doc-section" id="features">
      <h2>Features</h2>
      <ul class="feature-list">
        <li>End-to-end encrypted dotenv sync (.env and .env.* only)</li>
<li>Branch-scoped env sets with git-like workflows (status, push, pull, changes)</li>
<li>History + checkout for per-file rollback</li>
<li>Recovery phrase + keychain-backed UMK</li>
<li>Team access controls (read/write roles, invites)</li>
<li>Session management and device tracking</li>
<li>Self-hostable Cloudflare Worker + D1 API</li>
      </ul>
    </section>
  

    <section class="doc-section" id="overview">
      <h2>Overview</h2>
      <pre><code>rust e2ee dotenv sync

Usage: milieu [OPTIONS] &lt;COMMAND&gt;

Commands:
  register  create an account for milieu
  login     login to your milieu account
  logout    remove local auth and UMK from your keychain
  init      in your project directory, initialize the repo .milieu
  clone     clone a repo into this folder
  add       add a dotenv file to the current branch
  remove    remove a dotenv file from the current branch
  log       show version history for a file
  checkout  checkout a specific version of a file
  changes   show diffs for a file or all files
  repos     list repos linked to your user
  branch    manage repo branches and their dotenv files
  push      push branch changes to the server
  pull      download and decrypt dotenv files for a branch
  status    show local vs remote state for this repo
  doctor    check system prerequisites and configuration
  phrase    manage recovery phrase
  sessions  list active sessions for this user
  help      Print this message or the help of the given subcommand(s)

Options:
      --profile &lt;PROFILE&gt;  [default: default]
  -v, --verbose...         
  -h, --help               Print help
  -V, --version            Print version

tip: run &#96;milieu &lt;command&gt; --help&#96; for examples.</code></pre>
    </section>
  

    <section class="doc-section" id="register">
      <h2>register</h2>
      <pre><code>create an account for milieu

Usage: milieu register

Options:
  -h, --help  Print help

example: milieu register</code></pre>
    </section>
  

    <section class="doc-section" id="login">
      <h2>login</h2>
      <pre><code>login to your milieu account

Usage: milieu login

Options:
  -h, --help  Print help

example: milieu login</code></pre>
    </section>
  

    <section class="doc-section" id="logout">
      <h2>logout</h2>
      <pre><code>remove local auth and UMK from your keychain

Usage: milieu logout

Options:
  -h, --help  Print help

example: milieu logout</code></pre>
    </section>
  

    <section class="doc-section" id="init">
      <h2>init</h2>
      <pre><code>in your project directory, initialize the repo .milieu

Usage: milieu init [OPTIONS]

Options:
      --name &lt;NAME&gt;  
  -h, --help         Print help

examples:
  milieu init
  milieu init --name my_repo</code></pre>
    </section>
  

    <section class="doc-section" id="clone">
      <h2>clone</h2>
      <pre><code>clone a repo into this folder

Usage: milieu clone [OPTIONS]

Options:
      --repo &lt;REPO&gt;  
  -h, --help         Print help

examples:
  milieu clone --repo my-app
  milieu clone</code></pre>
    </section>
  

    <section class="doc-section" id="repos">
      <h2>repos</h2>
      <pre><code>list repos linked to your user

Usage: milieu repos &lt;COMMAND&gt;

Commands:
  list    list repos linked to this user
  manage  manage repo sharing and invites
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

example: milieu repos list</code></pre>
    </section>
  

    <section class="doc-section" id="branch">
      <h2>branch</h2>
      <pre><code>manage repo branches and their dotenv files

Usage: milieu branch &lt;COMMAND&gt;

Commands:
  list    list branches in this repo
  add     add a branch with one or more dotenv files
  remove  remove a branch from the manifest
  set     set the default branch
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

examples:
  milieu branch list
  milieu branch add dev --file .env
  milieu branch set dev</code></pre>
    </section>
  

    <section class="doc-section" id="add">
      <h2>add</h2>
      <pre><code>add a dotenv file to the current branch

Usage: milieu add [OPTIONS] &lt;PATH&gt;

Arguments:
  &lt;PATH&gt;  

Options:
      --tag &lt;TAG&gt;        
      --branch &lt;BRANCH&gt;  
  -h, --help             Print help

examples:
  milieu add .env
  milieu add .env.local --tag dev
  milieu add .env --branch prod</code></pre>
    </section>
  

    <section class="doc-section" id="remove">
      <h2>remove</h2>
      <pre><code>remove a dotenv file from the current branch

Usage: milieu remove [OPTIONS] &lt;PATH&gt;

Arguments:
  &lt;PATH&gt;  

Options:
      --branch &lt;BRANCH&gt;  
  -h, --help             Print help

examples:
  milieu remove .env
  milieu remove .env.local --branch prod</code></pre>
    </section>
  

    <section class="doc-section" id="push">
      <h2>push</h2>
      <pre><code>push branch changes to the server

Usage: milieu push [OPTIONS]

Options:
      --branch &lt;BRANCH&gt;  
  -h, --help             Print help

example: milieu push --branch dev</code></pre>
    </section>
  

    <section class="doc-section" id="pull">
      <h2>pull</h2>
      <pre><code>download and decrypt dotenv files for a branch

Usage: milieu pull [OPTIONS]

Options:
      --branch &lt;BRANCH&gt;  
  -h, --help             Print help

example: milieu pull --branch dev</code></pre>
    </section>
  

    <section class="doc-section" id="status">
      <h2>status</h2>
      <pre><code>show local vs remote state for this repo

Usage: milieu status [OPTIONS]

Options:
      --json  
  -h, --help  Print help

example: milieu status</code></pre>
    </section>
  

    <section class="doc-section" id="changes">
      <h2>changes</h2>
      <pre><code>show diffs for a file or all files

Usage: milieu changes [OPTIONS] [PATH]

Arguments:
  [PATH]  

Options:
      --version &lt;VERSION&gt;  
      --branch &lt;BRANCH&gt;    
  -h, --help               Print help

examples:
  milieu changes
  milieu changes .env
  milieu changes .env --version 3
  milieu changes --branch prod</code></pre>
    </section>
  

    <section class="doc-section" id="log">
      <h2>log</h2>
      <pre><code>show version history for a file

Usage: milieu log [OPTIONS] &lt;PATH&gt;

Arguments:
  &lt;PATH&gt;  

Options:
      --branch &lt;BRANCH&gt;  
  -h, --help             Print help

example: milieu log .env</code></pre>
    </section>
  

    <section class="doc-section" id="checkout">
      <h2>checkout</h2>
      <pre><code>checkout a specific version of a file

Usage: milieu checkout [OPTIONS] --version &lt;VERSION&gt; &lt;PATH&gt;

Arguments:
  &lt;PATH&gt;  

Options:
      --version &lt;VERSION&gt;  
      --branch &lt;BRANCH&gt;    
  -h, --help               Print help

example: milieu checkout .env --version 3</code></pre>
    </section>
  

    <section class="doc-section" id="doctor">
      <h2>doctor</h2>
      <pre><code>check system prerequisites and configuration

Usage: milieu doctor

Options:
  -h, --help  Print help

example: milieu doctor</code></pre>
    </section>
  

    <section class="doc-section" id="phrase">
      <h2>phrase</h2>
      <pre><code>manage recovery phrase

Usage: milieu phrase &lt;COMMAND&gt;

Commands:
  show    show the recovery phrase from keychain
  status  check if recovery phrase exists in keychain
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

examples:
  milieu phrase show
  milieu phrase status</code></pre>
    </section>
  

    <section class="doc-section" id="sessions">
      <h2>sessions</h2>
      <pre><code>list active sessions for this user

Usage: milieu sessions

Options:
  -h, --help  Print help

example: milieu sessions</code></pre>
    </section>
  
      </main>
    </div>
    <div class="drawer-backdrop" data-drawer-backdrop></div>
    <aside class="drawer-panel" id="doc-drawer" aria-label="Sections drawer">
      <h3>Sections</h3>
      <input class="search" type="search" placeholder="Search sections..." aria-label="Search sections" />
      <ul class="nav-list">
        <li><a href="#features" data-section="features">Features</a></li>
<li><a href="#overview" data-section="overview">Overview</a></li>
<li><a href="#register" data-section="register">register</a></li>
<li><a href="#login" data-section="login">login</a></li>
<li><a href="#logout" data-section="logout">logout</a></li>
<li><a href="#init" data-section="init">init</a></li>
<li><a href="#clone" data-section="clone">clone</a></li>
<li><a href="#repos" data-section="repos">repos</a></li>
<li><a href="#branch" data-section="branch">branch</a></li>
<li><a href="#add" data-section="add">add</a></li>
<li><a href="#remove" data-section="remove">remove</a></li>
<li><a href="#push" data-section="push">push</a></li>
<li><a href="#pull" data-section="pull">pull</a></li>
<li><a href="#status" data-section="status">status</a></li>
<li><a href="#changes" data-section="changes">changes</a></li>
<li><a href="#log" data-section="log">log</a></li>
<li><a href="#checkout" data-section="checkout">checkout</a></li>
<li><a href="#doctor" data-section="doctor">doctor</a></li>
<li><a href="#phrase" data-section="phrase">phrase</a></li>
<li><a href="#sessions" data-section="sessions">sessions</a></li>
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
`;
