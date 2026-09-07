# cyber-news-aggregator

A cybersecurity news aggregator in Rust. A standalone fetcher pulls ten
sources every six hours and writes each item to SQLite. The same store and
renderer build the /news page that the zero-trust server on mhebert.dev
serves behind its proof of work gate.

## What it does

- Fetches ten fixed HTTPS sources: NVD (REST JSON), CISA KEV (JSON), CISA ICS,
  BleepingComputer, Krebs on Security, The Hacker News, Dark Reading, SANS
  ISC, Schneier on Security, and r/netsec.
- Files each item under one of five categories: CVE, Breach, Threat Intel, OT,
  Research. A CVE advisory and a breach report never sit in the same pile.
- Stores items in SQLite and keeps a rolling seven days.
- Opens TLS directly with no redirects, no async, and no web framework. The
  domain allowlist is the feed list itself, enforced before any connection
  opens.

## Repository layout

    src/lib.rs                       Library root: feeds, fetch, renderer, store
    src/feeds.rs                     The fixed catalog and the category vocabulary
    src/store.rs                     SQLite schema, insert, query, cleanup
    src/fetch.rs                     HTTP over verified TLS, RSS/Atom/JSON parsing
    src/renderer.rs                  HTML for the /news page, no template engine
    src/bin/cyber_news_fetcher.rs    The six-hour fetcher binary

## Run the fetcher

    CYBER_NEWS_DB_PATH=./dev-news.db cargo run --bin cyber_news_fetcher

The database path resolves from the first command-line argument, then the
CYBER_NEWS_DB_PATH environment variable, then the production default at
/var/lib/cyber-news/news.db.

## Test and check

    cargo test
    cargo check
    cargo clippy -- -D warnings

## How it deploys

The production fetcher runs on the same NixOS box as the zero-trust server, as
a hardened oneshot service on a six-hour timer. The service and timer units
live in the site-infrastructure repo's configuration.nix. The server reads the
store it cannot write: /news opens the database read-only and answers the page
from the rows the fetcher left.

The database file is the whole interface between the two processes.
