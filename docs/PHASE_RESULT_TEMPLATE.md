# Phase Result Template — Copy once per completed experiment or phase

This is a **template**, not a completed result. Future entries may be `docs/results/P3A_YYYY-MM-DD.md` or updates to [PHASE_EVOLUTION.md](PHASE_EVOLUTION.md). Keep [PROJECT_STATE.md](PROJECT_STATE.md) authoritative for current progress. Preserve old results instead of rewriting their conclusions after later improvements.

## Identity and hypothesis

- Phase / experiment ID:
- Date (and protocol revision):
- Problem observed in previous phase:
- Single testable hypothesis:
- Prior baseline branch, git commit and PR:
- This phase branch, exact git commit and PR:
- Code change summary:
- What is **not** implemented, despite being planned:

## Fixture and reproduction

- Source fixture ID and permission/license:
- Original data SHA-256 (store privately if personal):
- Actual working width × height, max side, aspect/resize policy:
- Seed(s), CLI flags and full command:
- Build profile, Rust/OS/CPU where observed:
- Contour/structural mode and true accepted stroke budget:
- Comparison lane: natural defaults OR explicitly matched stroke/ink budget:
- Output filenames (PNG, ordered JSON, measurement report; unique):
- Output SHA-256 and manifest location:
- Is this artifact safe to show publicly? Source permission confirmed?:

## Evidence (report *observed*, not estimated)

- Core tests: N passed / failed; exact CI link:
- Fixture generation: run result:
- Actual render/measurement execution: run result:
- Stroke count; total path length; measured generation/runtime if available:
- Tone RMSE (overall/white/midtone/dark):
- White-source accidental-ink fraction:
- Edge proxy (threshold, tolerance, precision, recall, F1):
- Signed tone bias, when protocol defines it:
- Preview comparison / 100% crop / human comments:
- What improved against **exactly which** baseline:
- What regressed; unknown values; proxy caveats:
- Are the difference and seed/image count sufficient for conclusions?:

## Outcome and next step

- Decision: retain / revise / reject / inconclusive
- Exact reason and limits of the decision:
- Remaining blocker(s):
- Next falsifiable experiment:
- Historical phase entry linked from `PHASE_EVOLUTION.md`:
- Approved public gallery/source location (only with redistribution rights):
