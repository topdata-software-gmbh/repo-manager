---
filename: "_ai/backlog/active/261030_1838__IMPLEMENTATION_PLAN__web_ui_kit_vue3_and_repo_manager_migration.md"
title: "Web UI Kit Vue 3 and Repo Manager Migration"
createdAt: 2026-10-30 18:38
updatedAt: 2026-10-30 19:51
status: draft
priority: high
tags: [ui-kit, vue3, design-system, tokens, tailwind, repo-manager, migration, accessibility]
estimatedComplexity: complex
documentRevision: 2
sourceSpec: ""
sourceHandoff: ""
sourceAdrs:
  [
    "/topdata/topdata-design-system/_ai/technical_decisions/261001_1750__ADR__normalized-brand-manifests.md",
    "/topdata/topdata-design-system/_ai/technical_decisions/261029_0240__ADR__ui-component-registry.md",
    "/topdata/topdata-design-system/_ai/technical_decisions/261029_1100__ADR__tauri-vue-ui-kit-packages.md",
    "/topdata/topdata-design-system/_ai/technical_decisions/261030_1838__ADR__web-ui-kit-vue3-token-only-styling.md",
  ]
documentType: IMPLEMENTATION_PLAN
---

# Web UI Kit Vue 3 and Repo Manager Migration

Sources: design discussion in this session plus the four ADRs above. There is no
feature spec and no design handoff for this work.

## Problem

`repo-manager` is the only Vue/Tauri app using the shared UI kit, and it only
uses two of the kit's six components. Everything else is hand-rolled Tailwind
with a slate/cyan palette that contradicts the generated TopData tokens
(`#5085A9` primary, `#202020` surfaces). The kit itself still uses scoped SFC
styles, so its visuals cannot be reused by a plain-HTML app or extracted into a
future `ui-kit-web-tailwind`.

The result is exactly the inconsistency the design system exists to prevent: two
palettes, two sets of interaction patterns, and no enforced contract for future
apps such as `tradeguard-app`.

## Executive Summary

This plan makes the web kit the single visual source for Vue web/Tauri surfaces
and migrates `repo-manager` completely onto it.

1. Rename `@topdata-software-gmbh/ui` to
   `@topdata-software-gmbh/ui-kit-web-vue3`.
2. Move every component's styles out of `<style scoped>` into global, token-only
   `.td-*` CSS. The class names become the public visual contract and can later
   be extracted into `ui-kit-web-tailwind` without changing markup.
3. Add only the elements repo-manager needs: `TdSelect`, `TdTextarea`,
   `TdCheckbox`, `TdBadge`, `TdAlert`, `TdSpinner`, `TdMenu`, `TdMenuItem`, and
   `TdSegmentedControl`. `TdButton` already covers icon-only usage.
4. Migrate every repo-manager component, modal, onboarding step, table, and
   helper to the kit plus token-backed utilities. No raw Tailwind palette values
   remain outside the kit.
5. Add component specs, unit/a11y tests, playground entries, a token-only source
   test, registry/docs updates, and a changelog.
6. Defer brand variants, adapter schemas, the in-app brand switcher,
   `ui-kit-web-tailwind`, and the optional layout kit. They are recorded in ADR
   `261030-1` so later work starts from a decision instead of a blank page.

The work is intentionally broad in repo-manager but narrow in new infrastructure:
one kit rename, one CSS-layer move, nine small components, and one app migration.

## Definition of Done

- `@topdata-software-gmbh/ui-kit-web-vue3` is the only web Vue kit; no consumer
  imports the old package name.
- The kit exports global `.td-*` CSS and nine new elements with specs, tests,
  axe checks, and playground entries.
- Every repo-manager screen uses kit controls; no native form controls and no raw
  palette values remain in app Vue/TS/CSS source.
- `pnpm build`, `typecheck`, and `test` pass in both repositories.
- The registry, READMEs, specs index, and changelogs describe the new package and
  the deferred architecture.
- The implementation report records the outcome and any deviations.

## Design Decisions & Rejected Alternatives

| Decision | Rationale | ADR |
|---|---|---|
| Rename the kit to `@topdata-software-gmbh/ui-kit-web-vue3` | Names the medium and stack and reserves naming space for CLI/TUI/Tailwind kits. | `261030-1` |
| Global `.td-*` classes instead of scoped SFC styles | Makes the visual layer reusable by non-Vue apps and extractable to `ui-kit-web-tailwind` later. | `261030-1` |
| Token-only styling outside the kit; no raw palette | Removes the slate/cyan drift and makes brand-token changes propagate. | `261030-1`, `261001-1` |
| One kit per surface | Existing registry rule; repo-manager moves to the new kit, TradeGuard stays legacy until its own ADR. | `261029-1` |
| Add only nine proven components | YAGNI; `TdTable`, `TdEmptyState`, `TdTooltip`, `TdRadio`, and `TdIconButton` have no consumer. | `261030-1` |
| Store group colours as semantic token names, keep legacy hex fallback | Removes raw palette defaults without breaking existing user config files. | `261030-1` |
| Defer brand variants, adapter YAML/schema, switcher, Tailwind kit, layout kit | No second brand/medium consumer exists; the contract would be guessed. | `261030-1` |

Rejected alternatives:

- Keep `@topdata-software-gmbh/ui`: hides the stack and blocks the future naming
  scheme.
- Build `ui-theme-topdata` / brand variants now: speculative; does not fix the
  current inconsistency.
- Build design/adapter schemas and the switcher first: same reason, plus it
  delays the actual migration.
- Keep scoped styles and extract later: would make the later Tailwind kit a
  rewrite.
- Keep raw Tailwind palette classes in app layout: recreates the drift.
- Migrate TradeGuard in the same effort: it is a different surface, currently
  Vuetify + Options API, and needs its own one-kit-per-surface ADR.

## Project Environment

```text
Workspace: /topdata/repo-manager (primary app), /topdata/topdata-design-system (kit)
Node/pnpm: pnpm 12 workspace; Node 26+ required by the token build (per ADR 261029-2)
Vue: 3.4+ with <script setup lang="ts"> and Composition API
Styling: Tailwind 3 + generated --td-* tokens; kit ships compiled CSS
Tests: Vitest + @vue/test-utils + happy-dom + axe-core
Kit package: packages/ui-kit-web-vue3 (was packages/ui)
App package: repo-manager, dependency linked to ../topdata-design-system/packages/ui-kit-web-vue3
Git workflow: repository history commits directly on main; do not create feature branches or PRs
```

The plan will be implemented by an AI coding agent. Follow SOLID principles,
keep component behavior separate from styling and app state, and prefer the
smallest change that satisfies the phase. Work phase by phase, run the
validation commands after each phase, and do not start the next phase until its
exit criteria are met. Do not amend published ADRs; the new ADR supersedes only
the naming/styling parts of `261029-2`. Do not edit generated token files.

---

## Phase 1 — Rename the kit and move styles to global classes

**Goal:** `@topdata-software-gmbh/ui-kit-web-vue3` exists, every consumer imports
it, and existing components render global `.td-*` classes instead of scoped CSS.

**Files:**

- [MODIFY] `/topdata/topdata-design-system/packages/ui/package.json` (renamed)
- [MODIFY] `/topdata/topdata-design-system/packages/ui/vite.config.ts` (renamed)
- [MODIFY] `/topdata/topdata-design-system/packages/ui/src/index.ts` (renamed)
- [NEW FILE] `/topdata/topdata-design-system/packages/ui/src/styles/components/*.css`
- [MODIFY] all six existing SFCs to remove `<style scoped>`
- [MODIFY] all six existing component tests to read the new CSS files
- [MODIFY] `/topdata/topdata-design-system/playground/package.json`
- [MODIFY] `/topdata/topdata-design-system/playground/src/main.ts`
- [MODIFY] `/topdata/topdata-design-system/playground/src/App.vue`
- [MODIFY] `/topdata/topdata-design-system/packages/README.md`
- [MODIFY] `/topdata/topdata-design-system/packages/ui-component-registry.yaml`
- [MODIFY] `/topdata/repo-manager/package.json`
- [MODIFY] `/topdata/repo-manager/src/main.ts`
- [MODIFY] `/topdata/repo-manager/src/components/AddScanPathModal.vue`
- [MODIFY] `/topdata/repo-manager/src/components/ToastStack.vue`
- [MODIFY] `/topdata/repo-manager/src/components/__tests__/migration.smoke.test.ts`

**Steps:**

1. Rename the directory and update the package name:

```bash
git mv packages/ui packages/ui-kit-web-vue3
```

[MODIFY] `packages/ui-kit-web-vue3/package.json`

```json
{
  "name": "@topdata-software-gmbh/ui-kit-web-vue3",
  "version": "0.1.0",
  "type": "module",
  "files": ["dist"],
  "exports": {
    ".": { "types": "./dist/index.d.ts", "import": "./dist/index.js" },
    "./styles.css": "./dist/topdata-ui-kit.css"
  },
  "scripts": { "build": "vite build", "test": "vitest run", "typecheck": "vue-tsc --noEmit" },
  "dependencies": { "@topdata-software-gmbh/design-tokens": "workspace:*" },
  "peerDependencies": { "vue": "^3.4.30" }
}
```

Keep the existing `devDependencies` block unchanged; the snippet only shows the
fields that change.

[MODIFY] `packages/ui-kit-web-vue3/vite.config.ts`

```ts
output: {
  assetFileNames: (assetInfo) =>
    assetInfo.name?.endsWith(".css")
      ? "topdata-ui-kit.css"
      : "assets/[name][extname]",
},
```

2. Create the global CSS layer. One file per component, named after the
component. Move the complete contents of each SFC `<style scoped>` block,
remove `:deep(...)` wrappers (`.td-button__icon :deep(svg)` becomes
`.td-button__icon svg`), and keep every selector under `.td-*`.

[NEW FILE] `packages/ui-kit-web-vue3/src/styles/components/button.css`

The file below is the **complete** contents of the current `TdButton.vue`
`<style scoped>` block, with `:deep(...)` removed. Do not stop at the first
visible rules: the `:active` states, spinner, keyframes, and reduced-motion
block are required by `TdButton.test.ts`.

```css
.td-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--td-space-2);
  max-width: 100%;
  border: 1px solid transparent;
  border-radius: var(--td-radius-md);
  font-family: var(--td-font-body);
  font-weight: var(--td-weight-semibold);
  line-height: var(--td-leading-tight);
  cursor: pointer;
  transition:
    background-color var(--td-motion-fast) var(--td-ease-standard),
    border-color var(--td-motion-fast) var(--td-ease-standard),
    color var(--td-motion-fast) var(--td-ease-standard);
}

.td-button--sm {
  padding: var(--td-space-1) var(--td-space-2);
  font-size: var(--td-text-xs);
}

.td-button--md {
  padding: var(--td-space-2) var(--td-space-3);
  font-size: var(--td-text-sm);
}

.td-button--lg {
  padding: var(--td-space-3) var(--td-space-4);
  font-size: var(--td-text-base);
}

.td-button--primary {
  background-color: var(--td-color-primary);
  color: var(--td-color-on-primary);
}

.td-button--primary:hover:not(:disabled) {
  background-color: var(--td-color-primary-hover);
}

.td-button--primary:active:not(:disabled) {
  background-color: var(--td-color-primary-active);
}

.td-button--secondary {
  background-color: var(--td-color-surface-raised);
  border-color: var(--td-color-border);
  color: var(--td-color-fg);
}

.td-button--secondary:hover:not(:disabled) {
  background-color: var(--td-color-surface-hover);
  border-color: var(--td-color-border-strong);
}

.td-button--secondary:active:not(:disabled) {
  background-color: var(--td-color-surface-hover);
}

.td-button--ghost {
  background-color: transparent;
  color: var(--td-color-fg-muted);
}

.td-button--ghost:hover:not(:disabled) {
  background-color: var(--td-color-surface-hover);
  color: var(--td-color-fg);
}

.td-button--ghost:active:not(:disabled) {
  background-color: var(--td-color-surface-hover);
}

.td-button--danger {
  background-color: var(--td-color-error);
  color: var(--td-color-on-error);
}

.td-button--danger:hover:not(:disabled) {
  background-color: var(--td-color-error-hover);
}

.td-button--danger:active:not(:disabled) {
  background-color: var(--td-color-error-hover);
}

.td-button:disabled {
  background-color: var(--td-color-disabled);
  border-color: transparent;
  color: var(--td-color-disabled-fg);
  cursor: not-allowed;
}

.td-button:focus-visible {
  outline: 2px solid var(--td-color-focus);
  outline-offset: 2px;
}

.td-button--block {
  width: 100%;
}

.td-button__label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.td-button__icon,
.td-button__spinner {
  display: inline-flex;
  flex: none;
  align-items: center;
  justify-content: center;
  width: 1em;
  height: 1em;
}

.td-button__icon svg {
  width: 1em;
  height: 1em;
}

.td-button__spinner {
  border: 2px solid currentColor;
  border-right-color: transparent;
  border-radius: var(--td-radius-full);
  animation: td-button-spin 700ms linear infinite;
}

@keyframes td-button-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (prefers-reduced-motion: reduce) {
  .td-button {
    transition: none;
  }

  .td-button__spinner {
    animation: none;
    border-right-color: currentColor;
    opacity: 0.6;
  }
}

.td-button--icon-only.td-button--sm {
  padding: var(--td-space-1);
}

.td-button--icon-only.td-button--md {
  padding: var(--td-space-2);
}

.td-button--icon-only.td-button--lg {
  padding: var(--td-space-3);
}
```

Repeat for `input.css`, `form-field.css`, `modal.css`, `toast.css`, and
`toast-stack.css`. Preserve the current rules exactly; only remove scoping.

3. Import the CSS from the package entry so Vite bundles it into
`topdata-ui-kit.css`:

[MODIFY] `packages/ui-kit-web-vue3/src/index.ts`

```ts
import "./styles/base.css";
import "./styles/components/button.css";
import "./styles/components/input.css";
import "./styles/components/form-field.css";
import "./styles/components/modal.css";
import "./styles/components/toast.css";
import "./styles/components/toast-stack.css";

export { default as TdButton } from "./components/TdButton.vue";
export { default as TdInput } from "./components/TdInput.vue";
export { default as TdFormField } from "./components/TdFormField.vue";
export { default as TdModal } from "./components/TdModal.vue";
export { default as TdToast } from "./components/TdToast.vue";
export { default as TdToastStack } from "./components/TdToastStack.vue";

export { useTheme, THEME_STORAGE_KEY } from "./composables/useTheme";
export { useToast } from "./composables/useToast";
export type { Toast, ToastKind, ToastOptions, UseToast } from "./composables/useToast";
export type { ThemeMode, ThemeStorage, UseTheme, UseThemeOptions } from "./composables/useTheme";
```

`types.ts` is added in Phase 2; do not export it before it exists.

4. Remove the `<style scoped>` block from each SFC. The templates and class
bindings stay byte-for-byte equivalent. Example for `TdButton.vue`:

[MODIFY] `packages/ui-kit-web-vue3/src/components/TdButton.vue`

```diff
-<style scoped>
-.td-button { ... }
-...
-</style>
```

Do the same for `TdInput.vue`, `TdFormField.vue`, `TdModal.vue`, `TdToast.vue`,
and `TdToastStack.vue`.

4a. Add a `flush` option to `TdModal` for the full-screen onboarding wizard.
Keep the default inset dialog unchanged:

[MODIFY] `packages/ui-kit-web-vue3/src/components/TdModal.vue`

```diff
 const props = withDefaults(
   defineProps<{
     open: boolean;
     title?: string;
     description?: string;
     size?: "sm" | "md" | "lg" | "full";
+    flush?: boolean;
     closeOnEscape?: boolean;
```

```diff
 const classes = computed(() => [
   "td-modal__dialog",
   `td-modal__dialog--${props.size}`,
+  { "td-modal__dialog--flush": props.flush },
 ]);
```

```diff
       <div
         v-if="props.open"
-        class="td-modal"
+        class="td-modal"
+        :class="{ 'td-modal--flush': props.flush }"
         @click="onBackdropClick"
         @mousedown.self.prevent
       >
```

[APPEND TO] `packages/ui-kit-web-vue3/src/styles/components/modal.css`

`modal.css` was already created in step 2 by moving the complete `TdModal.vue`
`<style scoped>` block. **Append** the flush rules below; do not overwrite the
moved base rules (`.td-modal`, `.td-modal__dialog`, `.td-modal__body`, media
queries), or `TdModal.test.ts`/the modal visuals break.

```css
/* Append after the moved TdModal rules. */
.td-modal--flush {
  padding: 0;
}

.td-modal--flush .td-modal__body {
  padding: 0;
}

.td-modal__dialog--flush {
  max-width: 100%;
  height: 100%;
  max-height: 100%;
  border: 0;
  border-radius: 0;
}
```

Add `flush?: boolean` to the `TdModal` README API table and playground, with a
full-screen example. Add a test that asserts `td-modal--flush` and
`td-modal__dialog--flush` appear only when `flush` is true.

5. Update tests that scan component source for token usage. They must scan the
CSS file instead of the SFC:

[MODIFY] `packages/ui-kit-web-vue3/src/components/TdButton.test.ts`

```ts
import source from "../styles/components/button.css?raw";
```

Do the same for `TdInput.test.ts`, `TdFormField.test.ts`, `TdModal.test.ts`,
`TdToast.test.ts`, and `TdToastStack.test.ts`. Add a new aggregate test:

[NEW FILE] `packages/ui-kit-web-vue3/src/styles.test.ts`

```ts
import { describe, expect, it } from "vitest";

const modules = import.meta.glob("./styles/**/*.css", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

describe("kit styles", () => {
  it("contains every expected component stylesheet", () => {
    expect(Object.keys(modules)).toEqual(
      expect.arrayContaining([
        "./styles/base.css",
        "./styles/components/button.css",
        "./styles/components/input.css",
        "./styles/components/form-field.css",
        "./styles/components/modal.css",
        "./styles/components/toast.css",
        "./styles/components/toast-stack.css",
      ]),
    );
  });

  it("uses only semantic tokens and no raw colours", () => {
    for (const [file, css] of Object.entries(modules)) {
      expect(css, file).not.toMatch(/#[0-9a-f]{3,8}\b/i);
      expect(css, file).not.toMatch(/\b(?:rgb|hsl)a?\(/i);
    }
  });
});
```

6. Update all non-template consumers of the old package name. Search first, then
replace only the bare package name, not `ui-kit-web-vue3`:

```bash
rg -nP "@topdata-software-gmbh/ui(?![-\w])" /topdata/topdata-design-system /topdata/repo-manager \
  --glob "!**/_ai/backlog/archive/**" --glob "!**/_ai/technical_decisions/**"
```

Historical plans and accepted ADRs keep the old name; do not rewrite them.

[MODIFY] `playground/package.json`

```json
"dependencies": {
  "@topdata-software-gmbh/design-tokens": "workspace:*",
  "@topdata-software-gmbh/ui-kit-web-vue3": "workspace:*",
  "vue": "^3.4.30"
}
```

[MODIFY] `playground/src/main.ts`

```ts
import "@topdata-software-gmbh/design-tokens/tokens.css";
import "@topdata-software-gmbh/ui-kit-web-vue3/styles.css";
```

[MODIFY] `playground/src/App.vue`

```ts
import {
  TdButton,
  TdFormField,
  TdInput,
  TdModal,
  TdToastStack,
  useTheme,
  useToast,
  type ToastKind,
} from "@topdata-software-gmbh/ui-kit-web-vue3";
```

[MODIFY] `/topdata/repo-manager/package.json`

```json
"@topdata-software-gmbh/ui-kit-web-vue3": "link:../topdata-design-system/packages/ui-kit-web-vue3",
```

[MODIFY] `/topdata/repo-manager/src/main.ts`

```ts
import "@topdata-software-gmbh/design-tokens/tokens.css";
import "@topdata-software-gmbh/ui-kit-web-vue3/styles.css";
```

[MODIFY] `AddScanPathModal.vue`, `ToastStack.vue`, and
`migration.smoke.test.ts` imports:

```ts
import { TdButton, TdFormField, TdInput, TdModal } from "@topdata-software-gmbh/ui-kit-web-vue3";
import { TdToastStack } from "@topdata-software-gmbh/ui-kit-web-vue3";
```

7. Update `ui-component-registry.yaml` package, source, and notes. Keep the kit
id `topdata-vue` to avoid rule churn:

```yaml
- id: topdata-vue
  name: TopData Web UI (Vue 3)
  medium: webview
  stack: [vue, typescript, tauri]
  package: "@topdata-software-gmbh/ui-kit-web-vue3"
  version: "0.1.0"
  status: active
  owner: design-system
  source: packages/ui-kit-web-vue3
  tokens: [css]
  consumers:
    - app: repo-manager
      surface: desktop-window
  notes: "Global .td-* classes built only from --td-* tokens. Tailwind kit and brand variants deferred by ADR 261030-1."
```

**Validation:**

```bash
cd /topdata/topdata-design-system
pnpm install
pnpm --filter @topdata-software-gmbh/ui-kit-web-vue3 build
pnpm --filter @topdata-software-gmbh/ui-kit-web-vue3 test
pnpm --filter @topdata-software-gmbh/ui-kit-web-vue3 typecheck
pnpm --filter @topdata-software-gmbh/playground typecheck
cd /topdata/repo-manager
pnpm install
pnpm typecheck
pnpm test
```

**Exit criteria:**

- No `packages/ui` directory remains.
- No `@topdata-software-gmbh/ui` import remains outside historical docs/ADRs.
- The kit build emits `dist/topdata-ui-kit.css`.
- `rg "<style" packages/ui-kit-web-vue3/src/components` returns nothing.
- The aggregate styles test passes.

---

## Phase 2 — Add form elements

**Goal:** Add `TdSelect`, `TdTextarea`, and `TdCheckbox` with the same API and
accessibility conventions as the existing kit.

### Shared types

[NEW FILE] `packages/ui-kit-web-vue3/src/types.ts`

```ts
export interface TdSelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface TdSegmentOption {
  value: string;
  label: string;
  count?: number | string;
  disabled?: boolean;
}
```

### TdSelect

[NEW FILE] `packages/ui-kit-web-vue3/src/components/TdSelect.vue`

```vue
<script setup lang="ts">
import { computed, useAttrs } from "vue";
import type { TdSelectOption } from "../types";

defineOptions({ inheritAttrs: false });

const props = withDefaults(
  defineProps<{
    modelValue?: string;
    options?: TdSelectOption[];
    placeholder?: string;
    size?: "sm" | "md" | "lg";
    disabled?: boolean;
    invalid?: boolean;
    required?: boolean;
  }>(),
  { size: "md", options: () => [] },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
  change: [event: Event];
}>();

const attrs = useAttrs();

const classes = computed(() => [
  "td-select",
  `td-select--${props.size}`,
  {
    "td-select--disabled": props.disabled,
    "td-select--invalid": props.invalid,
  },
]);

function onChange(event: Event): void {
  emit("update:modelValue", (event.target as HTMLSelectElement).value);
  emit("change", event);
}
</script>

<template>
  <span :class="classes">
    <select
      v-bind="attrs"
      class="td-select__control"
      :value="props.modelValue ?? ''"
      :disabled="props.disabled"
      :required="props.required"
      :aria-invalid="props.invalid || undefined"
      @change="onChange"
    >
      <option v-if="props.placeholder" value="" disabled>{{ props.placeholder }}</option>
      <slot>
        <option
          v-for="option in props.options"
          :key="option.value"
          :value="option.value"
          :disabled="option.disabled"
        >
          {{ option.label }}
        </option>
      </slot>
    </select>
    <svg class="td-select__chevron" viewBox="0 0 20 20" aria-hidden="true" focusable="false">
      <path
        d="M6 8l4 4 4-4"
        fill="none"
        stroke="currentColor"
        stroke-width="1.75"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </span>
</template>
```

[NEW FILE] `packages/ui-kit-web-vue3/src/styles/components/select.css`

```css
.td-select {
  position: relative;
  display: inline-flex;
  width: 100%;
  min-width: 0;
  border: 1px solid var(--td-color-border);
  border-radius: var(--td-radius-md);
  background-color: var(--td-color-surface);
  color: var(--td-color-fg);
  transition:
    border-color var(--td-motion-fast) var(--td-ease-standard),
    box-shadow var(--td-motion-fast) var(--td-ease-standard);
}

.td-select:hover:not(.td-select--disabled) {
  border-color: var(--td-color-border-strong);
}

.td-select:focus-within {
  border-color: var(--td-color-focus);
  box-shadow: 0 0 0 2px var(--td-color-focus);
}

.td-select--invalid {
  border-color: var(--td-color-error);
}

.td-select--invalid:focus-within {
  box-shadow: 0 0 0 2px var(--td-color-error);
}

.td-select--disabled {
  background-color: var(--td-color-disabled);
  color: var(--td-color-disabled-fg);
  cursor: not-allowed;
}

.td-select--sm {
  font-size: var(--td-text-xs);
}

.td-select--md {
  font-size: var(--td-text-sm);
}

.td-select--lg {
  font-size: var(--td-text-base);
}

.td-select__control {
  width: 100%;
  min-width: 0;
  appearance: none;
  border: 0;
  background: transparent;
  color: inherit;
  cursor: inherit;
  outline: none;
}

.td-select--sm .td-select__control {
  padding: var(--td-space-1) var(--td-space-6) var(--td-space-1) var(--td-space-2);
}

.td-select--md .td-select__control {
  padding: var(--td-space-2) var(--td-space-8) var(--td-space-2) var(--td-space-3);
}

.td-select--lg .td-select__control {
  padding: var(--td-space-3) var(--td-space-8) var(--td-space-3) var(--td-space-4);
}

.td-select__chevron {
  position: absolute;
  right: var(--td-space-3);
  top: 50%;
  width: 1rem;
  height: 1rem;
  transform: translateY(-50%);
  color: var(--td-color-fg-muted);
  pointer-events: none;
}

.td-select__control option {
  background-color: var(--td-color-surface);
  color: var(--td-color-fg);
}
```

### TdTextarea

[NEW FILE] `packages/ui-kit-web-vue3/src/components/TdTextarea.vue`

```vue
<script setup lang="ts">
import { computed, useAttrs } from "vue";

defineOptions({ inheritAttrs: false });

const props = withDefaults(
  defineProps<{
    modelValue?: string | null;
    rows?: number;
    size?: "sm" | "md" | "lg";
    disabled?: boolean;
    readonly?: boolean;
    invalid?: boolean;
    required?: boolean;
  }>(),
  { rows: 3, size: "md" },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
  focus: [event: FocusEvent];
  blur: [event: FocusEvent];
}>();

const attrs = useAttrs();

const classes = computed(() => [
  "td-textarea",
  `td-textarea--${props.size}`,
  {
    "td-textarea--disabled": props.disabled,
    "td-textarea--readonly": props.readonly,
    "td-textarea--invalid": props.invalid,
  },
]);

function onInput(event: Event): void {
  if (props.disabled || props.readonly) return;
  emit("update:modelValue", (event.target as HTMLTextAreaElement).value);
}
</script>

<template>
  <textarea
    v-bind="attrs"
    :class="classes"
    :value="props.modelValue ?? ''"
    :rows="props.rows"
    :disabled="props.disabled"
    :readonly="props.readonly"
    :required="props.required"
    :aria-invalid="props.invalid || undefined"
    @input="onInput"
    @focus="emit('focus', $event)"
    @blur="emit('blur', $event)"
  />
</template>
```

[NEW FILE] `packages/ui-kit-web-vue3/src/styles/components/textarea.css`

```css
.td-textarea {
  display: block;
  width: 100%;
  min-width: 0;
  resize: vertical;
  border: 1px solid var(--td-color-border);
  border-radius: var(--td-radius-md);
  background-color: var(--td-color-surface);
  color: var(--td-color-fg);
  font-family: var(--td-font-body);
  line-height: var(--td-leading-relaxed);
  transition:
    border-color var(--td-motion-fast) var(--td-ease-standard),
    box-shadow var(--td-motion-fast) var(--td-ease-standard);
}

.td-textarea::placeholder {
  color: var(--td-color-fg-dim);
}

.td-textarea:hover:not(.td-textarea--disabled) {
  border-color: var(--td-color-border-strong);
}

.td-textarea:focus {
  outline: none;
  border-color: var(--td-color-focus);
  box-shadow: 0 0 0 2px var(--td-color-focus);
}

.td-textarea--invalid {
  border-color: var(--td-color-error);
}

.td-textarea--invalid:focus {
  box-shadow: 0 0 0 2px var(--td-color-error);
}

.td-textarea--disabled {
  background-color: var(--td-color-disabled);
  color: var(--td-color-disabled-fg);
  cursor: not-allowed;
}

.td-textarea--sm {
  padding: var(--td-space-1) var(--td-space-2);
  font-size: var(--td-text-xs);
}

.td-textarea--md {
  padding: var(--td-space-2) var(--td-space-3);
  font-size: var(--td-text-sm);
}

.td-textarea--lg {
  padding: var(--td-space-3) var(--td-space-4);
  font-size: var(--td-text-base);
}
```

### TdCheckbox

[NEW FILE] `packages/ui-kit-web-vue3/src/components/TdCheckbox.vue`

```vue
<script setup lang="ts">
import { computed } from "vue";
import { useId } from "../composables/useId";

const props = withDefaults(
  defineProps<{
    modelValue?: boolean;
    id?: string;
    label?: string;
    description?: string;
    ariaLabel?: string;
    disabled?: boolean;
    invalid?: boolean;
  }>(),
  { modelValue: false },
);

const emit = defineEmits<{ "update:modelValue": [value: boolean] }>();

const generatedId = useId("td-checkbox");
const inputId = computed(() => props.id ?? generatedId);
const descriptionId = computed(() => `${inputId.value}-description`);
const classes = computed(() => [
  "td-checkbox",
  {
    "td-checkbox--disabled": props.disabled,
    "td-checkbox--invalid": props.invalid,
  },
]);

function onChange(event: Event): void {
  if (props.disabled) return;
  emit("update:modelValue", (event.target as HTMLInputElement).checked);
}
</script>

<template>
  <label :class="classes" :for="inputId">
    <input
      :id="inputId"
      class="td-checkbox__control"
      type="checkbox"
      :checked="props.modelValue"
      :disabled="props.disabled"
      :aria-label="props.ariaLabel"
      :aria-invalid="props.invalid || undefined"
      :aria-describedby="props.description ? descriptionId : undefined"
      @change="onChange"
    />
    <span v-if="props.label || $slots.default" class="td-checkbox__body">
      <span class="td-checkbox__label"><slot>{{ props.label }}</slot></span>
      <span v-if="props.description" :id="descriptionId" class="td-checkbox__description">
        {{ props.description }}
      </span>
    </span>
  </label>
</template>
```

[NEW FILE] `packages/ui-kit-web-vue3/src/styles/components/checkbox.css`

```css
.td-checkbox {
  display: inline-flex;
  align-items: flex-start;
  gap: var(--td-space-2);
  color: var(--td-color-fg);
  cursor: pointer;
}

.td-checkbox--disabled {
  color: var(--td-color-disabled-fg);
  cursor: not-allowed;
}

.td-checkbox__control {
  flex: none;
  width: 1rem;
  height: 1rem;
  margin: 0.1rem 0 0;
  accent-color: var(--td-color-primary);
  cursor: inherit;
}

.td-checkbox--invalid .td-checkbox__control {
  outline: 1px solid var(--td-color-error);
  outline-offset: 2px;
}

.td-checkbox__body {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: var(--td-space-1);
}

.td-checkbox__label {
  font-size: var(--td-text-sm);
  line-height: var(--td-leading-tight);
}

.td-checkbox__description {
  color: var(--td-color-fg-muted);
  font-size: var(--td-text-xs);
  line-height: var(--td-leading-relaxed);
}
```

### Wiring, specs, tests, playground

Add imports to `src/index.ts`, export the shared option types, and render
examples in `playground/src/App.vue` for every state: default, disabled,
invalid, required, sizes, placeholder, description, and `v-model`.

```ts
import "./styles/components/select.css";
import "./styles/components/textarea.css";
import "./styles/components/checkbox.css";

export { default as TdSelect } from "./components/TdSelect.vue";
export { default as TdTextarea } from "./components/TdTextarea.vue";
export { default as TdCheckbox } from "./components/TdCheckbox.vue";
export type { TdSelectOption, TdSegmentOption } from "./types";
```

Add specs:

- `_ai/specs/SPEC__261030-1__td-select.md`
- `_ai/specs/SPEC__261030-2__td-textarea.md`
- `_ai/specs/SPEC__261030-3__td-checkbox.md`

Spec template (repeat per component):

```markdown
---
filename: "_ai/specs/SPEC__261030-1__td-select.md"
title: "TdSelect"
specId: 261030-1
status: Accepted
date: 2026-10-30
priority: high
estimatedComplexity: simple
tags: [design-system, ui, select, accessibility]
documentType: SPEC
---

# TdSelect

## Requirement: Native select wrapper

`TdSelect` SHALL render a native `<select>` with the kit's token styles, accept
`v-model`, forward non-prop attributes to the select, support an options array or
an option slot, and expose disabled, invalid, and required states.

### Scenario: v-model
- **WHEN** the user selects a different option
- **THEN** `update:modelValue` emits the selected string

### Scenario: Accessibility
- **WHEN** `TdSelect` is labelled by `TdFormField`
- **THEN** the select has an accessible name and no axe WCAG A/AA violations
```

Tests per component:

- `TdSelect.test.ts`: options render, v-model, disabled, invalid/aria-invalid,
  attrs forwarding, axe, token-only CSS.
- `TdTextarea.test.ts`: v-model, rows, disabled/readonly, invalid, axe,
  token-only CSS.
- `TdCheckbox.test.ts`: v-model boolean, label/description ids, disabled, axe,
  token-only CSS. The label must not emit when disabled.

**Validation:**

```bash
cd /topdata/topdata-design-system
pnpm --filter @topdata-software-gmbh/ui-kit-web-vue3 test
pnpm --filter @topdata-software-gmbh/ui-kit-web-vue3 typecheck
pnpm --filter @topdata-software-gmbh/ui-kit-web-vue3 build
pnpm --filter @topdata-software-gmbh/playground typecheck
```

**Exit criteria:** Three components exported, three specs, tests and axe checks
pass, playground renders every state, CSS is token-only.

---

## Phase 3 — Add feedback elements

**Goal:** Add `TdBadge`, `TdAlert`, and `TdSpinner`.

[NEW FILE] `packages/ui-kit-web-vue3/src/components/TdBadge.vue`

```vue
<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    variant?: "neutral" | "primary" | "success" | "info" | "warning" | "error";
    size?: "sm" | "md";
  }>(),
  { variant: "neutral", size: "sm" },
);

const classes = computed(() => [
  "td-badge",
  `td-badge--${props.variant}`,
  `td-badge--${props.size}`,
]);
</script>

<template>
  <span :class="classes">
    <span v-if="$slots.icon" class="td-badge__icon" aria-hidden="true"><slot name="icon" /></span>
    <slot />
  </span>
</template>
```

[NEW FILE] `packages/ui-kit-web-vue3/src/styles/components/badge.css`

```css
.td-badge {
  display: inline-flex;
  align-items: center;
  gap: var(--td-space-1);
  border: 1px solid transparent;
  border-radius: var(--td-radius-full);
  font-weight: var(--td-weight-medium);
  line-height: var(--td-leading-tight);
  white-space: nowrap;
}

.td-badge--sm {
  padding: 0.125rem var(--td-space-2);
  font-size: var(--td-text-xs);
}

.td-badge--md {
  padding: var(--td-space-1) var(--td-space-2);
  font-size: var(--td-text-sm);
}

.td-badge--neutral {
  background-color: var(--td-color-surface-raised);
  color: var(--td-color-fg-muted);
}

.td-badge--primary {
  background-color: color-mix(in srgb, var(--td-color-primary) 18%, transparent);
  color: var(--td-color-primary-hover);
}

.td-badge--success {
  background-color: var(--td-color-success-soft);
  color: var(--td-color-success-fg);
}

.td-badge--info {
  background-color: var(--td-color-info-soft);
  color: var(--td-color-info-fg);
}

.td-badge--warning {
  background-color: var(--td-color-warning-soft);
  color: var(--td-color-warning-fg);
}

.td-badge--error {
  background-color: var(--td-color-error-soft);
  color: var(--td-color-error-fg);
}

.td-badge__icon {
  display: inline-flex;
  flex: none;
  align-items: center;
  justify-content: center;
  width: 1em;
  height: 1em;
}

.td-badge__icon svg {
  width: 100%;
  height: 100%;
}
```

[NEW FILE] `packages/ui-kit-web-vue3/src/components/TdAlert.vue`

```vue
<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    variant?: "info" | "success" | "warning" | "error";
    title?: string;
    dismissible?: boolean;
  }>(),
  { variant: "info" },
);

const emit = defineEmits<{ dismiss: [] }>();

const role = computed(() =>
  props.variant === "error" || props.variant === "warning" ? "alert" : "status",
);
const live = computed(() =>
  props.variant === "error" || props.variant === "warning" ? "assertive" : "polite",
);
const classes = computed(() => ["td-alert", `td-alert--${props.variant}`]);
</script>

<template>
  <div :class="classes" :role="role" :aria-live="live">
    <span v-if="$slots.icon" class="td-alert__icon" aria-hidden="true"><slot name="icon" /></span>
    <div class="td-alert__body">
      <p v-if="props.title" class="td-alert__title">{{ props.title }}</p>
      <div class="td-alert__message"><slot /></div>
      <div v-if="$slots.actions" class="td-alert__actions"><slot name="actions" /></div>
    </div>
    <button
      v-if="props.dismissible"
      type="button"
      class="td-alert__dismiss"
      aria-label="Dismiss"
      @click="emit('dismiss')"
    >
      <svg viewBox="0 0 20 20" aria-hidden="true" focusable="false">
        <path
          d="M5 5l10 10M15 5L5 15"
          fill="none"
          stroke="currentColor"
          stroke-width="1.75"
          stroke-linecap="round"
        />
      </svg>
    </button>
  </div>
</template>
```

[NEW FILE] `packages/ui-kit-web-vue3/src/styles/components/alert.css`

```css
.td-alert {
  display: flex;
  align-items: flex-start;
  gap: var(--td-space-3);
  padding: var(--td-space-3) var(--td-space-4);
  border: 1px solid transparent;
  border-radius: var(--td-radius-md);
  color: var(--td-color-fg);
}

.td-alert--info {
  background-color: var(--td-color-info-soft);
  border-color: color-mix(in srgb, var(--td-color-info) 40%, transparent);
}

.td-alert--success {
  background-color: var(--td-color-success-soft);
  border-color: color-mix(in srgb, var(--td-color-success) 40%, transparent);
}

.td-alert--warning {
  background-color: var(--td-color-warning-soft);
  border-color: color-mix(in srgb, var(--td-color-warning) 40%, transparent);
}

.td-alert--error {
  background-color: var(--td-color-error-soft);
  border-color: color-mix(in srgb, var(--td-color-error) 40%, transparent);
}

.td-alert__icon {
  display: inline-flex;
  flex: none;
  width: 1.25rem;
  height: 1.25rem;
  margin-top: 0.1rem;
}

.td-alert__icon svg {
  width: 100%;
  height: 100%;
}

.td-alert__body {
  min-width: 0;
  flex: 1;
}

.td-alert__title {
  margin: 0;
  font-size: var(--td-text-sm);
  font-weight: var(--td-weight-semibold);
}

.td-alert__message {
  margin-top: var(--td-space-1);
  font-size: var(--td-text-xs);
  line-height: var(--td-leading-relaxed);
  color: var(--td-color-fg-muted);
}

.td-alert__actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--td-space-2);
  margin-top: var(--td-space-2);
}

.td-alert__dismiss {
  flex: none;
  padding: var(--td-space-1);
  border: 0;
  border-radius: var(--td-radius-sm);
  background: transparent;
  color: var(--td-color-fg-muted);
  cursor: pointer;
}

.td-alert__dismiss:hover {
  background-color: var(--td-color-surface-hover);
  color: var(--td-color-fg);
}

.td-alert__dismiss svg {
  width: 1rem;
  height: 1rem;
}
```

[NEW FILE] `packages/ui-kit-web-vue3/src/components/TdSpinner.vue`

```vue
<script setup lang="ts">
withDefaults(
  defineProps<{
    size?: "sm" | "md" | "lg";
    label?: string;
  }>(),
  { size: "md" },
);
</script>

<template>
  <span
    class="td-spinner"
    :class="`td-spinner--${size}`"
    :role="label ? 'status' : undefined"
    :aria-label="label || undefined"
    :aria-hidden="label ? undefined : 'true'"
  />
</template>
```

[NEW FILE] `packages/ui-kit-web-vue3/src/styles/components/spinner.css`

```css
.td-spinner {
  display: inline-block;
  flex: none;
  border: 2px solid color-mix(in srgb, currentColor 25%, transparent);
  border-top-color: currentColor;
  border-radius: var(--td-radius-full);
  animation: td-spinner-rotate var(--td-motion-slow) linear infinite;
}

.td-spinner--sm {
  width: 0.875rem;
  height: 0.875rem;
  border-width: 1.5px;
}

.td-spinner--md {
  width: 1.125rem;
  height: 1.125rem;
}

.td-spinner--lg {
  width: 1.5rem;
  height: 1.5rem;
  border-width: 3px;
}

@keyframes td-spinner-rotate {
  to {
    transform: rotate(360deg);
  }
}

@media (prefers-reduced-motion: reduce) {
  .td-spinner {
    animation-duration: 1.5s;
  }
}
```

Add `import "./styles/components/badge.css";`, `alert.css`, and `spinner.css`
to `src/index.ts`, then export `TdBadge`, `TdAlert`, `TdSpinner`. Add playground
examples, README table rows, and specs:

- `_ai/specs/SPEC__261030-4__td-badge.md`
- `_ai/specs/SPEC__261030-5__td-alert.md`
- `_ai/specs/SPEC__261030-6__td-spinner.md`

Tests:

- `TdBadge.test.ts`: variants, sizes, icon slot, axe.
- `TdAlert.test.ts`: role/aria-live by variant, title, actions slot, dismiss
  event, axe.
- `TdSpinner.test.ts`: size classes, labelled status role, decorative
  `aria-hidden`, axe.

**Validation:** same as Phase 2.

**Exit criteria:** Three components exported, specs/tests/axe pass, no raw
colours in CSS.

---

## Phase 4 — Add action and navigation elements

**Goal:** Add `TdMenu`, `TdMenuItem`, and `TdSegmentedControl` for the toolbar
and filter chips.

### Menu context

[NEW FILE] `packages/ui-kit-web-vue3/src/components/menuContext.ts`

```ts
import type { InjectionKey } from "vue";

export interface MenuContext {
  close: () => void;
}

export const menuContextKey: InjectionKey<MenuContext> = Symbol("TdMenu");
```

### TdMenu and TdMenuItem

[NEW FILE] `packages/ui-kit-web-vue3/src/components/TdMenu.vue`

```vue
<script setup lang="ts">
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  provide,
  ref,
  watch,
} from "vue";
import { menuContextKey } from "./menuContext";

defineOptions({ inheritAttrs: false });

const props = withDefaults(
  defineProps<{
    open?: boolean;
    align?: "start" | "end";
    ariaLabel?: string;
  }>(),
  { align: "end" },
);

const emit = defineEmits<{
  "update:open": [open: boolean];
  open: [];
  close: [];
}>();

const root = ref<HTMLElement | null>(null);
const trigger = ref<HTMLElement | null>(null);
const panel = ref<HTMLElement | null>(null);
const internalOpen = ref(false);
const controlled = computed(() => props.open !== undefined);
const isOpen = computed(() => (controlled.value ? Boolean(props.open) : internalOpen.value));

function items(): HTMLElement[] {
  return Array.from(
    panel.value?.querySelectorAll<HTMLElement>('[role="menuitem"]:not([disabled])') ?? [],
  );
}

function focusFirstItem(): void {
  void nextTick(() => items()[0]?.focus());
}

function focusTrigger(): void {
  trigger.value
    ?.querySelector<HTMLElement>("button, [href], input, select, textarea, [tabindex]")
    ?.focus();
}

function setOpen(next: boolean): void {
  if (next === isOpen.value) return;
  if (!controlled.value) internalOpen.value = next;
  emit("update:open", next);
  emit(next ? "open" : "close");
  if (next) focusFirstItem();
  else focusTrigger();
}

function toggle(): void {
  setOpen(!isOpen.value);
}

function close(): void {
  setOpen(false);
}

function onDocumentPointerDown(event: PointerEvent): void {
  if (!isOpen.value) return;
  if (root.value?.contains(event.target as Node)) return;
  close();
}

function onTriggerKeydown(event: KeyboardEvent): void {
  if (event.key === "ArrowDown") {
    event.preventDefault();
    setOpen(true);
  }
}

function onPanelKeydown(event: KeyboardEvent): void {
  if (event.key === "Escape") {
    event.preventDefault();
    close();
    return;
  }
  if (event.key === "Tab") {
    close();
    return;
  }

  const menuItems = items();
  if (menuItems.length === 0) return;

  const current = menuItems.indexOf(document.activeElement as HTMLElement);
  let nextIndex: number | null = null;

  if (event.key === "ArrowDown") nextIndex = current < 0 ? 0 : (current + 1) % menuItems.length;
  else if (event.key === "ArrowUp")
    nextIndex = current < 0 ? menuItems.length - 1 : (current - 1 + menuItems.length) % menuItems.length;
  else if (event.key === "Home") nextIndex = 0;
  else if (event.key === "End") nextIndex = menuItems.length - 1;

  if (nextIndex !== null) {
    event.preventDefault();
    menuItems[nextIndex]?.focus();
  }
}

provide(menuContextKey, { close });

onMounted(() => document.addEventListener("pointerdown", onDocumentPointerDown));
onBeforeUnmount(() => document.removeEventListener("pointerdown", onDocumentPointerDown));

watch(isOpen, (open) => {
  if (open) focusFirstItem();
});
</script>

<template>
  <div ref="root" class="td-menu" v-bind="$attrs">
    <div ref="trigger" class="td-menu__trigger" @click="toggle" @keydown="onTriggerKeydown">
      <slot name="trigger" :open="isOpen" :toggle="toggle" :close="close" />
    </div>
    <Transition name="td-menu">
      <div
        v-if="isOpen"
        ref="panel"
        class="td-menu__panel"
        :class="`td-menu__panel--${props.align}`"
        role="menu"
        :aria-label="props.ariaLabel"
        @keydown="onPanelKeydown"
      >
        <slot :close="close" />
      </div>
    </Transition>
  </div>
</template>
```

[NEW FILE] `packages/ui-kit-web-vue3/src/components/TdMenuItem.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { menuContextKey } from "./menuContext";

withDefaults(
  defineProps<{
    disabled?: boolean;
    danger?: boolean;
  }>(),
  {},
);

const emit = defineEmits<{ select: [] }>();
const menu = inject(menuContextKey, null);

function onClick(): void {
  emit("select");
  menu?.close();
}
</script>

<template>
  <button
    type="button"
    role="menuitem"
    class="td-menu-item"
    :class="{ 'td-menu-item--danger': danger }"
    :disabled="disabled"
    @click="onClick"
  >
    <span v-if="$slots.icon" class="td-menu-item__icon" aria-hidden="true"><slot name="icon" /></span>
    <span class="td-menu-item__body">
      <span class="td-menu-item__label"><slot /></span>
      <span v-if="$slots.description" class="td-menu-item__description">
        <slot name="description" />
      </span>
    </span>
  </button>
</template>
```

[NEW FILE] `packages/ui-kit-web-vue3/src/styles/components/menu.css`

```css
.td-menu {
  position: relative;
  display: inline-flex;
}

.td-menu__trigger {
  display: inline-flex;
}

.td-menu__panel {
  position: absolute;
  z-index: 30;
  top: calc(100% + var(--td-space-1));
  min-width: 16rem;
  max-width: min(22rem, calc(100vw - (2 * var(--td-space-4))));
  padding: var(--td-space-1);
  border: 1px solid var(--td-color-border);
  border-radius: var(--td-radius-lg);
  background-color: var(--td-color-surface-raised);
  box-shadow: var(--td-shadow-lg);
}

.td-menu__panel--start {
  left: 0;
}

.td-menu__panel--end {
  right: 0;
}

.td-menu-item {
  display: flex;
  width: 100%;
  gap: var(--td-space-2);
  align-items: flex-start;
  padding: var(--td-space-2) var(--td-space-3);
  border: 0;
  border-radius: var(--td-radius-md);
  background: transparent;
  color: var(--td-color-fg);
  text-align: left;
  cursor: pointer;
}

.td-menu-item:hover:not(:disabled),
.td-menu-item:focus-visible {
  background-color: var(--td-color-surface-hover);
  outline: none;
}

.td-menu-item:disabled {
  color: var(--td-color-disabled-fg);
  cursor: not-allowed;
}

.td-menu-item--danger {
  color: var(--td-color-error-fg);
}

.td-menu-item__icon {
  display: inline-flex;
  flex: none;
  width: 1rem;
  height: 1rem;
  margin-top: 0.1rem;
}

.td-menu-item__icon svg {
  width: 100%;
  height: 100%;
}

.td-menu-item__body {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: var(--td-space-1);
}

.td-menu-item__label {
  font-size: var(--td-text-sm);
}

.td-menu-item__description {
  color: var(--td-color-fg-muted);
  font-size: var(--td-text-xs);
  line-height: var(--td-leading-relaxed);
}
```

### TdSegmentedControl

[NEW FILE] `packages/ui-kit-web-vue3/src/components/TdSegmentedControl.vue`

```vue
<script setup lang="ts">
import type { TdSegmentOption } from "../types";

const props = defineProps<{
  modelValue: string[];
  options: TdSegmentOption[];
  ariaLabel?: string;
}>();

const emit = defineEmits<{ "update:modelValue": [value: string[]] }>();

function toggle(value: string): void {
  const next = props.modelValue.includes(value)
    ? props.modelValue.filter((item) => item !== value)
    : [...props.modelValue, value];
  emit("update:modelValue", next);
}
</script>

<template>
  <div class="td-segmented" role="group" :aria-label="ariaLabel">
    <button
      v-for="option in options"
      :key="option.value"
      type="button"
      class="td-segmented__item"
      :class="{ 'td-segmented__item--active': modelValue.includes(option.value) }"
      :aria-pressed="modelValue.includes(option.value)"
      :disabled="option.disabled"
      @click="toggle(option.value)"
    >
      <span>{{ option.label }}</span>
      <span v-if="option.count !== undefined" class="td-segmented__count">
        {{ option.count }}
      </span>
    </button>
  </div>
</template>
```

[NEW FILE] `packages/ui-kit-web-vue3/src/styles/components/segmented-control.css`

```css
.td-segmented {
  display: flex;
  flex-wrap: wrap;
  gap: var(--td-space-1);
}

.td-segmented__item {
  display: inline-flex;
  align-items: center;
  gap: var(--td-space-1);
  padding: var(--td-space-1) var(--td-space-2);
  border: 1px solid var(--td-color-border);
  border-radius: var(--td-radius-full);
  background-color: var(--td-color-surface);
  color: var(--td-color-fg-muted);
  font-size: var(--td-text-xs);
  cursor: pointer;
  transition:
    background-color var(--td-motion-fast) var(--td-ease-standard),
    border-color var(--td-motion-fast) var(--td-ease-standard),
    color var(--td-motion-fast) var(--td-ease-standard);
}

.td-segmented__item:hover:not(:disabled) {
  background-color: var(--td-color-surface-hover);
  color: var(--td-color-fg);
}

.td-segmented__item--active {
  border-color: var(--td-color-primary);
  background-color: color-mix(in srgb, var(--td-color-primary) 18%, transparent);
  color: var(--td-color-primary-hover);
}

.td-segmented__item:disabled {
  cursor: not-allowed;
  color: var(--td-color-disabled-fg);
}

.td-segmented__count {
  color: var(--td-color-fg-dim);
  font-family: var(--td-font-mono);
  font-size: var(--td-text-xs);
}
```

Add `import "./styles/components/menu.css";` and
`import "./styles/components/segmented-control.css";` to `src/index.ts`, export
`TdMenu`, `TdMenuItem`, and `TdSegmentedControl`. Add playground examples, README
table rows, and specs:

- `_ai/specs/SPEC__261030-7__td-menu.md` (covers `TdMenuItem`)
- `_ai/specs/SPEC__261030-8__td-segmented-control.md`

Tests:

- `TdMenu.test.ts`: opens on trigger click, closes on outside pointerdown,
  Escape closes and restores focus, ArrowDown/ArrowUp/Home/End move focus,
  disabled items are skipped, selecting an item emits and closes, axe.
- `TdSegmentedControl.test.ts`: toggles values, emits new arrays, aria-pressed,
  counts, disabled option, axe.

**Validation:** same as Phase 2.

**Exit criteria:** Three exported components, two specs, keyboard and axe tests
pass, playground renders the toolbar menu and status filters.

---

## Phase 5 — Repo-manager token foundation

**Goal:** The app consumes tokens for all layout values, helper modules emit
semantic variants instead of palette classes, and raw hex is gone from styling.

**Files:**

- [MODIFY] `/topdata/repo-manager/tailwind.config.js`
- [MODIFY] `/topdata/repo-manager/index.html`
- [MODIFY] `/topdata/repo-manager/src/style.css`
- [MODIFY] `/topdata/repo-manager/src/lib/groupDefaults.ts`
- [MODIFY] `/topdata/repo-manager/src/lib/repoStatus.ts`
- [NEW FILE] `/topdata/repo-manager/src/lib/__tests__/repoStatus.test.ts`

[MODIFY] `tailwind.config.js`

```js
import preset from '@topdata-software-gmbh/design-tokens/tailwind-preset';

/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],
  presets: [preset],
  theme: { extend: {} },
  plugins: [],
};
```

[MODIFY] `index.html`

```html
<!doctype html>
<html lang="en" data-theme="dark">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <meta name="color-scheme" content="dark" />
    <title>Git Repo Manager</title>
  </head>
  <body class="bg-canvas text-fg">
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

[MODIFY] `src/style.css`

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

:root {
  color-scheme: dark;
}

html,
body,
#app {
  height: 100%;
}

body {
  margin: 0;
  overflow: hidden;
  font-family: var(--td-font-body);
  -webkit-font-smoothing: antialiased;
  background-color: var(--td-color-canvas);
  color: var(--td-color-fg);
}

::-webkit-scrollbar {
  width: 10px;
  height: 10px;
}

::-webkit-scrollbar-track {
  background: var(--td-color-canvas);
}

::-webkit-scrollbar-thumb {
  background: var(--td-color-border-strong);
  border-radius: var(--td-radius-full);
  border: 2px solid var(--td-color-canvas);
}

::-webkit-scrollbar-thumb:hover {
  background: var(--td-color-fg-dim);
}

select option {
  background-color: var(--td-color-surface);
  color: var(--td-color-fg);
}
```

[MODIFY] `src/lib/groupDefaults.ts`

```ts
import type { RepoGroup } from '../types/domain';

export const GROUP_COLOR_TOKENS = [
  'primary',
  'accent',
  'success',
  'warning',
  'error',
  'info',
] as const;

export type GroupColorToken = (typeof GROUP_COLOR_TOKENS)[number];

export const GROUP_COLORS = GROUP_COLOR_TOKENS;

export function groupColorStyle(color: string): Record<string, string> {
  if (color.startsWith('#')) {
    return { backgroundColor: color };
  }
  return { backgroundColor: `var(--td-color-${color})` };
}

export function createGroup(
  name: string,
  repoPaths: string[],
  colorIndex: number,
): RepoGroup {
  return {
    id: `grp_${typeof crypto.randomUUID === 'function' ? crypto.randomUUID() : Date.now()}`,
    name: name.trim(),
    color: GROUP_COLORS[colorIndex % GROUP_COLORS.length],
    repoPaths: [...repoPaths],
    settings: {
      syncBranching: true,
      autoStashOnSwitch: true,
      defaultBaseBranch: 'main',
    },
  };
}
```

Existing config files may contain a hex group colour. `groupColorStyle` keeps
them rendering; new groups store a token name. Do not rewrite the saved config
format in this iteration.

[MODIFY] `src/lib/repoStatus.ts`

```ts
import type { Repository } from '../types/domain';

export type StatusBadgeVariant = 'neutral' | 'success' | 'info' | 'warning' | 'error';

export interface StatusBadge {
  label: string;
  variant: StatusBadgeVariant;
}

export function statusBadges(repo: Repository): StatusBadge[] {
  if (!repo.isAccessible) {
    return [{ label: 'Folder not found', variant: 'error' }];
  }

  const badges: StatusBadge[] = [];

  if (repo.state.hasConflicts) {
    badges.push({ label: 'Needs attention', variant: 'error' });
  }

  if (repo.state.operationState !== 'idle') {
    badges.push({ label: operationLabel(repo.state.operationState), variant: 'warning' });
  }

  if (!repo.state.isClean) {
    badges.push({ label: 'Changes not saved', variant: 'warning' });
  }

  if (repo.state.behindCount > 0) {
    badges.push({
      label: `${repo.state.behindCount} ${repo.state.behindCount === 1 ? 'update' : 'updates'} available`,
      variant: 'info',
    });
  }

  if (repo.state.aheadCount > 0) {
    badges.push({ label: `${repo.state.aheadCount} to send`, variant: 'success' });
  }

  if (repo.state.remotes.length === 0) {
    badges.push({ label: 'On this computer only', variant: 'neutral' });
  } else if (repo.state.isClean && repo.state.aheadCount === 0 && repo.state.behindCount === 0) {
    badges.push({ label: 'Up to date', variant: 'success' });
  }

  return badges;
}

export function operationLabel(operation: string): string {
  switch (operation) {
    case 'merging':
      return 'Merge in progress';
    case 'rebasing':
      return 'Rebase in progress';
    case 'cherry-picking':
      return 'Cherry-pick in progress';
    default:
      return 'Git operation in progress';
  }
}

export function branchLabel(repo: Repository): string {
  return repo.state.isDetachedHead ? 'Not on a branch' : repo.state.currentBranch || 'Unknown';
}
```

[NEW FILE] `src/lib/__tests__/repoStatus.test.ts`

```ts
import { describe, expect, it } from 'vitest';
import { statusBadges } from '../repoStatus';
import type { GitRepoState, Repository } from '../../types/domain';

function repo(overrides: Partial<GitRepoState> = {}): Repository {
  return {
    id: 'repo',
    name: 'repo',
    path: '/repo',
    kind: 'other',
    isAccessible: true,
    groupIds: [],
    state: {
      currentBranch: 'main',
      isDetachedHead: false,
      headCommitHash: 'abc',
      headCommitMessage: 'msg',
      isClean: true,
      aheadCount: 0,
      behindCount: 0,
      modifiedFiles: 0,
      untrackedFiles: 0,
      stagedFiles: 0,
      hasConflicts: false,
      operationState: 'idle',
      remotes: [],
      ...overrides,
    },
  };
}

describe('statusBadges', () => {
  it('uses error for inaccessible repos', () => {
    const value = { ...repo(), isAccessible: false } as Repository;
    expect(statusBadges(value)[0]?.variant).toBe('error');
  });

  it('uses semantic variants for cleanliness and sync state', () => {
    expect(statusBadges(repo())[0]).toMatchObject({ label: 'On this computer only', variant: 'neutral' });
    expect(
      statusBadges(repo({ remotes: [{ name: 'origin', url: 'git@example.com' }] }))[0],
    ).toMatchObject({
      label: 'Up to date',
      variant: 'success',
    });
    expect(statusBadges(repo({ isClean: false }))[0]).toMatchObject({
      label: 'Changes not saved',
      variant: 'warning',
    });
    expect(statusBadges(repo({ behindCount: 2 }))[0]).toMatchObject({
      label: '2 updates available',
      variant: 'info',
    });
  });
});
```

**Validation:**

```bash
cd /topdata/repo-manager
pnpm typecheck
pnpm test
```

**Exit criteria:** `tailwind.config.js` uses the preset; `style.css` and
`index.html` contain no raw palette; `repoStatus` tests pass. Component files are
migrated in the next phases; the token-only source test is added in Phase 9
after they are clean.

---

## Phase 6 — Migrate modals and forms

**Goal:** Every modal and form uses the kit components, with no custom overlay
markup, native inputs, or palette classes.

### GroupEditorModal

Replace the fixed overlay + custom form shell with `TdModal`:

[MODIFY] `src/components/GroupEditorModal.vue`

```vue
<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { Trash2 } from 'lucide-vue-next';
import {
  TdAlert,
  TdButton,
  TdCheckbox,
  TdFormField,
  TdInput,
  TdModal,
} from '@topdata-software-gmbh/ui-kit-web-vue3';
import { useRepoStore } from '../stores/repoStore';
import { GROUP_COLORS, groupColorStyle } from '../lib/groupDefaults';
import type { RepoGroup } from '../types/domain';

const store = useRepoStore();
const name = ref('');
const color = ref<string>(GROUP_COLORS[0]);
const repoPaths = ref<string[]>([]);
const syncBranching = ref(true);
const autoStashOnSwitch = ref(true);
const defaultBaseBranch = ref('main');
const submitting = ref(false);
const deleting = ref(false);
const error = ref('');

const isEditing = computed(() => store.editingGroup !== null);
const allSelected = computed(
  () =>
    store.repositories.length > 0 &&
    store.repositories.every((repo) => repoPaths.value.includes(repo.path)),
);

watch(
  () => store.groupEditorOpen,
  (open) => {
    if (!open) return;
    const group = store.editingGroup;
    name.value = group?.name ?? '';
    color.value = group?.color ?? GROUP_COLORS[0];
    repoPaths.value = group ? [...group.repoPaths] : [];
    syncBranching.value = group?.settings.syncBranching ?? true;
    autoStashOnSwitch.value = group?.settings.autoStashOnSwitch ?? true;
    defaultBaseBranch.value = group?.settings.defaultBaseBranch ?? 'main';
    submitting.value = false;
    deleting.value = false;
    error.value = '';
  },
);

function toggleRepository(path: string): void {
  repoPaths.value = repoPaths.value.includes(path)
    ? repoPaths.value.filter((existing) => existing !== path)
    : [...repoPaths.value, path];
}

function toggleAll(): void {
  repoPaths.value = allSelected.value ? [] : store.repositories.map((repo) => repo.path);
}

function newId(): string {
  return `grp_${typeof crypto.randomUUID === 'function' ? crypto.randomUUID() : Date.now()}`;
}

async function submit(): Promise<void> {
  const trimmed = name.value.trim();
  if (!trimmed) {
    error.value = 'Give the group a name.';
    return;
  }
  if (submitting.value) return;

  error.value = '';
  submitting.value = true;
  const group: RepoGroup = {
    id: store.editingGroup?.id ?? newId(),
    name: trimmed,
    color: color.value,
    repoPaths: [...repoPaths.value],
    settings: {
      syncBranching: syncBranching.value,
      autoStashOnSwitch: autoStashOnSwitch.value,
      defaultBaseBranch: defaultBaseBranch.value.trim() || 'main',
    },
  };

  try {
    await store.saveGroup(group);
    store.closeGroupEditor();
  } finally {
    submitting.value = false;
  }
}

async function remove(): Promise<void> {
  const group = store.editingGroup;
  if (!group || deleting.value) return;
  if (!window.confirm(`Delete group "${group.name}"? Your projects and files are not touched.`)) {
    return;
  }

  deleting.value = true;
  try {
    await store.deleteGroup(group.id);
    store.closeGroupEditor();
  } finally {
    deleting.value = false;
  }
}
</script>

<template>
  <TdModal
    :open="store.groupEditorOpen"
    :title="isEditing ? 'Edit project group' : 'New project group'"
    description="Name the group and choose the projects it contains."
    size="lg"
    @close="store.closeGroupEditor"
  >
    <form id="group-editor-form" class="space-y-4" @submit.prevent="submit">
      <div class="grid grid-cols-[1fr_auto] gap-4">
        <TdFormField label="Group name" required>
          <template #default="{ id, required }">
            <TdInput :id="id" v-model="name" :required="required" placeholder="Theme projects" autofocus />
          </template>
        </TdFormField>

        <div>
          <span class="mb-1.5 block text-xs font-medium text-fg">Color</span>
          <div class="flex items-center gap-1.5 pt-1">
            <TdButton
              v-for="swatch in GROUP_COLORS"
              :key="swatch"
              variant="ghost"
              class="h-5 w-5 rounded-full border-2 p-0 transition-transform hover:scale-110"
              :class="color === swatch ? 'border-fg' : 'border-transparent'"
              :style="groupColorStyle(swatch)"
              :title="swatch"
              :aria-label="`Use ${swatch} for this group`"
              :aria-pressed="color === swatch"
              @click="color = swatch"
            />
          </div>
        </div>
      </div>

      <div>
        <div class="mb-1.5 flex items-center justify-between">
          <span class="text-xs font-medium text-fg">Projects</span>
          <TdButton
            v-if="store.repositories.length > 0"
            variant="ghost"
            size="sm"
            @click="toggleAll"
          >
            {{ allSelected ? 'Clear all' : 'Select all' }}
          </TdButton>
        </div>

        <p
          v-if="store.repositories.length === 0"
          class="rounded-lg border border-dashed border-border px-4 py-6 text-center text-xs text-fg-muted"
        >
          No projects yet. Add the folder where your projects live, then look for projects.
        </p>
        <div
          v-else
          class="max-h-56 divide-y divide-border overflow-auto rounded-lg border border-border bg-surface"
        >
          <TdCheckbox
            v-for="repo in store.repositories"
            :key="repo.id"
            :model-value="repoPaths.includes(repo.path)"
            class="w-full px-3 py-2 hover:bg-surface-hover"
            @update:model-value="toggleRepository(repo.path)"
          >
            <span class="flex min-w-0 flex-1 items-center gap-3">
              <span class="min-w-0 flex-1">
                <span class="block truncate text-xs text-fg">{{ repo.name }}</span>
                <span class="block truncate font-mono text-xs text-fg-muted">{{ repo.path }}</span>
              </span>
              <span class="shrink-0 font-mono text-xs text-primary">{{ repo.state.currentBranch }}</span>
            </span>
          </TdCheckbox>
        </div>
        <p class="mt-1.5 text-xs text-fg-muted">
          {{ repoPaths.length }} of {{ store.repositories.length }} projects selected
        </p>
      </div>

      <details class="rounded-lg border border-border bg-surface px-4 py-3">
        <summary class="cursor-pointer text-xs font-medium text-fg">Advanced settings</summary>
        <div class="mt-3 grid grid-cols-2 gap-3">
          <TdCheckbox
            v-model="autoStashOnSwitch"
            label="Keep unsaved changes when switching branches"
            description="Changes are set aside and put back automatically."
          />
          <TdCheckbox
            v-model="syncBranching"
            label="Show the branch chooser for the group"
            description="Lets you switch all projects in this group together."
          />
        </div>
        <div class="mt-3 w-52">
          <TdFormField label="Starting branch for new work">
            <template #default="{ id }">
              <TdInput :id="id" v-model="defaultBaseBranch" placeholder="main" />
            </template>
          </TdFormField>
        </div>
      </details>

      <TdAlert v-if="error" variant="error">{{ error }}</TdAlert>
    </form>

    <template #footer>
      <div class="flex w-full items-center justify-between">
        <TdButton
          v-if="isEditing"
          variant="danger"
          :loading="deleting"
          @click="remove"
        >
          <template #icon-left><Trash2 /></template>
          Delete group
        </TdButton>
        <span v-else />
        <div class="flex items-center gap-2">
          <TdButton variant="ghost" @click="store.closeGroupEditor">Cancel</TdButton>
          <TdButton
            type="submit"
            form="group-editor-form"
            variant="primary"
            :loading="submitting"
          >
            {{ isEditing ? 'Save changes' : 'Create group' }}
          </TdButton>
        </div>
      </div>
    </template>
  </TdModal>
</template>
```

Keep `remove()` but replace `window.confirm` with
`TdModal` + a nested confirm state later; for this iteration a native
`window.confirm` is allowed only for destructive confirmation and must be
recorded as follow-up. Prefer adding a small local confirm panel inside the
modal if time permits.

### SaveChangesModal

- Replace overlay with `TdModal`.
- Replace `<textarea>` with `TdFormField` + `TdTextarea`.
- Replace loading text with `TdSpinner`.
- Replace change-count pill with `TdBadge`.
- Replace the error `<pre>` with `TdAlert variant="error"` preserving
  `whitespace-pre-wrap`.
- Replace footer buttons with `TdButton`.
- Replace `statusClass` with a variant map and `TdBadge`.

[MODIFY] `src/components/SaveChangesModal.vue`

Merge the imports and `statusVariant` below into the **existing** script setup.
Do not replace the script: `store`, `repos`, `changes`, `message`, `loading`,
`saving`, `error`, `totalFiles`, `canSave`, `watch`, `loadChanges`,
`statusLabel`, and `save` stay unchanged.

```vue
<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { FileText, Save } from 'lucide-vue-next';
import {
  TdAlert,
  TdBadge,
  TdButton,
  TdFormField,
  TdModal,
  TdSpinner,
  TdTextarea,
} from '@topdata-software-gmbh/ui-kit-web-vue3';
import { backend } from '../lib/backend';
import { useRepoStore } from '../stores/repoStore';
import type { FileChange, FileChangeStatus, Repository } from '../types/domain';

// ... existing `store`, `repos`, `changes`, `message`, `loading`, `saving`,
// `error`, `totalFiles`, `canSave`, `watch`, `loadChanges`, `statusLabel`, and
// `save` stay here unchanged ...

type ChangeVariant = 'success' | 'error' | 'info' | 'warning' | 'neutral';

function statusVariant(status: FileChangeStatus): ChangeVariant {
  switch (status) {
    case 'new':
      return 'success';
    case 'deleted':
    case 'conflict':
      return 'error';
    case 'renamed':
      return 'info';
    default:
      return 'warning';
  }
}
</script>

<template>
  <TdModal
    :open="store.saveModalOpen"
    title="Save to Git"
    description="Create a checkpoint of your work. Nothing is deleted or moved."
    size="lg"
    :show-close="!saving"
    :close-on-backdrop="!saving"
    :close-on-escape="!saving"
    @close="!saving && store.closeSaveChanges()"
  >
    <form id="save-changes-form" class="space-y-4" @submit.prevent="save">
      <div
        v-if="loading"
        class="flex items-center gap-2 rounded-lg border border-border bg-surface px-4 py-3 text-xs text-fg-muted"
      >
        <TdSpinner size="sm" label="Reading your changes" />
        Reading your changes…
      </div>

      <div v-for="repo in repos" :key="repo.id" class="rounded-lg border border-border bg-surface">
        <div class="flex items-center justify-between gap-3 px-4 py-3">
          <div class="min-w-0">
            <p class="truncate text-sm font-medium text-fg">{{ repo.name }}</p>
            <p class="truncate text-xs text-fg-muted">
              Working on <span class="font-mono text-fg-muted">{{ repo.state.currentBranch }}</span>
            </p>
          </div>
          <TdBadge variant="neutral">{{ changes[repo.path]?.length ?? 0 }} changed</TdBadge>
        </div>

        <div v-if="(changes[repo.path]?.length ?? 0) > 0" class="border-t border-border">
          <div class="max-h-40 overflow-auto px-4 py-2">
            <div
              v-for="change in changes[repo.path]"
              :key="`${change.status}:${change.path}`"
              class="flex items-center gap-2 py-0.5 font-mono text-xs"
            >
              <FileText class="h-3 w-3 shrink-0 text-fg-dim" />
              <span class="min-w-0 flex-1 truncate text-fg-muted" :title="change.path">
                {{ change.path }}
              </span>
              <TdBadge :variant="statusVariant(change.status)" size="sm">
                {{ statusLabel(change.status) }}
              </TdBadge>
            </div>
          </div>
        </div>
      </div>

      <TdFormField
        label="What did you change?"
        hint="Write a short note for your future self. It will appear in the project history."
        required
      >
        <template #default="{ id, required }">
          <TdTextarea
            :id="id"
            v-model="message"
            :required="required"
            :rows="3"
            autofocus
            class="resize-none"
            placeholder="For example: Updated the header layout and product card colors"
          />
        </template>
      </TdFormField>

      <TdAlert v-if="error" variant="error">
        <pre class="whitespace-pre-wrap font-mono text-xs">{{ error }}</pre>
      </TdAlert>
    </form>

    <template #footer>
      <div class="flex w-full items-center justify-between">
        <p class="text-xs text-fg-muted">
          {{ totalFiles }} {{ totalFiles === 1 ? 'file' : 'files' }} across
          {{ repos.length }} {{ repos.length === 1 ? 'project' : 'projects' }}
        </p>
        <div class="flex items-center gap-2">
          <TdButton variant="ghost" :disabled="saving" @click="store.closeSaveChanges">
            Cancel
          </TdButton>
          <TdButton
            type="submit"
            form="save-changes-form"
            variant="primary"
            :loading="saving"
            :disabled="!canSave"
          >
            <template #icon-left><Save /></template>
            Save to Git
          </TdButton>
        </div>
      </div>
    </template>
  </TdModal>
</template>
```

Keep `statusLabel` and the existing `loadChanges`, `save`, `canSave`, `totalFiles`
logic unchanged; only the presentation and `statusVariant` mapping change.

### HelpModal

- Replace overlay with `TdModal`.
- Replace the amber warning block with `TdAlert variant="warning"`.
- Replace footer button with `TdButton variant="primary"`.
- Replace numbered badge spans with token classes or `TdBadge variant="primary"`.

### AddScanPathModal and ToastStack

- Only update the package import.
- Keep the existing `TdModal`/`TdButton`/`TdFormField`/`TdInput` and
  `TdToastStack` usage.
- Replace any remaining `text-rose-*`, `text-slate-*`, etc. with tokens.

**Validation:**

```bash
cd /topdata/repo-manager
pnpm typecheck
pnpm test
```

**Exit criteria:** No custom overlay markup remains in modal components; no
native `<input>`, `<select>`, `<textarea>`, or `<button>` remains in these files;
no palette classes remain in them.

---

## Phase 7 — Migrate shell, toolbar, tables, and filters

**Goal:** The app shell and data views use kit elements and token utilities.

### App.vue

[MODIFY] `src/App.vue`

```vue
<template>
  <div class="flex h-screen w-screen overflow-hidden bg-canvas font-body text-fg">
    <OnboardingWizard v-if="store.onboardingOpen" />
    <template v-else>
      <Sidebar />
      <div class="flex h-full min-w-0 flex-1 flex-col overflow-hidden">
        <AllReposPage v-if="store.selectedGroupId === 'all'" />
        <template v-else>
          <Toolbar />
          <GuidanceBanner />
          <RepoTable />
        </template>
      </div>
      <GroupEditorModal />
      <AddScanPathModal />
      <SaveChangesModal />
      <HelpModal />
    </template>
    <ToastStack />
  </div>
</template>
```

### Sidebar

- Replace logo/title area with token classes (`bg-primary/10`, `text-primary`).
- Replace every icon button with `TdButton` variant `ghost`, size `sm`,
  `aria-label`, and icon slot.
- Replace counts with `TdBadge variant="neutral"`.
- Replace scanner `Loader2` with `TdSpinner size="sm"`.
- Replace `group.color` inline style with `groupColorStyle(group.color)`.
- Replace active nav classes with token utilities:
  `bg-surface-raised font-semibold text-primary` and
  `text-fg-muted hover:bg-surface hover:text-fg`.
- Replace scan path text classes with `text-fg-muted` / `text-fg-dim`.
- Replace **every** remaining native `<button>` with `TdButton`: the group row
  selector, the two header add buttons, the scan-path row toggle/remove
  buttons, and the three bottom actions (`Look for new projects`, `Setup`,
  `Add project folder`). The phase exit criterion is no native `<button>` in
  this file.

Example nav item:

```vue
<TdButton
  variant="ghost"
  block
  class="justify-between px-2.5 py-1.5 text-xs"
  :class="
    store.selectedGroupId === 'all'
      ? 'bg-surface-raised font-semibold text-primary'
      : 'text-fg-muted hover:bg-surface hover:text-fg'
  "
  @click="store.selectGroup('all')"
>
  <template #icon-left><Layers /></template>
  <span class="flex w-full items-center justify-between">
    <span>All repos</span>
    <TdBadge variant="neutral">{{ store.repositories.length }}</TdBadge>
  </span>
</TdButton>
```

Note: `TdButton` with a default slot renders a label; put the full row in the
default slot and only the leading icon in `#icon-left`.

### Toolbar

- Replace branch `<select>` with `TdSelect`:
  `v-model="branchSelection" :options="branchOptions" :disabled="!canSwitchBranch"`
  and `aria-label="Working on branch"`.
  Add the options computed and keep the existing `@change="onBranchChange"`:

```ts
const branchOptions = computed(() => [
  {
    value: '',
    label:
      store.filteredRepositories.length === 0 ? 'No projects yet' : 'Choose a branch…',
    disabled: true,
  },
  ...store.groupBranches.map((branch) => ({
    value: branch.name,
    label: `${branch.name} (${branch.repoCount}/${store.filteredRepositories.length})`,
  })),
]);
```
- Replace action buttons with `TdButton` variants:
  primary for Save, secondary for Get latest/Send changes, ghost for rescan/help.
- Replace spinners with `TdSpinner size="sm"`.
- Replace project count pill with `TdBadge variant="neutral"`.
- Replace the "up to date" green dot with `bg-success`.
- Replace the custom `moreOpen` menu with `TdMenu`/`TdMenuItem`:

```vue
<TdMenu aria-label="More Git actions">
  <template #trigger="{ open }">
    <TdButton
      variant="ghost"
      size="sm"
      aria-label="More Git actions"
      aria-haspopup="menu"
      :aria-expanded="open"
    >
      <template #icon-left><MoreHorizontal /></template>
    </TdButton>
  </template>
  <TdMenuItem :disabled="busy || store.filteredRepositories.length === 0" @select="runAction('fetch')">
    <template #icon><RefreshCw /></template>
    Check for updates
    <template #description>Looks for new work without changing your files.</template>
  </TdMenuItem>
  <TdMenuItem :disabled="busy || store.filteredRepositories.length === 0" @select="runAction('stash')">
    <template #icon><Archive /></template>
    Set aside changes
    <template #description>Temporarily store unfinished work in all projects.</template>
  </TdMenuItem>
  <TdMenuItem :disabled="busy || store.filteredRepositories.length === 0" @select="runAction('stash_pop')">
    <template #icon><ArchiveRestore /></template>
    Bring back set-aside changes
    <template #description>Restore the last set of changes in all projects.</template>
  </TdMenuItem>
</TdMenu>
```

Remove `moreOpen`, `moreRef`, `handleOutsideClick`, and the
`onMounted`/`onUnmounted` listeners from the script. Also remove the
`moreOpen.value = false;` line inside `runAction`, and the now-unused
`onMounted`/`onUnmounted` and `Loader2` imports. Do the same `Loader2` import
cleanup in `Sidebar.vue` and `RepoListTable.vue` after replacing it with
`TdSpinner`; `noUnusedLocals` is enabled, so a leftover symbol fails
`pnpm typecheck`.

### RepoListTable

- Replace status badge `<span>` with `TdBadge :variant="badge.variant"`.
- Replace icon buttons with `TdButton variant="ghost" size="sm"` and
  `aria-label`.
- Replace the labelled `Save`/`Review` button with `TdButton variant="ghost"
  size="sm"`; it is a native `<button>` in the actions cell even though it is
  not icon-only.
- Replace `Loader2` with `TdSpinner size="sm"`.
- Replace table header/row classes with tokens:
  `border-border bg-surface text-fg-muted`, `hover:bg-surface-hover`,
  `text-fg`, `text-fg-muted`, `text-fg-dim`, `text-primary`, `text-warning`,
  `text-success`, `text-error`.
- Replace the branch chip with `TdBadge variant="primary"` or `warning` when
  detached.

Example:

```vue
<TdBadge v-for="badge in statusBadges(repo)" :key="badge.label" :variant="badge.variant">
  <template #icon>
    <AlertCircle v-if="badge.label === 'Needs attention'" />
    <GitMerge v-else-if="badge.label.includes('in progress')" />
  </template>
  {{ badge.label }}
</TdBadge>
```

### RepoTable

- Replace the search `<input>` with `TdInput` + a `#prefix` Search icon and
  `aria-label="Search projects"`.
- Replace every empty-state `<button>` (`Add project folder`, `How this works`,
  `Edit group`, `Scan again`, `Add project folder`, `Clear search`) with
  `TdButton` (`primary` for the add actions, `secondary`/`ghost` for the rest).
- Replace raw palette classes with tokens.
- Keep `RepoListTable` as the table renderer.

### AllReposPage

- Replace the search `<input>` with `TdInput` and a `#prefix` Search icon.
- Replace the two `<select>` controls with `TdFormField` + `TdSelect`.
- Replace status filter `<button>` loop with `TdSegmentedControl`.
- Replace the header `Add project folder` and `Clear filters` buttons with
  `TdButton` (`secondary` for add, `ghost` for clear).
- Replace empty-state buttons with `TdButton`.
- Replace all palette classes with tokens.

```vue
<TdInput
  v-model="store.allReposFilters.search"
  type="search"
  placeholder="Search name, path, or branch…"
  aria-label="Search all repositories"
>
  <template #prefix><Search class="h-3.5 w-3.5 text-fg-muted" /></template>
</TdInput>

<TdFormField label="Group">
  <template #default="{ id }">
    <TdSelect :id="id" v-model="store.allReposFilters.groupId" :options="groupOptions" />
  </template>
</TdFormField>

<TdFormField label="Type">
  <template #default="{ id }">
    <TdSelect :id="id" v-model="kindFilter" :options="kindOptions" />
  </template>
</TdFormField>

<TdSegmentedControl
  v-model="statusFilter"
  :options="statusOptionsWithCounts"
  aria-label="Filter by status"
/>
```

Add the typed computed bridges and kind options:

```ts
const kindOptions = [
  { value: 'all', label: 'Any type' },
  { value: 'theme', label: 'Theme' },
  { value: 'plugin', label: 'Plugin' },
  { value: 'other', label: 'Other' },
];

const kindFilter = computed({
  get: () => store.allReposFilters.kind,
  set: (value: string) => {
    store.allReposFilters.kind = value as RepoKind | 'all';
  },
});

const statusFilter = computed({
  get: () => store.allReposFilters.statuses,
  set: (value: string[]) => {
    store.allReposFilters.statuses = value as RepoStatusFilter[];
  },
});

const statusOptionsWithCounts = computed(() =>
  statusOptions.map((option) => ({
    ...option,
    count: store.allReposStatusCounts[option.value],
  })),
);
```

Import `RepoKind` from `../types/domain` alongside `RepoStatusFilter` from
`../lib/repoFilters`.

### GuidanceBanner

- Replace the custom tone classes with `TdAlert`.
- Map `store.insight.tone`:
  `danger -> error`, `warning -> warning`, `info -> info`, `success -> success`.
- Pass the insight icon via `#icon`, message via default slot, and the
  "How this works"/action buttons via `#actions` using `TdButton`.

**Validation:**

```bash
cd /topdata/repo-manager
pnpm typecheck
pnpm test
```

**Exit criteria:** No native `<button>`, `<input>`, `<select>`, or `<textarea>`
remains in `App.vue`, `Sidebar.vue`, `Toolbar.vue`, `AllReposPage.vue`,
`RepoTable.vue`, `RepoListTable.vue`, or `GuidanceBanner.vue`; no palette
classes remain in those files.

---

## Phase 8 — Migrate onboarding

**Goal:** The first-run wizard uses the kit modal, form fields, alerts, badges,
and buttons.

### OnboardingWizard

[MODIFY] `src/components/OnboardingWizard.vue`

```vue
<script setup lang="ts">
import { computed, ref } from 'vue';
import { X } from 'lucide-vue-next';
import { TdButton, TdModal } from '@topdata-software-gmbh/ui-kit-web-vue3';
import { useRepoStore } from '../stores/repoStore';
import OnboardingStepChoose from './onboarding/OnboardingStepChoose.vue';
import OnboardingStepFind from './onboarding/OnboardingStepFind.vue';
import OnboardingStepGroups from './onboarding/OnboardingStepGroups.vue';
import OnboardingStepReady from './onboarding/OnboardingStepReady.vue';

const store = useRepoStore();
const heading = ref<HTMLElement | null>(null);
const steps = ['Find projects', 'Choose repos', 'Confirm groups', 'Ready'] as const;
const currentLabel = computed(() => steps[store.onboardingStep] ?? steps[0]);
</script>

<template>
  <TdModal
    :open="store.onboardingOpen"
    size="full"
    flush
    :show-close="false"
    :close-on-backdrop="false"
    aria-labelledby="onboarding-title"
    @close="store.closeOnboarding"
  >
    <template #header>
      <div class="w-full">
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-xs font-semibold uppercase tracking-widest text-primary">
              Step {{ store.onboardingStep + 1 }} of 4 · {{ currentLabel }}
            </p>
            <h1 id="onboarding-title" ref="heading" tabindex="-1" data-autofocus class="mt-1 text-xl font-semibold text-fg outline-none">
              Let's set up your workspace
            </h1>
          </div>
          <TdButton
            variant="ghost"
            aria-label="Close setup and resume later"
            @click="store.closeOnboarding"
          >
            <template #icon-left><X /></template>
          </TdButton>
        </div>
        <ol class="mt-4 flex gap-2" aria-label="Setup progress">
          <li
            v-for="(step, index) in steps"
            :key="step"
            class="h-1 flex-1 rounded-full"
            :class="index <= store.onboardingStep ? 'bg-primary' : 'bg-surface-raised'"
          >
            <span class="sr-only">{{ step }}</span>
          </li>
        </ol>
      </div>
    </template>

    <OnboardingStepFind v-if="store.onboardingStep === 0" />
    <OnboardingStepChoose v-else-if="store.onboardingStep === 1" />
    <OnboardingStepGroups v-else-if="store.onboardingStep === 2" />
    <OnboardingStepReady v-else />
  </TdModal>
</template>
```

`TdModal` handles Escape, focus trap, and scroll lock. Remove the manual
`onKeydown` listener and `onMounted` focus. `useFocusTrap` excludes
`[tabindex='-1']`, so the heading is not part of the normal focus order; the
`data-autofocus` attribute above makes the trap prefer it. If you prefer not to
use `data-autofocus`, keep an explicit
`nextTick(() => heading.value?.focus())` in the wizard instead.

### OnboardingStepFind

- Wrap path and owner fields in `TdFormField` + `TdInput`.
- Wrap max depth in `TdFormField` + `TdInput type="number"`.
- Wrap grouping style in `TdFormField` + `TdSelect`.
- Replace errors with `TdAlert variant="error"`.
- Replace scanning text with `TdSpinner` + `role="status"`.
- Replace submit with `TdButton variant="primary" :loading="submitting || store.isScanning"`.

`TdInput` emits strings, while the draft stores a number and a
`GroupingStyle` union. Add explicit computed bridges:

```ts
const maxDepth = computed({
  get: () => String(store.onboardingDraft.maxDepth),
  set: (value: string) => {
    const parsed = Number.parseInt(value, 10);
    store.onboardingDraft.maxDepth = Number.isFinite(parsed) ? parsed : 1;
  },
});

const groupingStyle = computed({
  get: () => store.onboardingDraft.groupingStyle,
  set: (value: string) => {
    store.onboardingDraft.groupingStyle = value as GroupingStyle;
  },
});
```

Use `v-model="maxDepth"` on the number input and `v-model="groupingStyle"` on the
select. Import `GroupingStyle` from `../../types/domain`.

### OnboardingStepChoose

- Replace search with `TdInput` + prefix Search icon.
- Replace checkbox rows with `TdCheckbox` (default slot contains name, kind badge,
  path, branch).
- Replace "Select all visible"/"Clear selection"/empty-state buttons with
  `TdButton`.
- Replace kind chip with `TdBadge variant="neutral"`.
- Replace errors with `TdAlert variant="error"`.
- Replace Back/Continue with `TdButton variant="ghost"` / `TdButton variant="primary"`.

### OnboardingStepGroups

- Replace include checkbox with `TdCheckbox` and `aria-label`.
- Replace group name `<input>` with `TdInput` (or `TdFormField` if a label is
  visible).
- Replace count pill with `TdBadge`.
- Replace error with `TdAlert variant="error"`.
- Replace Back/Finish with `TdButton`.

### OnboardingStepReady

- Replace success icon/text with token classes.
- Replace cards with `border-border bg-surface` and `text-fg`/`text-fg-muted`.
- Replace error with `TdAlert variant="error"`.
- Replace action buttons with `TdButton` (`primary` for first group, `secondary`
  for All repos, `secondary` for additional groups).

**Validation:**

```bash
cd /topdata/repo-manager
pnpm typecheck
pnpm test
```

**Exit criteria:** Onboarding has no native form controls, no custom overlay, no
palette classes; Escape/backdrop behavior matches the old wizard; keyboard focus
stays inside the modal.

---

## Phase 9 — Verification, enforcement, documentation, and housekeeping

**Goal:** Prove the migration is complete, keep the rules visible, and leave the
repo in a releasable state.

### 9.1 Source checks

Run and require empty output:

```bash
cd /topdata/repo-manager
rg -nP "@topdata-software-gmbh/ui(?![-\w])" package.json src
rg -n "\b(?:bg|text|border|ring|from|via|to|divide|placeholder|accent|shadow|outline)-(?:(?:slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose)-[0-9]{2,3}|(?:white|black))\b" src --glob '!**/__tests__/**'
rg -n "#[0-9a-fA-F]{3,8}\b" src --glob '!**/__tests__/**'
rg -n "<(button|input|select|textarea)\b" src --glob '*.vue'
```

Allowed exceptions:

- `<table>`, `<thead>`, `<tbody>`, `<tr>`, `<th>`, `<td>` in table layout.
- `src/lib/groupDefaults.ts` legacy hex fallback (the function supports old
  saved values). No hard-coded hex literals remain in that file.
- `src-tauri/` is Rust and out of scope.

### 9.2 Enforced token-only source test

Add this test only after Phases 5–8 are complete, so it enforces the end state
instead of failing mid-migration:

[NEW FILE] `src/lib/__tests__/token-only.test.ts`

```ts
import { describe, expect, it } from 'vitest';

const sources = import.meta.glob('../../**/*.{vue,ts,css}', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;

const palettePattern =
  /\b(?:bg|text|border|ring|from|via|to|divide|placeholder|accent|shadow|outline)-(?:(?:slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose)-\d{2,3}|(?:white|black))\b/;

function appSource(): Array<[string, string]> {
  return Object.entries(sources).filter(
    ([file]) => !file.includes('__tests__') && !file.includes('.test.'),
  );
}

describe('token-only app styling', () => {
  it('does not use raw Tailwind palette classes outside the kit', () => {
    for (const [file, source] of appSource()) {
      expect(source, file).not.toMatch(palettePattern);
    }
  });

  it('does not use raw hex colours outside the kit', () => {
    for (const [file, source] of appSource()) {
      expect(source, file).not.toMatch(/#[0-9a-f]{3,8}\b/i);
    }
  });
});
```

If `import.meta.glob` reports a key that is intentionally exempt, add a narrow
allowlist with a comment explaining the domain reason. Do not exclude whole
directories to make the test pass.

### 9.3 Full validation

```bash
cd /topdata/topdata-design-system
pnpm install
pnpm build
pnpm typecheck
pnpm test
cd /topdata/repo-manager
pnpm install
pnpm typecheck
pnpm test
pnpm build
```

Manual smoke test (`pnpm dev`, then `pnpm tauri dev` if the environment allows):

1. First-run onboarding: scan folder, choose repos, confirm groups, ready.
2. Group editor: create/edit/delete, color swatches, advanced checkboxes.
3. Toolbar: branch select, Save/Get latest/Send changes, rescan, More menu
   keyboard navigation, help.
4. All repos: search, group/type selects, status filter chips, clear filters,
   empty states.
5. Save changes: loading, file changes, error alert, save.
6. Help modal: Escape and backdrop close.
7. Toasts: success, error, info, warning, dismiss.
8. Theme/token spot check: no slate/cyan remains; primary buttons use the
   TopData blue; surfaces match the token palette.

### 9.4 Documentation and registry

- [MODIFY] `/topdata/topdata-design-system/README.md` — package name and
  global class layer.
- [MODIFY] `/topdata/topdata-design-system/packages/README.md` — kit package
  name and one-kit-per-surface example.
- [MODIFY] `/topdata/topdata-design-system/packages/tokens/README.md` —
  `useTheme` package reference.
- [MODIFY] `/topdata/topdata-design-system/packages/ui-kit-web-vue3/README.md` —
  install, import order, component table, global class contract, deferred
  Tailwind/layout/brand variants.
- [MODIFY] `/topdata/topdata-design-system/packages/ui-component-registry.yaml` —
  package, source, status, notes.
- [MODIFY] `/topdata/topdata-design-system/_ai/specs/SPEC__261029-1__tauri-vue-ui-kit.md`
  — replace package names, add the global class requirement, and add the new
  component traceability rows.
- [MODIFY] `/topdata/topdata-design-system/_ai/specs/index.md` — add the eight
  `SPEC__261030-*` entries.
- [MODIFY] `/topdata/topdata-design-system/CHANGELOG.md` — Unreleased Added/
  Changed entries for the rename, global classes, and nine components.
- [MODIFY] `/topdata/repo-manager/README.md` — design-system section, new
  package name, token-only styling rule, dev/test commands unchanged.
- [MODIFY] `/topdata/repo-manager/CHANGELOG.md` — Unreleased Changed entries:
  kit migration, tokenized UI, new elements, no raw palette.
- [MODIFY] `/topdata/repo-manager/_ai/technical_decisions/README.md` — link the
  new ADR if the project index is expected to list design-system ADRs; otherwise
  leave the design-system ADR where it is.

README component table entry:

```markdown
| Component | Purpose |
|---|---|
| `TdButton` | Primary, secondary, ghost, danger; icon-only and loading states |
| `TdInput` | Text-like input with adornments, sizes, and states |
| `TdFormField` | Label, hint, error, required state, and slot wiring |
| `TdSelect` | Native select with options, sizes, and states |
| `TdTextarea` | Multiline text with rows, sizes, and states |
| `TdCheckbox` | Checkbox with label, description, and disabled state |
| `TdBadge` | Status/count pill with semantic variants |
| `TdAlert` | Inline info/success/warning/error message with actions |
| `TdSpinner` | Decorative or labelled loading indicator |
| `TdMenu` / `TdMenuItem` | Keyboard-accessible dropdown menu |
| `TdSegmentedControl` | Multi-select filter chips with counts |
| `TdModal` | Focus-trapped dialog with sizes and dismissal rules |
| `TdToast` / `TdToastStack` | Presentational toast item and stack |
| `useToast` | Visible window, queue, and timers for toasts |
| `useTheme` | Dark/light preference and persistence |
```

Traceability additions:

```markdown
| Component | Spec | Tests |
|---|---|---|
| `TdSelect` | `_ai/specs/SPEC__261030-1__td-select.md` | `src/components/TdSelect.test.ts` |
| `TdTextarea` | `_ai/specs/SPEC__261030-2__td-textarea.md` | `src/components/TdTextarea.test.ts` |
| `TdCheckbox` | `_ai/specs/SPEC__261030-3__td-checkbox.md` | `src/components/TdCheckbox.test.ts` |
| `TdBadge` | `_ai/specs/SPEC__261030-4__td-badge.md` | `src/components/TdBadge.test.ts` |
| `TdAlert` | `_ai/specs/SPEC__261030-5__td-alert.md` | `src/components/TdAlert.test.ts` |
| `TdSpinner` | `_ai/specs/SPEC__261030-6__td-spinner.md` | `src/components/TdSpinner.test.ts` |
| `TdMenu`, `TdMenuItem` | `_ai/specs/SPEC__261030-7__td-menu.md` | `src/components/TdMenu.test.ts` |
| `TdSegmentedControl` | `_ai/specs/SPEC__261030-8__td-segmented-control.md` | `src/components/TdSegmentedControl.test.ts` |
```

### 9.5 Housekeeping

- `.gitignore`: no change expected; the rename does not add a new artifact type
  and `dist/` is already ignored in both repos. Verify.
- `README.md`: update only the sections listed above; do not add empty docs.
- `CHANGELOG.md`: update as listed above.
- Remove any temporary scripts, screenshots, or local notes.
- Confirm no generated `dist/` files are staged.
- Re-run `git status` and review every changed file before committing.

**Exit criteria:** All validation commands pass; manual smoke test passes; source
checks are empty; docs/registry/changelog are current; no raw palette or native
form controls remain in app Vue files.

---

## Phase 10 — Write the implementation report

Create `_ai/backlog/reports/261030_1838__IMPLEMENTATION_REPORT__web_ui_kit_vue3_and_repo_manager_migration.md`
using this frontmatter:

```yaml
---
filename: "_ai/backlog/reports/261030_1838__IMPLEMENTATION_REPORT__web_ui_kit_vue3_and_repo_manager_migration.md"
title: "Report: Web UI Kit Vue 3 and Repo Manager Migration"
createdAt: 2026-10-30 18:38
updatedAt: 2026-10-30 18:38
planFile: "_ai/backlog/active/261030_1838__IMPLEMENTATION_PLAN__web_ui_kit_vue3_and_repo_manager_migration.md"
specFile: ""
project: "repo-manager"
status: completed
filesCreated: 0
filesModified: 0
filesDeleted: 0
tags: [ui-kit, vue3, design-system, tokens, repo-manager, migration]
documentType: IMPLEMENTATION_REPORT
---
```

Report sections:

1. **Summary** — what shipped and whether the app is fully token-only.
2. **Files Changed** — new kit components/styles/specs, modified consumers,
   deleted scoped styles, renamed package.
3. **Key Changes** — global class layer, nine components, token migration,
   repo-manager migration, registry/docs.
4. **Deviations from Plan** — anything discovered during implementation; if
   nothing, say so.
5. **Technical Decisions** — any API or CSS decisions made inside the plan's
   boundaries.
6. **Testing Notes** — exact commands and results, plus the manual smoke list.
7. **Usage Examples** — TdMenu, TdSegmentedControl, and token utilities.
8. **Documentation Updates** — README/CHANGELOG/registry/specs/ADR.
9. **Next Steps** — deferred `ui-kit-web-tailwind`, layout kit, brand variants,
   adapter schemas, switcher, TradeGuard migration ADR.

**Exit criteria:** Report exists, counts are accurate, and it links the plan,
ADR, and changed package paths.
