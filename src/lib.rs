//! The cyber-news aggregator core, as a library and one standalone binary.
//!
//! `feeds` holds the fixed catalog of ten sources and the category vocabulary,
//! which is the whole egress surface. `fetch` reads one feed over verified TLS
//! and returns items. `store` persists them to SQLite. `renderer` builds the
//! /news HTML page from the rows a reader pulls back. The fetcher binary
//! (src/bin/cyber_news_fetcher.rs) ties feeds, fetch, and store together on a
//! six-hour timer.
//!
//! The /news route on mhebert.dev runs in the zero-trust-server repo, which
//! consumes this same store and renderer. Nothing in the write path shares
//! state with the read path except the database file: the fetcher writes it on
//! its timer, and the route opens it read-only. This crate is that shared code
//! extracted, so both processes build from one source.

pub mod feeds;
pub mod fetch;
pub mod renderer;
pub mod store;
