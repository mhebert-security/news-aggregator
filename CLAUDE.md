# CLAUDE.md — news-aggregator

This file is the operational brief for Claude Code working in this repo.
Read it before touching anything.

---

## What This Project Is

A cybersecurity news aggregator written in Rust. Two binaries: a standalone
fetcher that pulls RSS/Atom feeds from 10 sources every 6 hours and writes
items to SQLite, and a route handler that serves the feed at `/news` inside
the zero-trust-server on mhebert.dev. Access is session-gated — unauthenticated
requests get the PoW challenge, same as every other protected route.

---

## Infrastructure

### Production Server (Hetzner)
- **Host:** 188.245.239.118
- **SSH:** `ssh -i ~/.ssh/id_ed25519 root@188.245.239.118`
- **Zero-trust-server source:** `/projects/zero-trust-server/`
- **News fetcher binary:** `/opt/cyber-news/cyber-news-fetcher`
- **SQLite database:** `/var/lib/cyber-news/news.db`
- **Logs:** `journalctl -u cyber-news-fetcher -f`

### Local Development
- **Source:** `~/Desktop/Github Projects/news-aggregator/`
- **Build:** `cargo build` (debug) / `cargo build --release` (production)
- **Check:** `cargo check && cargo clippy -- -D warnings`
- **Local DB:** set `CYBER_NEWS_DB_PATH=./dev-news.db` for local testing

---

## Repo Structure

```
src/
  lib.rs               Shared types and utilities
  store.rs             SQLite interface — schema, insert, query, cleanup
  feeds.rs             Feed list, Category enum, domain allowlist
  renderer.rs          HTML page builder — no template engine
bin/
  cyber_news_fetcher.rs  Standalone fetcher binary — run by systemd timer
deploy/
  cyber-news-fetcher.service   systemd service unit
  cyber-news-fetcher.timer     systemd timer unit
  install.sh                   Timer installation script
```

The `/news` route handler lives in the zero-trust-server repo at
`src/handlers/news.rs` — it imports from this crate as a dependency.

---

## Coding Constraints — Non-Negotiable

- **No async/await anywhere.** Blocking I/O only. Tokio must never be added.
- **No web frameworks.** No Actix, Axum, Rocket, or anything similar.
- **No template engines.** Build HTML strings directly in renderer.rs.
- **Stdlib-first.** Approved external deps: rustls, rustls-pemfile,
  rustls-pki-types, rusqlite, quick-xml (RSS parsing only).
- **No unwrap() in production paths.** Handle errors explicitly. Log and
  continue — never panic on external input.
- **No redirect following.** Validate TLS on all outbound connections.
  Domain allowlist enforced before any outbound connection opens.

---

## SQLite Schema

Single `items` table:

```sql
CREATE TABLE IF NOT EXISTS items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    title       TEXT NOT NULL,
    url         TEXT NOT NULL UNIQUE,
    description TEXT,
    published   INTEGER NOT NULL,  -- Unix timestamp
    source      TEXT NOT NULL,
    category    TEXT NOT NULL,
    fetched_at  INTEGER NOT NULL   -- Unix timestamp
);

CREATE INDEX IF NOT EXISTS idx_published ON items (published DESC);
```

- `url` has a unique constraint — use `INSERT OR IGNORE`
- Items older than 7 days are deleted on each fetch cycle
- `CYBER_NEWS_DB_PATH` env var overrides the default path
- Default path: `/var/lib/cyber-news/news.db`

---

## Feed List

| Source | URL | Category |
|---|---|---|
| NVD CVE | https://nvd.nist.gov/feeds/xml/cve/misc/nvd-rss-analyzed.xml | Cve |
| CISA KEV | https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json | Cve |
| CISA ICS | https://www.cisa.gov/cybersecurity-advisories/ics-advisories.xml | Ot |
| Bleeping Computer | https://www.bleepingcomputer.com/feed/ | Breach |
| Krebs on Security | https://krebsonsecurity.com/feed/ | Breach |
| The Hacker News | https://feeds.feedburner.com/TheHackersNews | ThreatIntel |
| Dark Reading | https://www.darkreading.com/rss.xml | ThreatIntel |
| SANS ISC | https://isc.sans.edu/rssfeed.xml | Research |
| Schneier on Security | https://www.schneier.com/feed/atom | Research |
| r/netsec | https://www.reddit.com/r/netsec/.rss | Research |

---

## Outbound Fetch Rules

Every outbound connection must follow these rules — no exceptions:

- Extract the domain from the feed URL before connecting
- Verify the domain is in the feed list allowlist — if not, log and skip
- TLS verification enforced — no certificate skipping, ever
- `set_read_timeout` and `set_write_timeout` set to 10 seconds on every connection
- Abort the read if the response exceeds 1MB
- No redirect following
- User-Agent header: `mhebert-cyber-news/1.0`
- On any feed failure: log the error, continue to next feed, never panic

---

## Fetcher Workflow

```
1. Open SQLite database (create if not exists)
2. For each feed in the feed list:
   a. Check domain against allowlist
   b. Open TLS connection with 10s timeout
   c. Send HTTP GET with correct User-Agent
   d. Read response up to 1MB cap
   e. Parse RSS/Atom XML
   f. Extract title, url, description, published date
   g. Call insert_item() for each parsed item (duplicates ignored)
3. Call cleanup(7) to delete items older than 7 days
4. Exit cleanly
```

---

## HTML Renderer

`renderer.rs` builds a complete HTML page from a `Vec<NewsItem>`. No template
engine. Build the string directly.

Page structure:
- Dark theme using CSS variables matching mhebert.dev's style.css
- Header: "Cyber News" + last updated timestamp
- Category filter tabs: All / CVE / Breach / Threat Intel / OT / Research
- Per-item display: headline (linked to source), source name, category badge,
  relative time ago ("2 hours ago", "yesterday")
- Pure CSS tab filtering — no JavaScript
- Mobile responsive

---

## Systemd Timer

Fetcher runs every 6 hours via systemd timer:

```
OnCalendar=*-*-* 00,06,12,18:00:00
Persistent=true
```

`Type=oneshot` — the service exits after each run. Systemd handles the next
tick. No retry loops inside the binary.

---

## Git Protocol

Verify identity before every session:

```bash
git config user.name   # must return: Matthew Hebert
git config user.email  # must return: mattcfhebert@gmail.com
```

If wrong, fix before committing:

```bash
git config user.name "Matthew Hebert"
git config user.email "mattcfhebert@gmail.com"
```

One commit per completed task. Commit message format:

```
feat(news): <short description>
fix(news): <short description>
```

Push after every commit:

```bash
git push origin main
```

Never commit: `target/`, `dev-news.db`, `*.env`.

---

## Verification Before Any Commit

```bash
cargo check                        # zero errors
cargo clippy -- -D warnings        # zero warnings
```

Both must pass clean. Do not commit on a failed check.

---

## Prose Standard

All documentation, comments, and HTML copy follow these rules:

- Active voice. Cut every word that restates the sentence before it.
- Concrete nouns, strong verbs. No filler openers.
- Short paragraphs. Three sentences is a paragraph.
- No em dashes used as clause separators — restructure the sentence instead.
- HTML copy and error messages visible to users follow the same standard.
