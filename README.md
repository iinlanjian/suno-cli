<div align="center">

# suno-cli

**Generate AI music from your terminal — full Suno v5.5 support**

<br />

[![Star this repo](https://img.shields.io/github/stars/199-biotechnologies/suno-cli?style=for-the-badge&logo=github&label=%E2%AD%90%20Star%20this%20repo&color=yellow)](https://github.com/199-biotechnologies/suno-cli/stargazers)
&nbsp;&nbsp;
[![Follow @longevityboris](https://img.shields.io/badge/Follow_%40longevityboris-000000?style=for-the-badge&logo=x&logoColor=white)](https://x.com/longevityboris)

<br />

[![License: MIT](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)](LICENSE)
&nbsp;
[![Rust](https://img.shields.io/badge/Rust-2024-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
&nbsp;
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen?style=for-the-badge)](https://github.com/199-biotechnologies/suno-cli/pulls)

---

A 3.4MB Rust binary that talks directly to Suno's API. Generate songs with custom lyrics, style tags, vocal control, weirdness/style sliders, and every v5.5 feature — no browser needed.

[Install](#install) | [Quick Start](#quick-start) | [Commands](#commands) | [Features](#features) | [Contributing](#contributing)

</div>

## Why

Suno has no official API. The web UI works, but you can't script it, pipe lyrics from a file, batch-generate, or integrate it into a music production workflow.

This CLI fixes that. Cookie-based auth, every generation parameter exposed as a flag, dual JSON/table output for both humans and AI agents.

## Install

### Homebrew (macOS/Linux)

```bash
brew tap 199-biotechnologies/tap
brew install suno-cli
```

### Cargo (any platform)

```bash
cargo install suno-cli
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/199-biotechnologies/suno-cli/releases) — binaries for macOS (Apple Silicon + Intel), Linux (x86_64 + ARM), and Windows.

## Quick Start

```bash
# 1. Get your JWT from browser DevTools (Network tab → any Suno request → Authorization header)
suno auth --jwt "eyJhbG..." --device "your-device-id"

# 2. Check your credits
suno credits

# 3. Generate lyrics (free, no credits)
suno lyrics --prompt "a song about weekend coding sessions"

# 4. Generate a song
suno generate \
  --title "Weekend Code" \
  --tags "indie rock, guitar, upbeat" \
  --exclude "metal, heavy" \
  --lyrics-file lyrics.txt \
  --vocal male \
  --weirdness 40 \
  --style-influence 65 \
  --wait --download ./songs/
```

## Commands

```
suno generate        Custom mode — lyrics + tags + title + sliders
suno describe        Description mode — Suno writes lyrics from your prompt
suno lyrics          Generate lyrics only (free, no credits)
suno extend          Continue a clip from a timestamp
suno concat          Stitch clips into a full song
suno cover           Create a cover with different style
suno remaster        Remaster with a different model
suno stems           Extract vocals and instruments
suno list            List your songs
suno search <query>  Search songs by title or tags
suno status <id>     Check generation progress
suno download <ids>  Download audio/video (multiple IDs supported)
suno delete <ids>    Delete/trash clips
suno credits         Show balance and plan info
suno models          List available models with limits
suno auth            Set up authentication
suno config         show | set | check
suno agent-info     Machine-readable capabilities JSON
```

## Features

### Generation Parameters

| Flag | What it does | Values |
|---|---|---|
| `--title` | Song title | up to 100 chars |
| `--tags` | Style direction | `"pop, synths, upbeat"` (1000 chars) |
| `--exclude` | Styles to avoid | `"metal, heavy, dark"` (1000 chars) |
| `--lyrics` / `--lyrics-file` | Custom lyrics with `[Verse]` tags | up to 5000 chars |
| `--prompt` (inspire) | Free text description | up to 500 chars |
| `--model` | Model version | v5.5, v5, v4.5+, v4.5, v4, v3.5, v3, v2 |
| `--vocal` | Vocal gender | male, female |
| `--weirdness` | How experimental | 0-100 |
| `--style-influence` | How strictly to follow tags | 0-100 |
| `--variation` | Output variation | high, normal, subtle |
| `--instrumental` | No vocals | flag |
| `--wait` | Block until done | flag |
| `--download <dir>` | Auto-download after generation | directory path |

### Models

| Version | Codename | Default | Max Lyrics |
|---|---|---|---|
| **v5.5** | chirp-fenix | Yes | 5000 chars |
| v5 | chirp-crow | | 5000 chars |
| v4.5+ | chirp-bluejay | | 5000 chars |
| v4.5 | chirp-auk | | 5000 chars |
| v4 | chirp-v4 | | 3000 chars |

### Agent-Friendly

Every command supports `--json` for structured output. When stdout is piped, JSON is auto-detected. Progress, errors, and spinners go to stderr. Exit codes are semantic:

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Runtime error |
| 2 | Config error |
| 3 | Auth error |
| 4 | Rate limited |

```bash
# Pipe-friendly: auto-JSON when piped
suno feed | jq '.[0].title'

# Explicit JSON
suno credits --json
```

### How Auth Works

```
Browser DevTools → copy JWT + device-id → suno auth → stored at ~/.config/suno-cli/auth.json
```

The JWT expires after ~1 hour. Re-run `suno auth` with a fresh token when it expires. The CLI checks expiry before every request and tells you when it's time to refresh.

## Project Structure

```
src/
├── main.rs           Entry point, command routing
├── cli.rs            All clap derive structs
├── auth.rs           JWT storage, browser-token generation
├── config.rs         Config layering (env vars, defaults)
├── errors.rs         Error types with exit codes
├── download.rs       Audio/video download with progress bar
├── api/
│   ├── mod.rs        SunoClient — auth headers, base requests
│   ├── types.rs      Clip, Model, BillingInfo, etc.
│   ├── generate.rs   Music generation + polling
│   ├── lyrics.rs     Lyrics generation
│   ├── billing.rs    Credits and plan info
│   ├── feed.rs       Song listing
│   ├── concat.rs     Clip concatenation
│   ├── cover.rs      Cover generation
│   ├── remaster.rs   Remastering
│   └── stems.rs      Stem extraction
└── output/
    ├── mod.rs        Format detection (TTY vs piped)
    ├── json.rs       JSON envelope
    └── table.rs      Terminal tables
```

## Contributing

1. Fork the repo
2. Create a branch (`git checkout -b feature/your-idea`)
3. Make your changes and test with `cargo test`
4. Open a PR

We especially welcome:
- New API endpoint coverage (personas, voices, custom models)
- Better auth flows (Clerk cookie refresh, OS keychain)
- Integration tests

## License

MIT — see [LICENSE](LICENSE).

---

<div align="center">

Built by [Boris Djordjevic](https://github.com/longevityboris) at [Paperfoot AI](https://paperfoot.com)

<br />

**If this saves you time:**

[![Star this repo](https://img.shields.io/github/stars/199-biotechnologies/suno-cli?style=for-the-badge&logo=github&label=%E2%AD%90%20Star%20this%20repo&color=yellow)](https://github.com/199-biotechnologies/suno-cli/stargazers)
&nbsp;&nbsp;
[![Follow @longevityboris](https://img.shields.io/badge/Follow_%40longevityboris-000000?style=for-the-badge&logo=x&logoColor=white)](https://x.com/longevityboris)

</div>
