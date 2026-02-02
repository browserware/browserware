# Reorganize docs to match reality

Move milestone planning docs into docs/plans/ structure and update all content to accurately reflect the current implementation state. Fix stale references, outdated code snippets, and incorrect status markers.

## Context

- Files involved:
  - Modify: `docs/current_milestone.md` (move to `docs/plans/`)
  - Delete: `docs/current_milestone_details.md` (content merged into plan)
  - Create: `docs/plans/2026-02-03-m1-browser-detection.md` (consolidated, accurate plan)
  - Modify: `docs/architecture.md` (update crate status table)
  - Modify: `CHANGELOG.md` (add macOS detection to Unreleased)
- Related patterns: Conventional commits, docs/ directory structure
- Dependencies: None

## Approach

- **Testing approach**: N/A (docs-only changes, verify cargo doc still builds)
- Complete each task fully before moving to the next
- Preserve all useful technical content from existing docs
- All status markers must match actual code (verified by reading source files)

## Tasks

### Task 1: Create consolidated M1 plan in docs/plans/

**Files:**
- Create: `docs/plans/2026-02-03-m1-browser-detection.md`

- [x] Write consolidated M1 plan that combines useful content from `current_milestone.md` and `current_milestone_details.md`
- [x] Accurately mark completed items: types crate, registry, macOS detection (discovery-first approach), CLI browsers command with table/json/plain output, default browser indicator, rustdoc
- [x] Accurately mark remaining items: Windows detection, Linux detection, integration tests, CLI tests, usage examples, CHANGELOG update, performance testing, cross-platform CI
- [x] Remove stale "Week 3-8" timeline references (use task ordering instead)
- [x] Fix code snippets: remove registry-first detection code from details, align with discovery-first approach from decisions.md
- [x] Include risk mitigation table and success criteria (updated status)

### Task 2: Update docs/architecture.md crate status table

**Files:**
- Modify: `docs/architecture.md`

- [x] Update `browserware-detect` status from "Scaffold" to "macOS active, Windows/Linux stub"
- [x] Update `browserware-cli` status from "Scaffold" to "Active (browsers command)"
- [x] Update `browserware-types` status from "In progress" to "Active"

### Task 3: Update CHANGELOG.md with macOS detection work

**Files:**
- Modify: `CHANGELOG.md`

- [x] Add macOS browser detection entries under [Unreleased] section
- [x] Include: browserware-detect crate with macOS Launch Services integration, browser registry, CLI browsers command with output formats

### Task 4: Remove old milestone docs

**Files:**
- Delete: `docs/current_milestone.md`
- Delete: `docs/current_milestone_details.md`

- [x] Delete `docs/current_milestone.md` (content migrated to docs/plans/)
- [x] Delete `docs/current_milestone_details.md` (content merged and corrected)

## Verification

- [x] Verify no broken cross-references in remaining docs
- [x] Run `cargo doc --workspace --no-deps` to confirm docs still build
- [x] Check that all status claims in the new plan match actual source code

## Wrap-up

- [x] Move this plan to `docs/plans/completed/` when done
