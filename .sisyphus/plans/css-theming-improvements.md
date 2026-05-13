# CSS & Theming Improvements

## TL;DR

> **Quick Summary**: Fix broken CSS (media queries, undefined variables), unify dual token systems, eliminate duplication (buttons, headings), remove dead code, and add lightweight SCSS utilities for maintainability.
>
> **Deliverables**:
> - Fixed media query bug (CSS vars → SCSS vars for breakpoints)
> - Defined missing CSS custom properties
> - Unified design token system (palette → semantic tokens)
> - Single button component class replacing 4-5 duplications
> - Breakpoint mixin eliminating magic numbers
> - Removed dead code (~50 lines of unused rules/variables)
> - All hardcoded values replaced with variables
>
> **Estimated Effort**: Medium
> **Parallel Execution**: YES - 3 waves
> **Critical Path**: Task 1 (SCSS utilities) → Tasks 2-4 (token/heading/button unification) → Tasks 5-8 (component cleanup)

---

## Context

### Original Request
Analyse the CSS and theming for improvements in the wrk_graphs project (Rust/Yew WASM app using SCSS via Trunk).

### Analysis Findings
The project has 13 SCSS files organised into base/layout/components. The analysis revealed:
- 2 actual bugs (broken media queries, undefined variables)
- 1 class collision (`.container`)
- Dual disconnected token systems
- Significant duplication (buttons ×5, headings ×2)
- Dead code (~50 lines never consumed)
- Hardcoded values bypassing the variable system
- Zero SCSS features leveraged despite `.scss` extension

### Key Decisions
- **Visual design stays the same** — this is refactoring, not redesign
- **CSS custom properties remain primary** — they enable runtime theming
- **SCSS variables added only for compile-time needs** (breakpoints, maps)
- **The `--main-base-*` palette is kept** as the source-of-truth palette
- **`--xkcd-*` variables preserved** — likely consumed by Rust-side SVG chart rendering

---

## Work Objectives

### Core Objective
Eliminate bugs, unify the token system, remove duplication, and improve maintainability without changing the visual appearance.

### Concrete Deliverables
- `styles/base/_variables.scss` — restructured with palette → semantic layering
- `styles/base/_base.scss` — heading duplication removed, media query bug fixed
- `styles/base/_theme.scss` — dead `.dark-background` rules removed
- `styles/utilities/_mixins.scss` (new) — breakpoint mixin
- `styles/utilities/_buttons.scss` (new) — shared button class
- All component files — hardcoded values replaced, button classes unified

### Definition of Done
- [ ] `mise run build` succeeds with no SCSS compilation errors
- [ ] No CSS custom properties referenced that aren't defined in `:root`
- [ ] No `px` or `rem` literals for spacing/sizing (except font-size rem values and SVG positioning)
- [ ] Zero duplicate button style blocks across components
- [ ] Media queries use SCSS variables, not CSS custom properties
- [ ] Visual appearance unchanged (verified via screenshot comparison)

### Must Have
- All bugs fixed (media queries, undefined variables)
- Token system unified (palette → semantic → component usage)
- Button duplication eliminated
- Heading styles in one place only
- `.container` collision resolved

### Must NOT Have (Guardrails)
- No visual design changes (colors, spacing, fonts must look identical)
- No changes to Rust/Yew component structure or HTML
- No new CSS frameworks or dependencies
- No removal of `--xkcd-*` or `--main-base-*` variables (may be used in Rust code)
- No over-engineering (no Sass maps for everything, no BEM conversion, no utility-class framework)
- No touching SVG chart positioning values (left/top/bottom in `_latency_percentile_chart.scss`) — those are layout-critical for chart rendering

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** - ALL verification is agent-executed. No exceptions.

### Test Decision
- **Infrastructure exists**: NO (no CSS test framework)
- **Automated tests**: NO — verification via build success + visual screenshot comparison
- **Framework**: N/A

### QA Policy
Every task verified by:
1. `mise run build` succeeds (SCSS compiles without errors)
2. Grep-based checks for banned patterns (hardcoded values, undefined vars)
3. Screenshot comparison for visual regression (Playwright on built dist/)

Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation — SCSS utilities + bug fixes):
├── Task 1: Add SCSS utilities file (breakpoint mixin + SCSS variables) [quick]
├── Task 2: Fix media query bug in _base.scss [quick]
├── Task 3: Define missing CSS custom properties in _variables.scss [quick]
└── Task 4: Resolve .container class collision [quick]

Wave 2 (Unification — token system + shared components):
├── Task 5: Unify token system in _variables.scss (depends: 3) [unspecified-high]
├── Task 6: Consolidate heading styles (depends: 1) [quick]
├── Task 7: Create shared button class + remove duplicates (depends: 5) [unspecified-high]
└── Task 8: Remove dead code from _theme.scss (depends: 6) [quick]

Wave 3 (Component cleanup — hardcoded values + final polish):
├── Task 9: Replace hardcoded values in _dashboard.scss + _layout.scss (depends: 1, 5) [quick]
├── Task 10: Replace hardcoded values in _metric-panel.scss + _latency_percentile_chart.scss (depends: 5) [quick]
├── Task 11: Replace hardcoded values in _modal.scss + _home.scss (depends: 5, 7) [quick]
└── Task 12: Final build verification + screenshot comparison (depends: all) [quick]

Wave FINAL (After ALL tasks — review):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Real QA - visual comparison (unspecified-high + playwright)
└── Task F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|-----------|--------|------|
| 1 | — | 2, 6, 9, 10, 11 | 1 |
| 2 | 1 | 12 | 1 |
| 3 | — | 5 | 1 |
| 4 | — | 11 | 1 |
| 5 | 3 | 7, 9, 10, 11 | 2 |
| 6 | 1 | 8 | 2 |
| 7 | 5 | 11 | 2 |
| 8 | 6 | 12 | 2 |
| 9 | 1, 5 | 12 | 3 |
| 10 | 5 | 12 | 3 |
| 11 | 5, 7, 4 | 12 | 3 |
| 12 | all | F1-F4 | 3 |

### Agent Dispatch Summary

- **Wave 1**: 4 tasks — T1-T4 → `quick`
- **Wave 2**: 4 tasks — T5 → `unspecified-high`, T6 → `quick`, T7 → `unspecified-high`, T8 → `quick`
- **Wave 3**: 4 tasks — T9-T11 → `quick`, T12 → `quick` (with `playwright` skill)
- **FINAL**: 4 tasks — F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high` + `playwright`, F4 → `deep`

---

## TODOs

- [ ] 1. Add SCSS Utilities File (Breakpoint Mixin + SCSS Variables)

  **What to do**:
  - Create `styles/utilities/_mixins.scss` with:
    - SCSS variables: `$breakpoint-mobile: 768px`, `$breakpoint-tablet: 1024px`, `$breakpoint-desktop: 52em`, `$breakpoint-desktop-lg: 60em`
    - A `respond-to($bp)` mixin that accepts `mobile`, `tablet`, `desktop` and emits `@media (max-width: ...)`
  - Add `@import 'styles/utilities/mixins';` to `index.scss` BEFORE the base imports (so all files can use it)

  **Must NOT do**:
  - Don't create a full utility-class framework
  - Don't convert existing media queries yet (that's later tasks)
  - Don't add Sass maps or complex abstractions

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []
    - Simple file creation + single import line addition

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2, 3, 4)
  - **Blocks**: Tasks 2, 6, 9, 10, 11
  - **Blocked By**: None (can start immediately)

  **References**:
  - `index.scss` — Import order (line 1-17). New import must go BEFORE line 2 (`@import 'styles/base/variables'`)
  - `styles/base/_base.scss:129-135` — The broken media queries that use `var(--main-max-width)` — these breakpoint values (`52em`, `60em`) are what the SCSS variables must match
  - `styles/components/_dashboard.scss:50,63` — Uses `1024px` and `768px` breakpoints (reference for what values the mixin needs)

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: SCSS utilities file compiles correctly
    Tool: Bash
    Preconditions: File created, import added to index.scss
    Steps:
      1. Run `mise run build`
      2. Check exit code is 0
      3. Grep `styles/utilities/_mixins.scss` for `@mixin respond-to`
    Expected Result: Build succeeds, mixin is defined with $breakpoint-mobile, $breakpoint-tablet variables
    Failure Indicators: Build error mentioning undefined mixin or import not found
    Evidence: .sisyphus/evidence/task-1-build-success.txt

  Scenario: Import order is correct (utilities before base)
    Tool: Bash
    Preconditions: index.scss modified
    Steps:
      1. Read index.scss and verify the utilities import appears before base imports
      2. Grep for the exact import line
    Expected Result: `@import 'styles/utilities/mixins'` appears on line 2 (before `@import 'styles/base/variables'`)
    Failure Indicators: Import appears after base, or missing entirely
    Evidence: .sisyphus/evidence/task-1-import-order.txt
  ```

  **Commit**: YES (groups with none — standalone)
  - Message: `style(scss): add breakpoint mixin and SCSS utility variables`
  - Files: `styles/utilities/_mixins.scss`, `index.scss`
  - Pre-commit: `mise run build`

- [ ] 2. Fix Media Query Bug in _base.scss

  **What to do**:
  - Replace `@media (max-width: var(--main-max-width))` (line 129) with `@media (max-width: $breakpoint-desktop)` using the SCSS variable from Task 1
  - Replace `@media (min-width: var(--main-max-width-lg))` (line 135) with `@media (min-width: $breakpoint-desktop-lg)`
  - These are the ONLY media queries that use CSS custom properties — fix only these two

  **Must NOT do**:
  - Don't change any other media queries in this file
  - Don't refactor the `.container` styles (that's Task 4)
  - Don't touch heading styles (that's Task 6)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Task 1 completes)
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 12
  - **Blocked By**: Task 1 (needs SCSS variables defined)

  **References**:
  - `styles/base/_base.scss:129-139` — The two broken media queries to fix
  - `styles/utilities/_mixins.scss` (created in Task 1) — Source of `$breakpoint-desktop` and `$breakpoint-desktop-lg` variables

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: No CSS custom properties in media queries
    Tool: Bash
    Preconditions: _base.scss edited
    Steps:
      1. Run `grep -n "@media.*var(--" styles/base/_base.scss`
      2. Assert output is empty (no matches)
      3. Run `grep -n "@media.*\$breakpoint" styles/base/_base.scss`
      4. Assert two matches (the fixed queries)
    Expected Result: Zero `var(--` in media queries, two `$breakpoint-` references
    Failure Indicators: Any line still using `var(--` inside `@media`
    Evidence: .sisyphus/evidence/task-2-media-query-fix.txt

  Scenario: Build still succeeds after fix
    Tool: Bash
    Preconditions: Both media queries updated
    Steps:
      1. Run `mise run build`
      2. Check exit code is 0
    Expected Result: Build passes with no errors
    Failure Indicators: SCSS compilation error about undefined variable
    Evidence: .sisyphus/evidence/task-2-build.txt
  ```

  **Commit**: YES (groups with 3, 4)
  - Message: `fix(scss): fix broken media queries, undefined vars, container collision`
  - Files: `styles/base/_base.scss`
  - Pre-commit: `mise run build`

- [ ] 3. Define Missing CSS Custom Properties

  **What to do**:
  - Add to `:root` in `styles/base/_variables.scss`:
    ```
    --color-text-on-dark: #f2f0ec;        /* = --main-base-07, text on dark backgrounds */
    --color-text-secondary-on-dark: #a09f93; /* = --main-base-04, secondary text on dark */
    ```
  - Place these in the Colors section (after line 8, near other text color vars)
  - Values derived from the existing `--main-base-*` palette to maintain visual consistency

  **Must NOT do**:
  - Don't change any existing variable values
  - Don't restructure the variables file (that's Task 5)
  - Don't remove any variables

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 2, 4)
  - **Blocks**: Task 5
  - **Blocked By**: None (can start immediately)

  **References**:
  - `styles/base/_variables.scss:1-14` — `:root` Colors section where new vars go
  - `styles/base/_variables.scss:64-65` — `--main-base-04: #a09f93` and `--main-base-07: #f2f0ec` (source palette values)
  - `styles/components/_metric-panel.scss:102,110` — Where `--color-text-secondary-on-dark` and `--color-text-on-dark` are consumed (currently undefined = bug)

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: All referenced CSS variables are now defined
    Tool: Bash
    Preconditions: Variables added to _variables.scss
    Steps:
      1. Extract all `var(--` references from all .scss files
      2. Extract all custom property definitions from _variables.scss
      3. Cross-reference: find any var() usage that has no definition
    Expected Result: Zero undefined variable references
    Failure Indicators: Any `var(--color-text-*-on-dark)` reference without matching `:root` definition
    Evidence: .sisyphus/evidence/task-3-undefined-vars.txt

  Scenario: Build compiles without error
    Tool: Bash
    Steps:
      1. Run `mise run build`
    Expected Result: Exit code 0
    Evidence: .sisyphus/evidence/task-3-build.txt
  ```

  **Commit**: YES (groups with 2, 4)
  - Message: `fix(scss): fix broken media queries, undefined vars, container collision`
  - Files: `styles/base/_variables.scss`
  - Pre-commit: `mise run build`

- [ ] 4. Resolve .container Class Collision

  **What to do**:
  - In `styles/components/_home.scss`, rename `.container` (line 1) to `.home-container`
  - Update all references within `_home.scss` that target `.container` (lines 1, 11, 56, 60) to `.home-container`
  - Verify in Rust source files: find where the `container` class is used in the home component and update to `home-container`
  - The base `.container` in `_base.scss:123-133` remains unchanged (it's the generic utility)

  **Must NOT do**:
  - Don't change the base `.container` class definition
  - Don't rename containers in other components (dashboard uses `--container-max-width` variable, not the class)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 2, 3)
  - **Blocks**: Task 11
  - **Blocked By**: None (can start immediately)

  **References**:
  - `styles/components/_home.scss:1-16, 55-63` — The `.container` rules to rename
  - `styles/base/_base.scss:123-133` — The base `.container` that must NOT be changed
  - `src/` — Rust/Yew source files where the HTML class `"container"` is applied on the home page (agent must grep `src/` for `"container"` usage in the home component)

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: No duplicate .container definitions
    Tool: Bash
    Steps:
      1. Grep all .scss files for `^\.container` (class definition at start of line)
      2. Assert only ONE match (in _base.scss)
      3. Grep _home.scss for `.home-container` — should exist
      4. Grep _home.scss for `^.container` — should NOT exist
    Expected Result: Single `.container` def in _base.scss, `.home-container` in _home.scss
    Failure Indicators: Multiple `.container` definitions across files
    Evidence: .sisyphus/evidence/task-4-container-collision.txt

  Scenario: Rust source updated to match
    Tool: Bash
    Steps:
      1. Grep `src/` recursively for the class usage in the home component
      2. Verify it uses `home-container` (not bare `container` where it was previously the home page's container)
    Expected Result: Home component Rust source references `home-container`
    Failure Indicators: Rust source still uses `container` class for home-specific layout
    Evidence: .sisyphus/evidence/task-4-rust-class.txt
  ```

  **Commit**: YES (groups with 2, 3)
  - Message: `fix(scss): fix broken media queries, undefined vars, container collision`
  - Files: `styles/components/_home.scss`, `src/**/*.rs` (home component)
  - Pre-commit: `mise run build`

- [ ] 5. Unify Token System in _variables.scss

  **What to do**:
  - Restructure `styles/base/_variables.scss` to establish a clear hierarchy:
    1. **Palette layer** (rename section header): Keep all `--main-base-*` values as-is. Add a comment: `/* Palette — source-of-truth colors. Do not use directly in components. */`
    2. **Semantic layer**: Make semantic tokens reference palette where values match:
       - `--color-bg-primary: var(--main-base-01);` (currently `#333333`, `--main-base-01` is `#393939` — KEEP original `#333333` if it doesn't match exactly)
       - Actually: audit each semantic token against palette. Only alias if value matches exactly. Otherwise keep the literal hex but add a comment noting which palette value is closest.
    3. **Keep `--xkcd-*` variables** in their own clearly-commented section: `/* Chart palette — consumed by Rust SVG rendering. Do not remove. */`
    4. **Remove `--main-max-width` and `--main-max-width-lg`** from CSS custom properties (they're now SCSS variables in Task 1, and CSS vars can't be used in media queries anyway)
    5. **Remove `--main-fonts`** from `:root` — replace with SCSS variable `$font-family-mono` in `_mixins.scss`, then update `_base.scss:20` to use it inline (body font-family is set once)
  - Reorganize sections with clear headers: Palette → Semantic Colors → Graph Colors → Spacing → Borders → Shadows → Transitions → Container

  **Must NOT do**:
  - Don't change any COLOR VALUES — only restructure, alias, and comment
  - Don't remove `--main-base-*` variables (used in `_criterion.scss`)
  - Don't remove `--xkcd-*` variables (may be used in Rust)
  - Don't rename any variable names (would break all consumers)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []
    - Requires careful cross-referencing of values across files

  **Parallelization**:
  - **Can Run In Parallel**: YES (within Wave 2)
  - **Parallel Group**: Wave 2 (with Tasks 6, 7, 8)
  - **Blocks**: Tasks 7, 9, 10, 11
  - **Blocked By**: Task 3 (new vars must be added first)

  **References**:
  - `styles/base/_variables.scss` (entire file) — The file being restructured
  - `styles/components/_criterion.scss:33,37` — Uses `--main-base-0b` and `--main-base-08` directly
  - `styles/base/_base.scss:18-20` — Uses `--main-base-00`, `--main-base-07`, `--main-fonts`
  - `styles/utilities/_mixins.scss` (from Task 1) — Where `$font-family-mono` will live alongside breakpoints

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: All existing variable consumers still resolve correctly
    Tool: Bash
    Preconditions: _variables.scss restructured
    Steps:
      1. Run `mise run build` — must pass
      2. Grep all .scss files for `var(--main-base-` — verify those vars still defined in :root
      3. Grep all .scss files for `var(--color-` — verify those vars still defined in :root
      4. Grep for `var(--main-max-width)` — should have zero results (removed, replaced by SCSS vars)
      5. Grep for `var(--main-fonts)` — should have zero results (replaced by SCSS variable)
    Expected Result: Build passes, all var() references resolve to defined properties
    Failure Indicators: Build error or grep finds undefined var references
    Evidence: .sisyphus/evidence/task-5-token-unification.txt

  Scenario: File is well-organized with clear section headers
    Tool: Bash
    Steps:
      1. Read `styles/base/_variables.scss`
      2. Verify sections exist: "Palette", "Semantic Colors", "Graph Colors", "Spacing", "Shadows"
      3. Verify `--xkcd-*` section has "consumed by Rust" comment
    Expected Result: Clear hierarchical organization with documentation comments
    Failure Indicators: Flat list without section separation, missing comments
    Evidence: .sisyphus/evidence/task-5-organization.txt
  ```

  **Commit**: YES (groups with 6, 7, 8)
  - Message: `refactor(scss): unify token system, consolidate headings and buttons`
  - Files: `styles/base/_variables.scss`, `styles/utilities/_mixins.scss`, `styles/base/_base.scss`
  - Pre-commit: `mise run build`

- [ ] 6. Consolidate Heading Styles

  **What to do**:
  - Remove ALL heading styles (`h1`–`h4`) from `styles/base/_base.scss` (lines 77-115) — these are overridden by `_theme.scss` and are dead code
  - Keep heading styles ONLY in `styles/base/_theme.scss` (lines 1-43) — this is the authoritative source
  - In `_theme.scss`, convert the hardcoded `rem` font-sizes to use the breakpoint mixin for responsive sizing:
    - Keep existing sizes as-is for desktop
    - Add `@include respond-to(mobile)` block to reduce sizes (matching what `_layout.scss:30-33` already does for `h1`)
  - Remove the responsive heading override from `styles/layout/_layout.scss:25-33` (`.header-content h1`) — it should be handled by the consolidated heading styles

  **Must NOT do**:
  - Don't change the actual font sizes (keep 2.5rem, 2rem, 1.75rem, 1.5rem, 1.25rem, 1rem)
  - Don't change heading colors, weight, or letter-spacing
  - Don't add heading styles for component-specific contexts (those remain in component files)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (within Wave 2)
  - **Parallel Group**: Wave 2 (with Tasks 5, 7, 8)
  - **Blocks**: Task 8
  - **Blocked By**: Task 1 (needs breakpoint mixin)

  **References**:
  - `styles/base/_base.scss:77-115` — Heading styles to REMOVE (dead code)
  - `styles/base/_theme.scss:1-43` — Heading styles to KEEP (authoritative)
  - `styles/layout/_layout.scss:25-33` — `.header-content h1` responsive override to remove
  - `styles/utilities/_mixins.scss` (from Task 1) — `respond-to(mobile)` mixin to use

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Headings defined in exactly one file
    Tool: Bash
    Steps:
      1. Grep all .scss files (excluding component files) for `^h[1-6]` declarations
      2. Assert matches ONLY in `_theme.scss`
      3. Verify `_base.scss` has no h1-h6 style rules
    Expected Result: Only `_theme.scss` defines base heading styles
    Failure Indicators: Heading definitions in _base.scss or _layout.scss
    Evidence: .sisyphus/evidence/task-6-headings-consolidated.txt

  Scenario: Build succeeds
    Tool: Bash
    Steps:
      1. Run `mise run build`
    Expected Result: Exit code 0
    Evidence: .sisyphus/evidence/task-6-build.txt
  ```

  **Commit**: YES (groups with 5, 7, 8)
  - Message: `refactor(scss): unify token system, consolidate headings and buttons`
  - Files: `styles/base/_base.scss`, `styles/base/_theme.scss`, `styles/layout/_layout.scss`
  - Pre-commit: `mise run build`

- [ ] 7. Create Shared Button Class + Remove Duplicates

  **What to do**:
  - Create `styles/utilities/_buttons.scss` with a `.btn-primary` class:
    ```scss
    .btn-primary {
      background-color: var(--color-accent);
      color: var(--color-text-primary);
      border: 2px solid var(--color-border);
      border-radius: var(--radius-sm);
      padding: var(--spacing-sm) var(--spacing-md);
      font-size: 1.1rem;
      font-weight: 700;
      cursor: pointer;
      transition: all var(--transition-fast);
      box-shadow: var(--shadow-sm);
      text-transform: uppercase;

      &:hover {
        background-color: var(--color-accent-hover);
        transform: translateY(-2px);
        box-shadow: var(--shadow-md);
      }
      &:active {
        transform: translateY(0);
        box-shadow: var(--shadow-sm);
      }
      &:disabled {
        opacity: 0.6;
        cursor: not-allowed;
      }
    }
    ```
  - Add `@import 'styles/utilities/buttons';` to `index.scss` (after mixins, before components)
  - Remove duplicate button styles from:
    - `styles/base/_theme.scss:117-145` — Remove entire `button` block (or reduce to just `font: inherit` reset)
    - `styles/components/_home.scss:33-53` — Replace with `@extend .btn-primary` or note that Rust component should use `btn-primary` class
    - `styles/components/_dashboard.scss:18-35` — Replace share-button styles
    - `styles/components/_wrk_config.scss:79-108` — Replace copy-button styles (keep `.copied` state variant)
    - `styles/components/_error.scss:41-58` — Replace primary-button styles
  - For component-specific overrides (like `.copied` state), keep ONLY the unique properties
  - Update Rust source files where button classes are applied — add `btn-primary` alongside existing classes (don't remove existing classes as they may be used for JS targeting)

  **Must NOT do**:
  - Don't change button appearance (same colors, sizes, transitions)
  - Don't remove component-specific state classes (`.copied`, etc.)
  - Don't create multiple button variants unless they already visually differ

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []
    - Touches many files, needs careful cross-referencing with Rust source

  **Parallelization**:
  - **Can Run In Parallel**: YES (within Wave 2)
  - **Parallel Group**: Wave 2 (with Tasks 5, 6, 8)
  - **Blocks**: Task 11
  - **Blocked By**: Task 5 (token system must be finalized first)

  **References**:
  - `styles/base/_theme.scss:117-145` — Current global `button` styles
  - `styles/components/_home.scss:33-53` — `.share-section .share-button` duplicate
  - `styles/components/_dashboard.scss:18-35` — `.dashboard .share-button` duplicate
  - `styles/components/_wrk_config.scss:79-108` — `.command-preview .copy-button` duplicate (has `.copied` variant)
  - `styles/components/_error.scss:41-58` — `.error-actions .primary-button` duplicate
  - `src/` — Rust source files where button class names are set (grep for `"share-button"`, `"copy-button"`, `"primary-button"`)

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Single button definition, no duplicates
    Tool: Bash
    Steps:
      1. Grep all .scss files for `background-color: var(--color-accent)` combined with `cursor: pointer`
      2. Assert at most 2 matches: one in _buttons.scss (.btn-primary) and optionally one variant
      3. Verify _buttons.scss contains `.btn-primary` with hover/active/disabled states
    Expected Result: Button styles defined once, component files only have overrides
    Failure Indicators: Full button style blocks still in component files
    Evidence: .sisyphus/evidence/task-7-button-dedup.txt

  Scenario: Build succeeds and buttons still render
    Tool: Bash
    Steps:
      1. Run `mise run build`
      2. Verify exit code 0
    Expected Result: Clean build
    Evidence: .sisyphus/evidence/task-7-build.txt
  ```

  **Commit**: YES (groups with 5, 6, 8)
  - Message: `refactor(scss): unify token system, consolidate headings and buttons`
  - Files: `styles/utilities/_buttons.scss`, `index.scss`, `styles/base/_theme.scss`, `styles/components/_home.scss`, `styles/components/_dashboard.scss`, `styles/components/_wrk_config.scss`, `styles/components/_error.scss`
  - Pre-commit: `mise run build`

- [ ] 8. Remove Dead Code from _theme.scss

  **What to do**:
  - Remove all `.dark-background` rules from `styles/base/_theme.scss`:
    - Lines 17-24: `.dark-background h1-h6` (redundant — same color as line 11)
    - Lines 54-56: `.dark-background p`
    - Lines 80-83: `.dark-background ul, .dark-background ol`
  - Remove the comment on line 16 ("Specific heading color for dark backgrounds")
  - Verify by grepping all Rust source and SCSS files for `dark-background` class usage — confirm zero matches

  **Must NOT do**:
  - Don't remove any non-`.dark-background` rules
  - Don't change colors or other properties in remaining rules
  - Don't remove the `::selection` or `:focus-visible` rules at the end of the file

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (within Wave 2)
  - **Parallel Group**: Wave 2 (with Tasks 5, 6, 7)
  - **Blocks**: Task 12
  - **Blocked By**: Task 6 (heading consolidation should complete first to avoid merge conflicts)

  **References**:
  - `styles/base/_theme.scss:16-24, 54-56, 80-83` — The dead `.dark-background` rules to remove
  - `src/` — Grep for `"dark-background"` to confirm it's never applied as a class

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: No .dark-background references remain
    Tool: Bash
    Steps:
      1. Grep all .scss and .rs files for `dark-background`
      2. Assert zero matches
      3. Run `mise run build`
    Expected Result: Zero references, clean build
    Failure Indicators: Any file still references dark-background
    Evidence: .sisyphus/evidence/task-8-dead-code-removed.txt
  ```

  **Commit**: YES (groups with 5, 6, 7)
  - Message: `refactor(scss): unify token system, consolidate headings and buttons`
  - Files: `styles/base/_theme.scss`
  - Pre-commit: `mise run build`

- [ ] 9. Replace Hardcoded Values in _dashboard.scss + _layout.scss

  **What to do**:
  - In `styles/components/_dashboard.scss`:
    - Line 16: `gap: 0.5rem` → `gap: var(--spacing-sm)`
    - Line 19: `padding: 0.5rem 1rem` → `padding: var(--spacing-sm) var(--spacing-md)`
    - Line 23: `border-radius: 4px` → `border-radius: var(--radius-sm)` (note: radius-sm is 8px — if 4px is intentional, add `--radius-xs: 4px` to variables first)
    - Line 26: `transition: background-color 0.2s ease` → `transition: background-color var(--transition-fast)`
  - In `styles/layout/_layout.scss`:
    - Line 4: `gap: 1rem` → `gap: var(--spacing-md)`
    - Line 6: `transition: opacity 0.2s` → `transition: opacity var(--transition-fast)`
    - Line 22: `margin-bottom: 1rem` → `margin-bottom: var(--spacing-md)`
  - Convert media queries to use mixin: `@media (max-width: 768px)` → `@include respond-to(mobile)`

  **Must NOT do**:
  - Don't change the grid-template-areas or grid structure
  - Don't change breakpoint values (768px, 1024px remain the same, just expressed via mixin)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 10, 11)
  - **Blocks**: Task 12
  - **Blocked By**: Tasks 1, 5

  **References**:
  - `styles/components/_dashboard.scss:16, 19, 23, 26, 50, 63` — Hardcoded values + media queries
  - `styles/layout/_layout.scss:4, 6, 22, 30` — Hardcoded values + media query
  - `styles/base/_variables.scss` — Variable names to use
  - `styles/utilities/_mixins.scss` — `respond-to` mixin

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: No hardcoded spacing/transition values remain
    Tool: Bash
    Steps:
      1. Grep `_dashboard.scss` for bare `0.5rem`, `1rem` (not inside var()), `0.2s`, `4px`
      2. Grep `_layout.scss` for bare `1rem`, `0.2s`
      3. Assert zero matches for spacing/transition literals
    Expected Result: All values use CSS custom properties or SCSS mixin
    Failure Indicators: Hardcoded spacing or transition values found
    Evidence: .sisyphus/evidence/task-9-hardcoded-values.txt
  ```

  **Commit**: YES (groups with 10, 11)
  - Message: `refactor(scss): replace hardcoded values with design tokens`
  - Files: `styles/components/_dashboard.scss`, `styles/layout/_layout.scss`
  - Pre-commit: `mise run build`

- [ ] 10. Replace Hardcoded Values in _metric-panel.scss + _latency_percentile_chart.scss

  **What to do**:
  - In `styles/components/_metric-panel.scss`:
    - Line 47: `padding: 8px 0` → `padding: var(--spacing-sm) 0`
    - Line 124: `padding: 2px 8px` → `padding: var(--spacing-xs) var(--spacing-sm)`
  - In `styles/components/_latency_percentile_chart.scss`:
    - Line 88: `margin-bottom: 20px` → `margin-bottom: var(--spacing-lg)`
    - Lines 59, 78: `font-size: 14px` → `font-size: 0.875rem` (keep as rem, not a variable — this is chart-specific)
    - Lines 51-53, 68-72: **LEAVE AS-IS** — these are SVG chart positioning values (`bottom: 10px`, `left: 40px`, `width: calc(100% - 60px)`) that are layout-critical for chart rendering
  - Convert media queries to use mixin where present

  **Must NOT do**:
  - Don't change SVG positioning values (10px, 40px, 60px offsets in chart axes)
  - Don't change chart point sizes (r: 4, r: 5)
  - Don't alter the `.percentile-line` stroke-width values

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 9, 11)
  - **Blocks**: Task 12
  - **Blocked By**: Task 5

  **References**:
  - `styles/components/_metric-panel.scss:15, 30, 47, 61, 71, 81, 124` — Hardcoded values + media queries
  - `styles/components/_latency_percentile_chart.scss:51-53, 59, 68-72, 78, 88` — Values to assess (some keep, some replace)
  - `styles/base/_variables.scss` — Variable names for replacement values

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Spacing values use variables (except chart positioning)
    Tool: Bash
    Steps:
      1. Grep `_metric-panel.scss` for bare `8px` — should be zero
      2. Grep `_latency_percentile_chart.scss` for `margin-bottom: 20px` — should be zero
      3. Verify chart positioning (10px, 40px, 60px) is STILL present (intentionally kept)
    Expected Result: Spacing uses variables; chart positioning unchanged
    Failure Indicators: Chart positioning altered, or spacing still hardcoded
    Evidence: .sisyphus/evidence/task-10-hardcoded-values.txt
  ```

  **Commit**: YES (groups with 9, 11)
  - Message: `refactor(scss): replace hardcoded values with design tokens`
  - Files: `styles/components/_metric-panel.scss`, `styles/components/_latency_percentile_chart.scss`
  - Pre-commit: `mise run build`

- [ ] 11. Replace Hardcoded Values in _modal.scss + _home.scss

  **What to do**:
  - In `styles/components/_modal.scss`:
    - Line 4: `rgba(0, 0, 0, 0.7)` → Define `--color-overlay: rgba(0, 0, 0, 0.7)` in _variables.scss, then use `var(--color-overlay)`
    - Line 53: `border: 1px solid` → `border: 1px solid` (keep — 1px borders are intentionally thinner than 2px elsewhere for form fields)
  - In `styles/components/_home.scss`:
    - Line 23: `max-width: 800px` → Consider adding `--container-narrow: 800px` variable or keep as-is (it's a one-off constraint)
    - Verify all spacing values already use variables (they do — this file is mostly clean after Task 4's rename)
  - Convert any remaining `@media (max-width: 768px)` to `@include respond-to(mobile)`
  - Add `--color-overlay` to `styles/base/_variables.scss` in the Colors section

  **Must NOT do**:
  - Don't change modal sizing (90%, max-width 600px, max-height 90vh)
  - Don't alter form field border widths (1px is intentionally different from 2px accent borders)
  - Don't remove the `max-width: 800px` if you're unsure — keep it as-is rather than over-abstracting

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 9, 10)
  - **Blocks**: Task 12
  - **Blocked By**: Tasks 4, 5, 7

  **References**:
  - `styles/components/_modal.scss:4` — `rgba(0, 0, 0, 0.7)` overlay color
  - `styles/components/_home.scss:55-73` — Media query to convert to mixin
  - `styles/base/_variables.scss` — Where `--color-overlay` will be added

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: No raw rgba() in component files
    Tool: Bash
    Steps:
      1. Grep `styles/components/` for `rgba(` — should be zero
      2. Grep `styles/base/_variables.scss` for `--color-overlay` — should exist
      3. Run `mise run build`
    Expected Result: rgba moved to variable, build passes
    Failure Indicators: Raw rgba still in component, or variable undefined
    Evidence: .sisyphus/evidence/task-11-hardcoded-values.txt
  ```

  **Commit**: YES (groups with 9, 10)
  - Message: `refactor(scss): replace hardcoded values with design tokens`
  - Files: `styles/components/_modal.scss`, `styles/components/_home.scss`, `styles/base/_variables.scss`
  - Pre-commit: `mise run build`

- [ ] 12. Final Build Verification + Screenshot Comparison

  **What to do**:
  - Run full build: `mise run build`
  - Serve the built `dist/` directory locally
  - Take screenshots of all pages using Playwright:
    - Home page (index)
    - Dashboard page (if accessible via URL hash/route)
    - Error/404 page
  - Compare against baseline (take baseline screenshots BEFORE starting this plan)
  - Verify no visual regressions
  - Run final grep checks:
    - `grep -r "rgba\|#[0-9a-f]" styles/components/` → No hardcoded colors in components
    - `grep -rn "@media.*var(--" styles/` → No CSS vars in media queries
    - Verify all `var(--` references resolve to defined properties

  **Must NOT do**:
  - Don't make any code changes in this task — verification only
  - Don't skip the screenshot comparison

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: [`playwright`]
    - Playwright needed for screenshot capture and visual comparison

  **Parallelization**:
  - **Can Run In Parallel**: NO (must be last)
  - **Parallel Group**: Sequential (after Wave 3)
  - **Blocks**: F1-F4
  - **Blocked By**: All previous tasks (9, 10, 11)

  **References**:
  - `dist/` — Built output directory
  - `index.html` — Entry point for the app
  - All success criteria commands from plan footer

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Full build passes
    Tool: Bash
    Steps:
      1. Run `mise run build`
      2. Assert exit code 0
      3. Verify `dist/` directory contains index.html and CSS output
    Expected Result: Clean production build
    Evidence: .sisyphus/evidence/task-12-build.txt

  Scenario: No banned patterns in codebase
    Tool: Bash
    Steps:
      1. Run `grep -rn "@media.*var(--" styles/` — expect zero results
      2. Run `grep -rn "rgba\|rgb(" styles/components/` — expect zero results
      3. Extract all var(--X) usages, cross-check against :root definitions
    Expected Result: Zero banned patterns, all variables defined
    Evidence: .sisyphus/evidence/task-12-grep-checks.txt

  Scenario: Visual comparison (no regression)
    Tool: Playwright
    Steps:
      1. Serve dist/ via `python3 -m http.server 8080` (or similar)
      2. Navigate to localhost:8080
      3. Screenshot home page → compare with baseline
      4. Navigate to a dashboard route if available
      5. Screenshot → compare
    Expected Result: Screenshots match baseline within tolerance (< 1% pixel diff)
    Failure Indicators: Visible layout shifts, missing elements, color changes
    Evidence: .sisyphus/evidence/task-12-screenshot-home.png, .sisyphus/evidence/task-12-screenshot-dashboard.png
  ```

  **Commit**: NO (verification only)

---

## Final Verification Wave

> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (grep for patterns, check file contents). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in .sisyphus/evidence/. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `mise run build`. Check all SCSS files for: undefined CSS custom properties (grep for `var(--` and cross-reference with `:root`), hardcoded color hex values outside `:root`, duplicate rule blocks, unused selectors. Verify SCSS compiles cleanly.
  Output: `Build [PASS/FAIL] | Undefined Vars [N] | Hardcoded Colors [N] | Duplicates [N] | VERDICT`

- [ ] F3. **Real QA — Visual Comparison** — `unspecified-high` + `playwright` skill
  Build the project (`mise run build`). Serve `dist/` locally. Take screenshots of: home page, dashboard page (use sample data), error page. Compare against baseline screenshots taken BEFORE changes. Verify no visual regressions.
  Output: `Pages [N/N match] | Regressions [NONE/list] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff (git diff). Verify 1:1 — everything in spec was built, nothing beyond spec was built. Check "Must NOT do" compliance (no Rust changes, no visual changes, no new dependencies). Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

| After Task(s) | Commit Message | Files |
|--------------|----------------|-------|
| 1 | `style(scss): add breakpoint mixin and SCSS utility variables` | `styles/utilities/_mixins.scss`, `index.scss` |
| 2, 3, 4 | `fix(scss): fix broken media queries, undefined vars, container collision` | `styles/base/_base.scss`, `styles/base/_variables.scss`, `styles/components/_home.scss` |
| 5, 6, 7, 8 | `refactor(scss): unify token system, consolidate headings and buttons` | `styles/base/_variables.scss`, `styles/base/_base.scss`, `styles/base/_theme.scss`, `styles/utilities/_buttons.scss`, `index.scss`, component files |
| 9, 10, 11 | `refactor(scss): replace hardcoded values with design tokens` | `styles/components/*.scss`, `styles/layout/_layout.scss` |
| 12 | No commit — verification only |

---

## Success Criteria

### Verification Commands
```bash
mise run build              # Expected: exits 0, no SCSS errors
grep -r "rgba\|#[0-9a-f]" styles/components/  # Expected: no hardcoded colors in components
grep -rn "var(--color-text.*on-dark" styles/   # Expected: only references to defined vars
grep -rn "@media.*var(--" styles/              # Expected: no results (bug is fixed)
```

### Final Checklist
- [ ] `mise run build` passes
- [ ] All "Must Have" items implemented
- [ ] All "Must NOT Have" items absent
- [ ] Visual appearance identical to before (screenshot evidence)
- [ ] No undefined CSS custom property references
- [ ] No CSS custom properties in media queries
