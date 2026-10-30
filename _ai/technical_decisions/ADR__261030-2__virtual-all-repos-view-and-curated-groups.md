---
title: "Virtual All Repos View and Curated Groups"
status: Accepted
date: 2026-10-30
deciders: [Marc]
tags: [ui, navigation, groups, filtering, repository-metadata]
adrId: 261030-2
---

# Virtual All Repos View and Curated Groups

## Context

The application already has an **All projects** sidebar entry and a single table
that changes its contents when a group is selected. The group is a curated list
of repository paths. Users with many cloned shops can have well over a hundred
repositories, including third-party plugins, and need to browse or filter across
all of them without treating that entire set as one actionable group.

Making "all repositories" a persisted group would be misleading. New repositories
appear when folders are rescanned, so the group would either be stale or need
automatic membership updates. Automatic membership would also make bulk actions
surprising: saving or pushing "all repositories" could touch unrelated clones.

The design discussion established a separate, filterable all-repositories page
with these expectations:

- It is always available and always reflects all discovered repositories.
- It cannot be renamed, deleted, or treated as a curated group.
- It supports filters for status, group, repository type, and text search.
- It offers per-repository actions only. Bulk actions stay in deliberately opened
  groups.
- Normal groups remain curated lists and can be edited in the existing group
  editor.

The application also needs a repository type filter for Shopware-oriented work.
Type detection must be local and generic, not based on an organization name or a
remote URL allowlist.

## Decision

Introduce **All repos** as a pinned virtual view, backed by the existing
`selectedGroupId = 'all'` navigation state rather than a new persisted group.

- The sidebar entry is renamed from **All projects** to **All repos** and uses a
  distinct icon and total count. It has no edit or delete controls.
- Selecting it renders a dedicated `AllReposPage` instead of the group workspace.
  The group toolbar and guidance banner are hidden on this page.
- The all-repos page combines filters for:
  - status: needs attention, changes not saved, updates available, to send,
    up to date, on this computer only;
  - group: any, ungrouped, or a specific group;
  - type: theme, plugin, other;
  - search: repository name, path, or current branch.
- Filter state lives in the Pinia store so it survives navigation to a group and
  back during the session.
- The page exposes only row-level **Save to Git**, **Get latest**, and **Send
  changes** actions. Group views keep their existing bulk actions.
- Repository type is added to the backend `Repository` DTO as a small local
  heuristic: presence of a theme manifest marks a theme, a Shopware plugin
  package type marks a plugin, and everything else is other. The heuristic reads
  local metadata only and never contacts a remote.
- Normal groups remain curated static lists. New discoveries do not join them
  automatically; only the virtual view reflects them.

## Consequences

Positive:

- The distinction between "everything I can see" and "the set I intentionally
  act on" stays clear.
- Large workspaces remain browsable without risking bulk operations across
  unrelated repositories.
- The type filter is useful for Shopware work without embedding a vendor allowlist.
- The existing group editor and group action semantics remain unchanged.

Negative:

- The UI needs two list presentations: a group workspace and an all-repos browser.
  Shared row and status components must be extracted to avoid divergence.
- Type detection adds filesystem reads during repository inspection and an extra
  field to every repository DTO.
- Filter combinations and status taxonomy need explicit tests; otherwise the
  counts and result sets can drift.

## Alternatives Considered

- **Persist an "All repos" group**: Rejected because it would become stale or
  require automatic membership changes that make bulk actions unsafe.
- **Only upgrade the existing table with filters**: Rejected because the group
  toolbar's bulk actions would remain attached to the all-repositories view.
- **Bulk actions on the filtered all-repos set**: Rejected because filters can
  easily select more repositories than the user intends.
- **Path-only type detection**: Rejected because folder names are not reliable
  indicators of whether a repository is a theme or a plugin.
- **Remote-based type or ownership detection**: Rejected because it depends on
  network conventions and does not work for local or renamed remotes.
