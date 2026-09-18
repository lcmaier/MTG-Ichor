//! The build script (`build = "stamp_commit.rs"` in Cargo.toml — named so it
//! does not read as `CardDataBuilder::build`). It bakes the build's commit
//! into the binary as `MTGSIM_COMMIT`, which the trace sink writes into every
//! trace's `game` header (`state::trace`). A
//! trace has to say which engine wrote it, and a two-version diff keys on
//! that; nothing at runtime can answer it honestly, because the binary a
//! sitting copies aside outlives the checkout it was built from.
//!
//! `unknown` when `git` is unavailable or this is not a checkout. The tree's
//! dirtiness is recorded as a `-dirty` suffix, which is why the script reruns
//! on any change under `src`, `tests` and `benches` as well as on the git
//! refs — a binary built from uncommitted edits must not claim the commit.

use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?;
    let s = s.trim();
    (!s.is_empty()).then(|| s.to_string())
}

fn main() {
    let commit = match git(&["rev-parse", "--short=12", "HEAD"]) {
        Some(head) => {
            let dirty = git(&["status", "--porcelain", "--untracked-files=no"]).is_some();
            if dirty { format!("{head}-dirty") } else { head }
        }
        None => "unknown".to_string(),
    };
    println!("cargo:rustc-env=MTGSIM_COMMIT={commit}");

    // The paths that change the answer: the tree, and the refs HEAD reads.
    println!("cargo:rerun-if-changed=stamp_commit.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=tests");
    for path in ["HEAD", "packed-refs"] {
        if let Some(p) = git(&["rev-parse", "--git-path", path]) {
            println!("cargo:rerun-if-changed={p}");
        }
    }
    if let Some(branch) = git(&["symbolic-ref", "-q", "HEAD"])
        && let Some(p) = git(&["rev-parse", "--git-path", &branch])
    {
        println!("cargo:rerun-if-changed={p}");
    }
}
