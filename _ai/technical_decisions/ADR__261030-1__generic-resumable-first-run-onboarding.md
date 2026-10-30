---
title: "Generic, Resumable First-Run Onboarding"
status: Accepted
date: 2026-10-30
deciders: [Marc]
tags: [onboarding, ux, configuration, discovery, open-source]
adrId: 261030-1
---

# Generic, Resumable First-Run Onboarding

## Context

Git Repo Manager is a public open-source application, but its first useful setup
depends on local conventions: where a person keeps repositories, which remote
owners belong to them, and how their folders map to useful groups. The current
guided empty state assumes the user knows which folder to add and what a Git
repository looks like. It also defaults to a shallow scan depth of three, which
misses common nested layouts such as
`<root>/<shop>/vol/www/custom/plugins/<repo>`.

Hardcoding one organization's names, remote hosts, or folder layouts into the
application would make the public repository misleading and would not survive
other users' environments. Setup must remain generic and locally configurable.
It also needs to survive an interrupted setup: a first-run flow that loses all
answers when closed is worse than no wizard.

The product decisions from the design discussion are:

- Setup is a full-screen, four-step flow: find projects, choose repositories,
  confirm groups, and a ready screen.
- Only one project folder is needed during setup. Additional roots can be added
  later through the existing scan-path flow.
- Owner names are optional and local-only. They are used to pre-select matching
  repositories, never to infer ownership from a vendor list.
- The scan is deep by default (eight levels), with an advanced override.
- Repositories are shown for acceptance; nothing is added to a group silently.
- Suggested groups are derived from the folder structure and can be renamed or
  skipped. They are seeds for curated groups, not a permanent automatic mapping.
- On completion, the ready screen explains **Save to Git**, **Get latest**, and
  **Send changes**, then offers to open a group. It does not force a commit.

## Decision

Add an optional, backward-compatible onboarding state to the persisted
configuration and implement a resumable first-run flow in the frontend.

- `AppConfig` gains an `onboarding` object with `completed`, `started`,
  `currentStep`, `scanPath`, `ownerNames`, `groupingStyle`, and
  `selectedRepoPaths`. Missing fields deserialize to defaults so existing
  configuration files keep working.
- The wizard opens only when onboarding is incomplete and either it has already
  started or the configuration has no scan paths and no groups. Existing
  configurations without an onboarding block are therefore treated as complete.
- Progress is persisted through a dedicated `save_onboarding_state` service and
  transport endpoints. The scanned repository list is not persisted, but the
  user's selected paths are recorded as choices; resuming rescans the chosen
  folder and revalidates those choices instead of trusting stale Git state.
- Owner names are entered by the user, stored in the local configuration, and
  matched generically against remote URLs. No organization names or hosts are
  compiled into the application.
- The setup's scan path uses a deep default of eight levels. The existing scan
  path modal remains available for later folders and keeps its own defaults.
- Suggested groups use the first folder level below the chosen scan root. If
  repositories sit directly below the root, the root folder name is used. The
  user can rename or exclude each suggestion and can change the grouping style
  between per-top-level-folder and one-group-for-this-folder.
- Completing setup marks onboarding complete and leaves the created groups as
  ordinary curated groups. Future scans never auto-add repositories to them;
  only the virtual all-repositories view reflects new discoveries.
- A persistent **Setup** entry reopens the flow for existing users without
  changing their completed state unless they finish the flow again.

## Consequences

Positive:

- The public project stays organization-neutral and testable with generic
  fixtures.
- Interrupted setup can resume without repeating folder selection or owner names.
- Existing users are not interrupted by a new first-run screen.
- Deep scanning finds nested clones while still allowing power users to tune the
  depth.

Negative:

- The configuration schema and both transports (Tauri and axum) need coordinated
  changes.
- Persisting setup state introduces another write path and validation surface.
- Folder-derived group suggestions will sometimes be imperfect; the rename and
  skip controls are essential, and the existing group editor remains the long-term
  correction mechanism.
- Rescanning on resume can be slower than restoring a cached list, but avoids
  acting on stale filesystem state.

## Alternatives Considered

- **Hardcoded organization/remote allowlist**: Rejected because the repository is
  public and one organization's conventions do not generalize.
- **Browser local storage only**: Rejected because desktop and web modes would
  behave differently, and setup would not survive a different browser profile.
- **Select every discovered repository by default**: Rejected because a scan can
  include third-party clones and hundreds of unrelated repositories.
- **A multi-folder wizard in the first release**: Rejected as unnecessary for the
  first useful setup; the existing scan-path flow already supports later roots.
- **A real persisted "all repositories" group**: Rejected because it would go
  stale as new repositories appear and would duplicate the virtual view.
