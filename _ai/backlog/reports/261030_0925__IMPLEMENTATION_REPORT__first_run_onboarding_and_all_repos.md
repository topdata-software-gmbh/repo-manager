---
filename: "_ai/backlog/reports/261030_0925__IMPLEMENTATION_REPORT__first_run_onboarding_and_all_repos.md"
title: "Report: First-Run Onboarding Wizard and Filterable All Repos Page"
createdAt: 2026-10-30 09:25
updatedAt: 2026-10-30 09:25
planFile: "_ai/backlog/active/261030_0925__IMPLEMENTATION_PLAN__first_run_onboarding_and_all_repos.md"
specFile: ""
project: "repo-manager"
status: completed
filesCreated: 14
filesModified: 17
filesDeleted: 0
tags: [onboarding, ux, discovery, groups, filtering]
documentType: IMPLEMENTATION_REPORT
---

# Report: First-Run Onboarding Wizard and Filterable All Repos Page

## Summary

The application now opens a resumable four-step setup wizard backed by a
backward-compatible `onboarding` block in `config.json`. The wizard scans one
folder deeply, optionally pre-selects projects by remote owner name, validates
an explicit repository selection, proposes editable groups, and ends on a ready
screen that explains Save to Git, Get latest, and Send changes. The former
"All projects" table is now a virtual **All repos** page with status, group,
repository-kind, and search filters, offering row-level actions only.

## Files Changed

### Created

- `src/components/OnboardingWizard.vue` — full-screen wizard shell, progress,
  Escape-to-close, and step routing.
- `src/components/onboarding/OnboardingStepFind.vue` — folder, owner names,
  advanced scan depth/grouping, and scan submission.
- `src/components/onboarding/OnboardingStepChoose.vue` — search, selection,
  kind/branch summary, and explicit acceptance of discovered repositories.
- `src/components/onboarding/OnboardingStepGroups.vue` — editable suggested
  groups with include toggles and name validation.
- `src/components/onboarding/OnboardingStepReady.vue` — setup summary, action
  explanations, and opening a group or All repos.
- `src/components/AllReposPage.vue` — virtual browse page with combined filters.
- `src/components/RepoListTable.vue` — shared repository table with group and
  browse action modes.
- `src/lib/onboarding.ts` — onboarding defaults, first-run detection, owner
  parsing/matching, and group suggestions.
- `src/lib/repoFilters.ts` — status matchers, combined filters, and status counts.
- `src/lib/repoStatus.ts` — shared status badges, operation labels, branch labels.
- `src/lib/groupDefaults.ts` — group palette and group factory.
- `src/lib/__tests__/onboarding.test.ts` — 15 pure-logic tests.
- `src/lib/__tests__/repoFilters.test.ts` — 13 pure-logic tests.
- `_ai/backlog/reports/261030_0925__IMPLEMENTATION_REPORT__first_run_onboarding_and_all_repos.md`
  — this report.

### Modified

- `src-tauri/src/models.rs` — added `GroupingStyle`, `OnboardingState`,
  `RepoKind`, the `AppConfig.onboarding` field, and `Repository.kind`.
- `src-tauri/src/config.rs` — legacy-config test proving files without onboarding
  still load.
- `src-tauri/src/service.rs` — `save_onboarding_state` normalization/validation
  and unit tests.
- `src-tauri/src/commands/config_commands.rs` — Tauri command wrapper.
- `src-tauri/src/main.rs` — registered the command.
- `src-tauri/src/web.rs` — `POST /api/onboarding` handler and route.
- `src-tauri/src/scanner.rs` — local repository-kind detection and test.
- `src/types/domain.ts` — onboarding, grouping, and repository-kind types.
- `src/lib/backend.ts` — `saveOnboardingState`.
- `src/stores/repoStore.ts` — onboarding state/actions, config loading, filters,
  single-repository actions, and `initialize`.
- `src/App.vue` — wizard/All repos routing and shared initialization.
- `src/components/Sidebar.vue` — "All repos" label and Setup entry.
- `src/components/RepoTable.vue` — now delegates rows to `RepoListTable`.
- `src/components/Toolbar.vue` — group-only fallback title.
- `src/components/GroupEditorModal.vue` — imports the shared group palette.
- `README.md` — first-run flow, All repos behavior, config example, API row.
- `CHANGELOG.md` — Unreleased entries for the feature.

## Key Changes

- **Onboarding persistence:** `OnboardingState` is additive and defaults when
  missing. Owner names are trimmed, de-`@`-ed, lowercased, deduplicated, and
  emptied of blanks; steps are validated to `0..=3`; scan paths expand `~`.
- **Transports:** the service function is exposed through both Tauri
  (`save_onboarding_state`) and axum (`POST /api/onboarding`), with a typed
  frontend call.
- **Kind detection:** themes are detected by `src/Resources/theme.json`; plugins
  by `composer.json` type `shopware-platform-plugin`; everything else is
  `other`. No network or vendor assumptions.
- **Pure frontend logic:** first-run detection, owner matching, folder-derived
  grouping, status matching, combined filters, and status counts are isolated in
  testable modules.
- **Wizard:** four focused steps, resumable progress, owner pre-selection,
  selectable repository list, editable/skippable groups, and a ready screen that
  creates groups only when the user finishes.
- **All repos:** always lists every discovered repository, cannot be renamed or
  deleted, exposes no bulk actions, and shares row rendering with group tables.

## Deviations from Plan

- Replaced `Array.prototype.at(-1)` in `src/lib/onboarding.ts` with indexed
  access because the project targets ES2020 and `vue-tsc` rejects `.at`.
- The plan's `completeOnboarding` snippet fell back to the first group for a
  `null` argument, contradicting its prose. `null` now opens the virtual All
  repos view as intended.
- "Select all visible" now unions with the current selection instead of replacing
  it, so it does not silently drop selections hidden by the search term.
- Scan depth from the wizard is rounded and clamped to `1..=12` before saving,
  guarding against an empty number input.
- Optional store tests and the optional All repos UI smoke test were not added;
  the behavior is covered by the mandatory pure-module tests plus the existing
  migration smoke test.

## Technical Decisions

- Onboarding is persisted after every step transition and when the wizard closes,
  so abrupt shutdowns do not lose progress.
- Replaying setup from the sidebar restarts at step 1 but preserves the completed
  flag, so replay never makes the app think setup is unfinished.
- Resuming an interrupted ready step returns to the groups step (`currentStep`
  capped at 2), preventing duplicate group creation.
- The wizard uses the existing Tailwind visual language rather than forcing
  design-system components where they do not fit the full-screen flow.
- `runRepoAction` reuses `executeGroupAction` with a single path and a
  concurrency limit of one, avoiding separate backend semantics for row actions.

## Testing Notes

All commands were run from the project root:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --check   # passed
cargo check --manifest-path src-tauri/Cargo.toml        # passed
cargo test --manifest-path src-tauri/Cargo.toml         # 11 passed
pnpm typecheck                                          # passed
pnpm test                                               # 30 passed (3 files)
pnpm build                                              # passed
```

The Rust suite includes legacy-config loading, onboarding normalization/step
validation, and theme/plugin/other kind detection. The frontend suite includes
28 new pure-logic tests.

## Usage Examples

- **First run:** launch the app with no scan paths or groups. The wizard opens at
  "Find projects"; enter a folder and press **Find my projects**.
- **Resume:** close the wizard partway through, restart the app, and setup resumes
  from the saved step and folder.
- **Replay:** use **Setup** at the bottom of the sidebar.
- **All repos:** select the first sidebar entry; combine status chips, group,
  type, and search. Use the row icons for Get latest / Send changes.
- **API:** with web mode running,

  ```bash
  curl -X POST http://127.0.0.1:8787/api/onboarding \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"started":true,"currentStep":1,"scanPath":"~/devel","ownerNames":["acme"],"groupingStyle":"topLevel","selectedRepoPaths":[]}'
  ```

## Documentation Updates

- `README.md` describes the first-run wizard, All repos filtering, Setup replay,
  the additive `onboarding` config block, and the new endpoint.
- `CHANGELOG.md` records the wizard, All repos page, kind detection, and transport
  additions under `## [Unreleased]`.
- `_ai/technical_decisions/README.md` already links ADR 261030-1 and ADR 261030-2;
  links were verified.

## Next Steps

Out of scope for this plan:

- Saved filter presets for All repos.
- Multi-folder management inside the onboarding wizard.
- Deeper vendor-specific kind detection (for example additional plugin manifests).
- Store-level tests once a stable backend mock harness is introduced.
