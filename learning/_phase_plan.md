# Learning Archive Phase Plan

Phase 0 output. The per-project contract for which phases run, in what order, sub-divided how, with budget per phase. Every later phase opens by re-reading this file first.

The plan is **adaptive** (per `references/discovery-and-phase-planning.md`'s 4 knobs: phase set / order / decomposition / budget) but **additive** (per `references/phased-execution.md`'s F16 defense — phases are never skipped, collapsed, or made conditional on agentic judgement; if a phase's inventory is genuinely empty, the phase still runs and writes one absence-documentation file).

---

## Operating Mode

**Initialise.** `learning/` does not exist at HEAD. Reference: `ls /Users/atacanercetinkaya/Documents/Programming-Projects/Nyquestro/learning` returned `No such file or directory` before Phase 0 created the folder.

User's verbatim ask: "I want you to run upkeep learning please. this is our first time running upkeep learning in this project so there will be a lot to learn here." Direct match for Initialise per `references/operating-modes.md` §1: "Use when `learning/` does not exist or is too incomplete to update safely."

## Project Shape

Code repo, single Rust crate (edition 2024, single binary `nyquestro`). Mature `context/` regenerated 2026-05-04 covering all 10 src/ subsystems. Domain: HFT / matching engines / market microstructure — **domain-introducing** (the learner needs HFT theory taught; the README names microstructure observables, order-type breadth, lock-free internals, kernel bypass, and the strategy-agent business model as central, none of which are foundational programming knowledge).

**Interview-layer priority: high.** README is written as a portfolio piece; `notes/hft-firm-priorities.md` explicitly maps Jane Street / Citadel / HRT / Jump / Optiver / DRW / Tower / Two Sigma evaluative axes and identifies the project as deliberately CV-targeted for these audiences. The Phase 9 interview-layer is heavier than median.

**Architectural maturity: foundational tier shipped, next-tier roadmap.** Matching MVP + multi-instrument + Coinbase live + JSONL telemetry + Ratatui dashboard all shipped 2026-05-04 (10 commits in 16 minutes). Lock-free book / UDP gateway / FIX / risk layer / strategy agent remain README-defined planned work. The dashboard rendering must teach **both** the implemented foundation **and** the planned future direction (per `references/source-model.md` §"If the README makes an area central, it belongs in `learning/` even when it is not yet implemented").

## Phase Set

All standard phases (0, 1–11, Z) run. No project-specific phase added — the standard set already absorbs Nyquestro's shape via budget calibration on Phases 1, 2, 3, 9. Project-specific additions considered and rejected:

- **"Provenance and Upstream Diff" phase** — N/A. Nyquestro is from-scratch, not a fork.
- **"Compliance and Regulatory" phase** — out of scope. The risk-layer plan covers fat-finger / VaR / throttle, but compliance reporting is roadmap-tier and not central enough to warrant a phase.
- **"Performance Methodology" phase** — considered. The HDR-histogram + planned Criterion + planned `perf stat` integration is project-specific teaching. **Decision: fold into Phase 5b (testing-strategy + scaling-envelope cross-cutting files) and Phase 8 materials (Criterion / proptest / mutation references).** Adding it as a separate phase would split material that belongs together.

## Phase Order

Default dependency-honest order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10 → 11 → Z. **No deviation.**

Considered deviations:

- **Phase 3 (Domain-Advanced) before Phase 4 (Project-Architecture)?** — would help in a project where architecture invokes lock-free primitives the learner hasn't met. Rejected because Nyquestro's actual architecture is single-threaded BTreeMap-backed; lock-free is README-future, not implementation-current. The architecture file teaches the present clearly without needing INV-A01 first.
- **Phase 9 (Interview) before Phase 10 (Exercises)?** — already the default order. Confirmed.

## Phase Set Table

| Phase | Folder / role | Sub-divided? | Budget (content files) | Justification |
|---|---|---|---|---|
| **0** | discovery + phase planning (already executed) | N/A | 2 (`_inventory.md`, `_phase_plan.md`) | always runs; this file is one of its outputs |
| **1** | `concepts/foundations/` | flat | **10** | INV-F01..F10 — Rust+systems prerequisites; under sub-division threshold |
| **2** | `concepts/core/` | **split (2a, 2b)** | **20** total — 2a: 10 files, 2b: 10 files | INV-C01..C20 — domain-introducing; HFT/microstructure breadth warrants split into 2a (matching-engine mechanics: book / priority / cross / sweep / FIFO / lifecycle / self-match / quote semantics) and 2b (microstructure observables + order-type taxonomy: microprice / OFI / spread / queue position / adverse selection / inventory / tick / hidden / order-types / aggressor-vs-passive) |
| **3** | `concepts/domain-patterns/` + `concepts/advanced/` | **split (3a, 3b)** | **34** total — 3a: 12 files, 3b: 22 files | distinct file types (patterns the codebase actually uses vs theory the README invokes); 3b further sub-dividable if needed mid-phase per the ~10 threshold rule |
| **4** | `project/architecture/` | flat | **6** | INV-AR01..AR06 — meets the ≥4 zoom-level floor with two extra (synthetic/live mode + README-vs-implementation gap) |
| **5** | `project/systems/` + cross-cutting | **split (5a, 5b)** | **12** total — 5a: 10 files (one per src/ module), 5b: 2 files (`testing-strategy.md`, `scaling-envelope.md`) | distinct file types; engineering-depth floor (hallmark / performance-and-concurrency / failure-mode / observability / edge-case-≥8 / anticipated-questions-≥8) per system-file makes 5a the heaviest content phase in the run |
| **6** | `project/decisions/` | **split (6a, 6b)** | **18** total — 6a: 7 files (D01..D07: foundational engineering decisions), 6b: 11 files (D08..D18: implementation / venue / library choices) | INV-D enumeration crosses the ~10 threshold; the natural sub-division is "what kind of system are we building" (6a) vs "what specific tools were chosen" (6b) |
| **7** | `project/comparisons/` + `project/evolution/` | **split (7a, 7b)** | **15** total — 7a: 10 files, 7b: 5 files | distinct file types (cross-section comparisons vs longitudinal narratives) |
| **8** | `materials/` (incl. mandatory `comparable-systems.md`) | flat | **8** | INV-M01..M08 — under sub-division threshold; comparable-systems.md is the single mandatory cross-cutting file with ≥5 production-grade systems |
| **9** | `interview/` | **split (9a, 9b, 9c, 9d)** | **21** total — 9a: 8 files (pitches + hallmarks + qa-bank + domain-prereqs + evolution-narrative + frontier + diagnostic + behavioural), 9b: 6 files (mock/{canonical, adversarial, depth-probes, breadth-probes, coding, ambiguous}), 9c: ~3 files (recall flashcard sets), 9d: ~4 files (articulation drills incl. per-firm calibration) | high interview-priority project per `notes/hft-firm-priorities.md`; sub-division mandatory per the `interview/` subfolder structure |
| **10** | `exercises/` + `coding-gate/` | **split (10a, 10b)** | **27** total — 10a: 20 files (foundations + core + domain-patterns + project), 10b: 7 files (coding-gate, project-DSA-curated) + EXERCISE_GUIDE + EXERCISE_ORDER | INV-X enumeration crosses threshold; coding-gate needs separate manifest because the curation rule is project-specific, not a generic LeetCode list |
| **11** | `paths/` + top-level navigation | flat | **11** | 8 paths (INV-P01..P08) + LEARNING_MAP.md + GLOSSARY.md + STUDY_GUIDE.md (top-level navigation files compose the archive's entry points; written last because they cite everything) |
| **Z** | audit | N/A | 1 (`_audit.md`) | always runs |

## Total Content-File Count

Summed budgets: 2 + 10 + 20 + 34 + 6 + 12 + 18 + 15 + 8 + 21 + 27 + 11 + 1 = **185 content files**.

Plus per-phase admin artefacts (`_MANIFEST.md`, `_REVIEW.md`, `_HANDOFF.md` per phase + per sub-section). At minimum: 11 phases × 3 = 33 admin files; with sub-divisions adding sub-section manifests/reviews/handoffs, total admin ≈ 50 files.

**Grand total: ~235 files.** This is a domain-introducing, interview-priority Initialise on a 10-subsystem project. Scale is on-spec.

## Sub-Division Decisions — Justification per Split

**Phase 2 (concepts/core/) split into 2a / 2b** — 20 files crosses threshold. Split runs cleanly along the conceptual axis: 2a teaches *what an order book is and how matching works*; 2b teaches *what participants observe about the book and what order types are available*. Reading either standalone produces a coherent unit; reading flat would produce 20 uniform-feeling files.

**Phase 3 split into 3a (domain-patterns) / 3b (advanced)** — distinct file roles. 3a = patterns the codebase uses repeatedly across systems (event sourcing, validated constructors, drop-on-full backpressure, multi-instrument routing). 3b = theory the README invokes or that the project's roadmap will need (lock-free, intrusive lists, NUMA, kernel bypass, ITCH wire format, HDR histogram theory, OU SDE). Different teaching shape, different reads required.

**Phase 5 split into 5a (per-system) / 5b (cross-cutting)** — distinct file roles per `references/discovery-and-phase-planning.md` §worked example. 5a's 10 files share one template (the engineering-depth-floor systems-file shape); 5b's 2 files (testing-strategy, scaling-envelope) are cross-cutting and require different reads.

**Phase 6 split into 6a / 6b along the foundational/implementation axis** — 18 files crosses threshold. 6a covers the structural decisions that define what kind of system Nyquestro *is* (safe-Rust, BTreeMap+VecDeque, Symbol pack, deterministic match price, four-phase submit_limit, single-source severity, Copy events). 6b covers the implementation/library/venue decisions that define what tools were chosen (JSONL telemetry, drop-on-full, ANSI-16, in-process TUI, Coinbase initial venue, Market wrapper shape, synthetic-before-live phasing, HDR over t-digest, edition 2024, Tokio, tokio-tungstenite). 6a teaches the project's identity; 6b teaches the project's pragmatism. Both warrant their own focused file set.

**Phase 7 split into 7a (comparisons) / 7b (evolution)** — distinct file types. Comparisons are matrix-shaped survey files; evolution files are timeline-shaped narratives. Per `references/project-narrative-standards.md` (referenced in SKILL.md task-rules), evolution files have a specific narrative-with-cited-pivot-triggers shape that doesn't fit the comparison template.

**Phase 9 split into 9a / 9b / 9c / 9d** — mandatory per the `interview/` subfolder structure documented in `references/learning-architecture.md` §`interview/`. 9a covers the navigation + curation files (pitches / hallmarks / qa-bank / domain-prereqs / evolution-narrative / frontier / diagnostic / behavioural); 9b covers the six mock-archetype subfolders; 9c covers recall (flashcard) sets; 9d covers articulation drills (whiteboard scripts, per-firm calibration, non-expert explanations). Different read shapes per sub-phase.

**Phase 10 split into 10a / 10b** — distinct file roles per `references/coding-gate-strategy.md` (referenced in SKILL.md). 10a is the foundations/core/domain-patterns/project exercise progression; 10b is the project-DSA-curated coding-gate where the curation rule (drills only on data structures THIS project actually uses) is the load-bearing teaching.

## Budget per Phase — Inventory-Driven, Not Quota

Every budget above is the natural file count from the inventory enumeration in `_inventory.md`. The budget is not a cap — it tracks the inventory. If a phase surfaces additional INV items mid-run, the budget grows additively per the `references/phased-execution.md` "adaptive but additive" rule; the new items get appended to `_inventory.md` and flagged in that phase's `_HANDOFF.md`.

The budget is *also* not a target. A phase coming in well under budget while teaching shallow-but-numerous files is a failure mode (Potemkin archive); coming in over budget with each file teaching depth is fine.

## Phase 0 Verification — What Did I Miss?

Mandatory section per `references/discovery-and-phase-planning.md` §"Phase 0 Verification Gate". After writing the inventory and the phase set above, re-read the project surface area and answer "what did I miss?".

Re-read sweep: README in full (623 lines), `notes/hft-firm-priorities.md` (138 lines), `context/architecture.md` head (200 lines), `context/notes.md`, `notes/free-data-sources.md` summary, source-tree mtimes, the LifeOS `Projects/Nyquestro/_Overview.md`.

**What I missed in the first inventory pass — items added now (additive):**

1. **INV-A23: DEX adapter and on-chain settlement direction** — README §"Long-Term Direction" explicitly names a "DEX adapter layer that bridges the matching engine to on-chain settlement, connecting to Ethereum and Solana where the matching problem has the same structure but the settlement layer is a smart contract rather than a clearinghouse." Nontrivial — connects Nyquestro to Aurix, names a domain bridge worth teaching. Added under `concepts/advanced/` as an INV-A item; budget for Phase 3b extended from 22 to 23.

2. **INV-I21: per-firm interview calibration** — `notes/hft-firm-priorities.md` §1 enumerates per-firm character (Jane Street OCaml + correctness; Citadel C++ + kernel bypass + FPGA + co-location; HRT pure-quant + math; Optiver Rust-friendly + options; Jump distributed + crypto; DRW multi-asset + open-to-non-C++; Tower HFT pure-play; Two Sigma ML-heavy + research). The qa-bank Q&A surface differs by firm; the hallmarks emphasis differs by firm. This warrants its own file under `interview/articulation/per-firm-calibration.md`. Added as INV-I21; Phase 9d budget extended from 4 to 5.

3. **INV-CMP11: extensive testing framework — five-day test pyramid plan** — `notes/hft-firm-priorities.md` §8 Tier 1 explicitly names `plans/extensive-testing-framework.md` as the highest-leverage hiring-signal next move (proptest + proptest-state-machine + criterion + insta + stress + llvm-cov + cargo-mutants). The plan itself is teachable as a comparison: "what each test layer pins, what it doesn't, and why all five matter together". Added under `project/comparisons/`; Phase 7a budget extended from 10 to 11.

4. **Glossary scope refined** — `_inventory.md`'s INV-G entry (~40) is a placeholder. The actual glossary surface depends on cross-references generated during Phases 1–10; finalised list lands in Phase 11 when navigation closes. This is a planned-resolution point, not a missed item.

5. **No coverage gap on the README's "What This Project Is Not" line** — explicitly checked. The "not a retail trading tool / not a chart pattern set / not a wrapper around an exchange API / not a backtesting framework" framing is captured in INV-I04 (hallmarks) as engineer-to-engineer differentiation; no separate INV item warranted.

6. **Aurix bridge cross-link** — README's Long-Term Direction connects Nyquestro to Aurix's DeFi analytics work as a domain bridge. Captured in the new INV-A23 above; cross-referenced from `interview/hallmarks` and `concepts/advanced/dex-adapter-and-on-chain-settlement.md`.

**Inventory adjustments after verification (applied to `_inventory.md` next pass):**

- INV-A22 → INV-A23 added: "DEX adapter and on-chain settlement (Aurix bridge)" — README stretch direction connecting matching engines to on-chain settlement.
- INV-CMP11 added: "Extensive testing framework — proptest + state-machine + criterion + insta + stress + llvm-cov + cargo-mutants comparison".
- INV-I21 added: "Per-firm interview calibration (Jane Street / Citadel / HRT / Optiver / Jump / DRW / Tower / Two Sigma)".

Budget deltas: Phase 3b 22 → 23. Phase 7a 10 → 11. Phase 9d 4 → 5.

**Updated content-file total:** 185 + 3 = **188**.

**What was NOT missed (claim plus evidence):**

- All 10 src/ subsystems landed as INV-S items (`types`, `errors`, `order`, `events`, `book`, `metrics`, `simulator`, `feed`, `telemetry`, `ui`).
- Both mandatory cross-cutting project-files landed (`testing-strategy.md`, `scaling-envelope.md`).
- The mandatory `materials/comparable-systems.md` file role landed.
- The mandatory ≥4 zoom-level architecture surface landed (6 INV-AR items, exceeding the floor).
- All four operating-mode classifications considered; Initialise selected with explicit one-sentence justification.
- The full standard phase set runs; no phase skipped or made conditional.
- All standard `interview/` subfolders represented (pitch / hallmarks / qa-bank / domain-prereqs / evolution-narrative / frontier / diagnostic / behavioural / red-team / onboarding-others / mock × 6 archetypes / recall / articulation).
- The README's Features and Roadmap section (the long checkbox list) maps onto the inventory cleanly: implemented items → systems / decisions; planned items → advanced / future-direction labelling per `references/source-model.md` §"Status Labelling".

## Top Three Risks of Remaining Shallow After This Pass — Pattern-Breaker Commitment

Per the `SKILL.md` Phase 0 §"Pattern-breaker checkpoint" obligation:

1. **Microstructure observables (INV-C12..C17, INV-CMP01)** — microprice, OFI, effective spread, queue position, adverse selection, inventory risk. These are the topics where "Generic-Baseline Collapse" is most likely: a textbook explanation of OFI is the same for any matching engine; the hallmark is *how Nyquestro computes and surfaces it specifically* (the dashboard's pressure bar, the `book::ofi()` formula, the choice of L1 vs L5 imbalance). Phase 2b will deepen the project-grounding of every observable beyond the textbook formula. **Verification at Phase Z:** every C12–C17 file carries a "how Nyquestro implements / surfaces this" section anchored in `src/book/order_book.rs` or `src/ui/panes.rs`.

2. **Lock-free architecture (INV-A01..A04, INV-D02 second half)** — README's headline differentiator and the foundation of the long-term direction. Risk: getting this wrong in interview-prep is catastrophic (the engineer rehearses a wrong claim about CAS / ABA / hazard pointers under time pressure with conviction, and gets caught). Phase 3b's INV-A01..A04 must be source-cited (Herlihy & Shavit, Treiber 1986, Michael & Scott 1996, Maged Michael's hazard-pointers paper); Phase 6's INV-D02 lock-free-vs-current decision must show the migration path explicitly. **Verification at Phase Z:** every claim about lock-free behaviour grep-ed against `src/` (must yield "not implemented; this is theoretical") or against cited primary literature.

3. **Interview-layer fact-check across `interview/qa-bank.md`, `interview/red-team.md`, `interview/articulation/`** — per `code-inspection-protocol.md` §"Extended Scope: Interview-Layer Fact-Check", rehearsed wrong answers are strictly worse than no preparation. Risk: under the volume of ≥80 qa-bank questions, plausible-sounding answers slip through without grounding. Phase 9a–9d will include per-claim citations (`src/` line references for implementation claims; Easley/O'Hara / Stoikov / Cont-Kukanov / Herlihy-Shavit citations for theoretical claims; `notes/` references for design-rationale claims). **Verification at Phase Z:** spot-check 10 random qa-bank answers + 5 red-team answers + 3 articulation drill claims against either source code or cited primary references; flag any unverified claim.

## Resumability Across Sessions

Per `references/phased-execution.md` §"Resumability Per Phase Boundary", a run killed mid-Phase-N resumes by re-reading Phase 0's `_phase_plan.md` (this file), re-reading handoff files from Phases 1..N-1, and re-running Phase N from the start. Each phase's inputs are all on disk by design.

Concrete implication for this project: a single conversation may not complete all 11 content phases. The skill is designed to handle this — running Phases 1–4 in one session and Phases 5–11 + Z in a follow-up session is structurally fine; each phase re-reads what it needs.

## Open Questions for Human Review

**Question 1 (scope confirmation):** the plan above commits to ~188 content files + ~50 admin files = ~235 files of teaching material. The user's verbatim ask acknowledged scale ("there will be a lot to learn here"). Confirming proceed-as-planned vs trim before Phase 1 starts. The natural trim points if scope needs reduction:

- **Phase 9 (interview-layer)** could drop to a minimum-viable set (60s/3min/10min pitches + hallmarks + qa-bank + frontier + per-firm calibration = 6 files instead of 21). Cost: the mock-interview / red-team / articulation depth is what defends against actual senior-interview probes; thinning here is the most expensive trim per file.
- **Phase 10 (exercises)** could drop coding-gate (10b) and keep only the foundations/core/project exercise progression (10a). Cost: coding-gate is the project-DSA-curated drill set; thinning means the engineer falls back on generic LeetCode lists which `references/coding-gate-strategy.md` explicitly names as the failure mode this phase prevents.
- **Phase 3b (advanced)** could drop INV-A items that are README-future-only and not interview-likely (e.g. SIMD price comparison, eBPF telemetry). Cost: cheap trim — these are clearly stretch-direction-only.
- **Phase 7 (comparisons + evolution)** could drop to ~6 highest-leverage comparisons + 3 evolution files. Cost: medium — comparisons teach design space, which is a senior-interview surface.

**Question 2 (resumability decision):** run all phases in one session, or run Phases 0 → 4 in one session and Phases 5 → Z in follow-up sessions? The phased model is designed for either; the latter is safer on context-window risk for a run this size.

**Question 3 (priority within phases):** if scope is constrained, run heaviest-leverage first (interview + systems + decisions) and let lighter-weight phases (foundations + materials + paths) fill remaining capacity?

These three questions land at the user before Phase 1 begins.
