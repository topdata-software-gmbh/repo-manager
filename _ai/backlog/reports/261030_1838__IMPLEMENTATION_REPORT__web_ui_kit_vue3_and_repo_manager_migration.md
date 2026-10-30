---
filename: "_ai/backlog/reports/261030_1838__IMPLEMENTATION_REPORT__web_ui_kit_vue3_and_repo_manager_migration.md"
title: "Report: Web UI Kit Vue 3 and Repo Manager Migration"
createdAt: 2026-10-30 18:38
updatedAt: 2026-10-30 18:38
planFile: "_ai/backlog/active/261030_1838__IMPLEMENTATION_PLAN__web_ui_kit_vue3_and_repo_manager_migration.md"
specFile: ""
project: "repo-manager"
status: completed
filesCreated: 2
filesModified: 28
filesDeleted: 0
tags: [ui-kit, vue3, design-system, tokens, repo-manager, migration]
documentType: IMPLEMENTATION_REPORT
---

# Report: Web UI Kit Vue 3 and Repo Manager Migration

## 1. Summary

The web kit is now `@topdata-software-gmbh/ui-kit-web-vue3`, its component visuals
live in global token-only `.td-*` stylesheets, and it adds the nine elements the
app needs (`TdSelect`, `TdTextarea`, `TdCheckbox`, `TdBadge`, `TdAlert`,
`TdSpinner`, `TdMenu`, `TdMenuItem`, `TdSegmentedControl`). Every repo-manager
screen, modal, onboarding step, table, and helper was migrated onto the kit and
the token Tailwind preset.

The app is fully token-only: no raw palette classes, hard-coded hex values, or
native form controls remain in `src/` (outside tests). `pnpm typecheck`,
`pnpm test`, and `pnpm build` pass in both repositories.

## 2. Files Changed

### Design system (`/topdata/topdata-design-system`)

- Renamed `packages/ui` → `packages/ui-kit-web-vue3` and
  `@topdata-software-gmbh/ui` → `@topdata-software-gmbh/ui-kit-web-vue3`.
- Extracted all six existing SFC `<style scoped>` blocks into
  `src/styles/components/{button,input,form-field,modal,toast,toast-stack}.css`;
  added `{select,textarea,checkbox,badge,alert,spinner,menu,segmented-control}.css`.
- New components with tests: `TdSelect`, `TdTextarea`, `TdCheckbox`, `TdBadge`,
  `TdAlert`, `TdSpinner`, `TdMenu`, `TdMenuItem`, `TdSegmentedControl`, plus
  `menuContext.ts`, `types.ts`, and the `TdModal` `flush` prop/layout.
- New `src/styles.test.ts` aggregate token-only stylesheet check.
- Updated all six existing component tests to read their CSS file instead of the
  SFC source; added a flush test to `TdModal.test.ts`.
- Updated the playground (`package.json`, `main.ts`, `App.vue`) with examples for
  every new component and state.
- Updated `packages/README.md`, `packages/tokens/README.md`, the kit README,
  `ui-component-registry.yaml`, `SPEC__261029-1`, the specs index, and the
  design-system changelog.
- Added `_ai/specs/SPEC__261030-{1..8}`.

### Repo manager (`/topdata/repo-manager`)

- Dependency and imports renamed to the new kit; `tailwind.config.js` now uses
  the generated token preset.
- New `src/lib/__tests__/repoStatus.test.ts` and
  `src/lib/__tests__/token-only.test.ts`.
- Rewrote `App.vue`, `Sidebar.vue`, `Toolbar.vue`, `RepoTable.vue`,
  `RepoListTable.vue`, `AllReposPage.vue`, `GuidanceBanner.vue` and all modals /
  onboarding components against the kit.
- `groupDefaults.ts` stores semantic group-colour tokens with a legacy hex
  fallback; `repoStatus.ts` emits `TdBadge` variants instead of palette classes.
- `index.html`, `src/style.css`, README and changelog updated.

## 3. Key Changes

- **Global class layer:** every kit selector is a documented `.td-*` class built
  only from `--td-*` tokens. SFCs contain no `<style scoped>` block. A non-Vue
  consumer can import `styles.css` and use the same class names.
- **Nine new components:** implemented to the plan's API and state matrix, with
  unit tests, axe checks, specs, and playground entries.
- **Token migration:** app layout uses the token Tailwind preset
  (`bg-canvas`, `text-fg`, `border-border`, …) instead of slate/cyan/rose values.
- **Repo-manager migration:** all native form controls and buttons in app Vue
  files were replaced by kit components; status badges, filters, menus, loading
  states, and empty states use kit elements.
- **Enforcement:** `token-only.test.ts` fails on any raw Tailwind palette class or
  hex colour in app source.

## 4. Deviations from Plan

- **Vitest CSS processing.** Both `vitest.config.ts` files now set `css: true`.
  Vitest's default `css: false` stubs CSS modules, so `import … from "*.css?raw"`
  and `import.meta.glob(..., { query: "?raw" })` returned empty strings. With
  `css: true` the raw content is available and the token-only tests work.
- **`TdMenu` uncontrolled default.** Vue coerces an absent Boolean prop to
  `false`, which made `controlled` always true and broke the uncontrolled
  `<TdMenu>` usage the toolbar needs. The prop is declared with an explicit
  `open: undefined` default so omission is preserved.
- **`TdMenu` typed emit.** `emit(next ? "open" : "close")` does not satisfy the
  union overload; it is split into separate `emit("open")` / `emit("close")`
  branches.
- **HelpModal header.** The default `TdModal` header is used (title/description
  props) instead of a custom header, which keeps the accessible name logic and
  close button intact.
- **Group delete confirmation.** `GroupEditorModal` still uses a native
  `window.confirm` for the destructive confirmation, as the plan allows; replacing
  it with a nested confirm panel remains a follow-up.
- **Onboarding padding.** With `flush`, the modal body drops padding, so the
  wizard's `<main>` re-adds `px-6 py-8` to preserve the old layout.
- **Theme attribute.** `index.html` uses `data-theme="dark"` (the token contract)
  instead of the previous Tailwind `class="dark"`.
- **Manual smoke test.** The visual/keyboard smoke list was not executed in this
  headless environment; the automated suites and production build pass. The
  checklist is reproduced below for a human pass.

## 5. Technical Decisions

- `:deep(...)` selectors were unwrapped during extraction (for example
  `.td-button__icon :deep(svg)` → `.td-button__icon svg`), which is equivalent
  once the styles are global.
- The kit remains presentational: menu open state, selection, and app data stay
  in the SFC/composables; no store or Tauri API is imported.
- `TdMenu` keeps controlled (`open` prop / `update:open`) and uncontrolled modes,
  with outside-pointer dismissal, Escape with focus restore, and
  Arrow/Home/End traversal that skips disabled items.
- Group colours are stored as token names (`primary`, `accent`, …) so brand token
  changes propagate; `groupColorStyle` still renders legacy hex values from
  existing config files.
- Token-only styling outside the kit is enforced by scanning raw source, not by a
  lint plugin, so the rule travels with the repo and runs in `pnpm test`.

## 6. Testing Notes

### Design system

```bash
cd /topdata/topdata-design-system
pnpm install
pnpm build        # tokens, ui-kit-web-vue3, playground
pnpm typecheck
pnpm test         # ui-kit: 17 files / 170 tests; tokens: 6 tests; playground: no tests
```

Results: all passed. The kit build emits `dist/topdata-ui-kit.css`.

### Repo manager

```bash
cd /topdata/repo-manager
pnpm install
pnpm typecheck
pnpm test         # 5 files / 34 tests
pnpm build        # vue-tsc + vite build
```

Results: all passed. `token-only.test.ts` passed against the final source.

### Manual smoke checklist (recommended)

1. First-run onboarding: scan folder, choose repos, confirm groups, ready.
2. Group editor: create/edit/delete, colour swatches, advanced checkboxes.
3. Toolbar: branch select, Save/Get latest/Send changes, rescan, More menu
   keyboard navigation, help.
4. All repos: search, group/type selects, status filters, clear filters, empty
   states.
5. Save changes: loading, file changes, error alert, save.
6. Help modal: Escape and backdrop close.
7. Toasts: success, error, info, warning, dismiss.
8. Theme/token spot check: TopData blue primary, token surfaces, no slate/cyan.

## 7. Usage Examples

```vue
<TdMenu aria-label="More Git actions">
  <template #trigger="{ open }">
    <TdButton variant="ghost" size="sm" aria-label="More Git actions" :aria-expanded="open">
      <template #icon-left><MoreHorizontal /></template>
    </TdButton>
  </template>
  <TdMenuItem @select="runAction('fetch')">
    <template #icon><RefreshCw /></template>
    Check for updates
    <template #description>Looks for new work without changing your files.</template>
  </TdMenuItem>
</TdMenu>
```

```vue
<TdSegmentedControl
  v-model="statusFilter"
  :options="statusOptionsWithCounts"
  aria-label="Filter by status"
/>
```

```vue
<section class="flex flex-1 flex-col border-b border-border bg-surface text-fg">
  <TdBadge variant="success">Up to date</TdBadge>
  <TdButton variant="primary" @click="store.openSaveChanges()">Save to Git</TdButton>
</section>
```

## 8. Documentation Updates

- Design-system `README.md`, `packages/README.md`, `packages/tokens/README.md`,
  and the kit README now describe the new package, import order, global class
  contract, component table, and deferred work.
- `ui-component-registry.yaml` records `topdata-vue` as active with the new
  package/source and the global-class note.
- `SPEC__261029-1` gained a Global Class Layer requirement and component
  traceability rows; `_ai/specs/index.md` lists the eight new specs.
- Both changelogs record the rename, global classes, nine components, and app
  migration.
- ADR `261030_1838__ADR__web-ui-kit-vue3-token-only-styling.md` already records
  the decision (no new ADR was required).

## 9. Next Steps

- `ui-kit-web-tailwind` for non-Vue web apps, reusing the same `.td-*` contract.
- Brand variant packages, design/adapter schemas, and the in-app brand switcher.
- Optional `ui-kit-web-vue3-layout` (AppShell, Sidebar, Toolbar, DataTable page).
- TradeGuard migration under its own one-kit-per-surface ADR; it stays on the
  legacy Vuetify kit until then.
- Replace the remaining native `window.confirm` in the group editor with a
  `TdModal` confirm state.
