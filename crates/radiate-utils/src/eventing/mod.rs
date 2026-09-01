//! Playground for a unified `Event`/`EventHandler` pub/sub system — a simplified successor
//! to `radiate-engines::events`, prototyped here in isolation so nothing else in the
//! workspace depends on it yet. See `.claude/plans/piped-kindling-willow.md` for the design.
//!
//! `direct.rs` is the chosen shape as of 2026-09-01: no mailbox, no per-message boxing — a
//! subscriber is `Arc<Mutex<H>>`, publishing locks and calls `handle()` directly. An earlier
//! mailbox-based variant (`mailbox.rs`/`subscriber.rs`/`hub.rs`, plus a shared `handler.rs`)
//! was built and benchmarked alongside it; once both were measured fairly (with a warm-up
//! pass — the first read showed mailbox publish ~3x behind, which turned out to be almost
//! entirely cold-start cost, not a real gap) `direct` came out consistently faster across
//! every scenario, and simpler, so the mailbox variant was removed rather than kept as a
//! second maintained path. If FIFO ordering under a multi-worker pool or panic isolation
//! ever becomes a real requirement, that history — and the git log for this directory — is
//! the place to look, not a live sibling module.
//!
//! `#![allow(dead_code)]`: this whole subtree is WIP — pieces are only exercised by their own
//! `#[cfg(test)]` modules until this is wired into the rest of the workspace. Remove once the
//! module is promoted out of playground status.
#![allow(dead_code)]

mod direct;
mod executor;
mod subscription;
