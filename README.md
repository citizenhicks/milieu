# milieu <img src="https://raw.githubusercontent.com/citizenhicks/milieu/main/server/assets/favicon-32x32.png" alt="milieu icon" width="24" height="24" />

[![CI](https://github.com/citizenhicks/milieu/actions/workflows/ci.yml/badge.svg)](https://github.com/citizenhicks/milieu/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/citizenhicks/milieu?display_name=tag)](https://github.com/citizenhicks/milieu/releases)
[![Homebrew Tap](https://img.shields.io/badge/homebrew-citizenhicks%2Fmilieu-orange?logo=homebrew)](https://github.com/citizenhicks/homebrew-milieu)

Milieu is a small CLI that syncs encrypted dotenv files across machines without committing plaintext secrets.

**Beta warning:** this project is in public beta. Use at your own risk.

## Features

- End-to-end encrypted dotenv sync (.env and .env.\* only)
- Git-like workflows (status, push, pull, changes, history, checkout)
- Branch-scoped env sets per repo
- Recovery phrase + keychain-backed UMK storage
- Team access controls (read/write roles, invites)
- Session management + device tracking
- Self-hostable Cloudflare Worker + D1 API

This repo contains:

- `crates/milieu`: the Rust CLI
- `server`: a minimal HTTP API (Cloudflare Workers + D1) that stores only ciphertext

## Status

Default API base URL: `https://milieu.sh` (override with `MILIEU_BASE_URL`).

## Install

Homebrew tap (macOS + Linux):

```
brew tap citizenhicks/milieu
brew install milieu
```

## Server config (local)

Use `server/wrangler.local.toml` for your local route + D1 settings. The repo includes
`server/wrangler.local.toml.example` as a template.

## Release to Homebrew Tap (macOS + Linux)

Automated on tag push (see `.github/workflows/release-brew.yml`):

## Quick layout

```
crates/milieu/   # Rust CLI
server/         # Cloudflare Workers API + D1 schema
```
