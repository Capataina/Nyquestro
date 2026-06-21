# Plan: Small Wins (credibility polish)

## Header

- **Status:** Planned (not started)
- **Scope:** A bundle of small, high-credibility-per-effort tasks, each completable in well under a day. None change engine behaviour; all raise the *signal* the repository sends to an employer or engineer skimming it. Grouped into one file so they don't get lost as individual scraps.
- **Why this matters:** The biggest hiring lever a side project pulls is usually not "more features" — it's looking *finished and professional* in the first 60 seconds someone spends on it. A green CI badge, real numbers, accurate docs, and a moving picture of the dashboard do more for the first impression than a fifth order type.
- **Exit rule:** each sub-item below has its own done-marker; this file is archived when all are ticked.

Items in this bundle (the latency-numbers-in-README item folded into [`benchmark-harness.md`](benchmark-harness.md), since the numbers are that harness's output):

1. Continuous integration (GitHub Actions)
2. Documentation-drift fix
3. Dashboard demo capture (asciinema / GIF)

---

## 1. Continuous integration (GitHub Actions)

- **What:** A `.github/workflows/ci.yml` that, on every push and PR, runs `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`. Add the green status badge to the top of the README.
- **Why for hiring:** A CI badge reads instantly as "this person ships like a professional." It also *proves* the test suite passes rather than asking the reader to take it on faith. Currently there is no CI at all.
- **Steps:**
  - [ ] `.github/workflows/ci.yml` — `stable` toolchain, cache `~/.cargo` + `target/`, jobs: fmt, clippy (deny warnings), test.
  - [ ] Confirm the build is `fmt`-clean and `clippy`-clean first (fix anything it flags — should be near-zero given the current 0-warning build).
  - [ ] Add the badge to `README.md`.
  - [ ] (Optional) a second job that runs `--benchmark --json` and uploads the numbers as an artefact, once `benchmark-harness.md` lands.
- **Done when:** a push shows a green check on GitHub and the README badge is live.

## 2. Documentation-drift fix

- **What:** Bring the directional docs back in line with what the code actually does. Three concrete drifts found on 2026-06-21:
  - The README's **⚡ Matching Engine** roadmap section is entirely unticked, yet resting-order support, aggressive sweep, partial-fill tracking, atomic cancellation, and self-match prevention all shipped on 2026-05-05 and are pinned by `tests/matching_test.rs`. The README currently *under-sells* the project. Tick the items that are actually done; leave the genuinely-future ones (lock-free pool, SPMC ring buffer, UDP gateway, risk guard, strategy agent) unticked.
  - `context/ARCHITECTURE.md`'s repo-structure tree and dependency diagram **omit `src/feed/` and `src/telemetry/`**, although both modules are described correctly in the subsystem table and prose lower down. Add them to the tree and the dependency picture.
  - `context/ARCHITECTURE.md` says "**88 tests** at last run"; the suite is now at **101** (verified `cargo test`, 2026-06-21). Update the count.
- **Why for hiring:** A README that under-claims is almost as bad as one that over-claims — a sharp reader notices the gap between the unticked roadmap and the obviously-working engine and wonders which to trust. Accurate docs are free credibility.
- **Steps:**
  - [ ] Tick the done items in the README **Matching Engine** (and any other) roadmap sections; leave future items unticked.
  - [ ] Add `feed/` and `telemetry/` to the architecture tree + dependency diagram.
  - [ ] Update the test count 88 → current.
- **Done when:** README roadmap reflects reality, architecture tree includes every `src/` module, test count is current.

## 3. Dashboard demo capture (asciinema / GIF)

- **What:** A short (~15–30s) recording of the live dashboard — synthetic mode is enough, or Coinbase live for extra flair — embedded at the top of the README.
- **Why for hiring:** The TUI is genuinely the most visually striking thing in the repo (gauge stacks, sparklines, distribution bars, health dots). A static repo gets *skimmed*; a moving dashboard gets *watched*. This is the single highest "wow per minute" artefact available, and it costs one recording.
- **Steps:**
  - [ ] Record with `asciinema rec` (crisp, copy-pasteable, small) or a terminal GIF tool (`vhs` by Charm produces reproducible scripted GIFs and is ideal here — a `demo.tape` script also documents *how* the demo was made).
  - [ ] Capture a Tab-cycle across all three symbols so the multi-instrument design is visible.
  - [ ] Embed at the top of the README (asciinema link or committed GIF under `docs/`).
- **Done when:** the README opens with a moving demo of the dashboard.

## Notes

- These are deliberately independent — do them in any order, commit each separately (small modular commits).
- Sequencing tip: do the doc-drift fix *after* `benchmark-harness.md` lands, so the README's new "Measured performance" numbers and the roadmap ticks land together as one coherent "the README now tells the truth" pass.
