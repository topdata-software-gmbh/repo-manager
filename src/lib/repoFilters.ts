import type { RepoGroup, RepoKind, Repository } from '../types/domain';

export type RepoStatusFilter =
  | 'needsAttention'
  | 'changesNotSaved'
  | 'updatesAvailable'
  | 'toSend'
  | 'upToDate'
  | 'localOnly';

export interface AllReposFilters {
  statuses: RepoStatusFilter[];
  groupId: string;
  kind: RepoKind | 'all';
  search: string;
}

export const emptyAllReposFilters = (): AllReposFilters => ({
  statuses: [],
  groupId: 'all',
  kind: 'all',
  search: '',
});

export function matchesStatus(
  repo: Repository,
  status: RepoStatusFilter,
): boolean {
  switch (status) {
    case 'needsAttention':
      return (
        !repo.isAccessible ||
        repo.state.hasConflicts ||
        repo.state.operationState !== 'idle'
      );
    case 'changesNotSaved':
      return !repo.state.isClean;
    case 'updatesAvailable':
      return repo.isAccessible && repo.state.behindCount > 0;
    case 'toSend':
      return repo.isAccessible && repo.state.aheadCount > 0;
    case 'upToDate':
      return (
        repo.isAccessible &&
        repo.state.isClean &&
        repo.state.aheadCount === 0 &&
        repo.state.behindCount === 0 &&
        repo.state.remotes.length > 0 &&
        !repo.state.hasConflicts &&
        repo.state.operationState === 'idle'
      );
    case 'localOnly':
      return repo.isAccessible && repo.state.remotes.length === 0;
  }
}

export function filterAllRepos(
  repos: Repository[],
  filters: AllReposFilters,
  groups: RepoGroup[],
): Repository[] {
  const search = filters.search.trim().toLowerCase();
  const group =
    filters.groupId === 'all' || filters.groupId === 'ungrouped'
      ? null
      : groups.find((candidate) => candidate.id === filters.groupId) ?? null;

  return repos.filter((repo) => {
    if (
      filters.statuses.length > 0 &&
      !filters.statuses.some((status) => matchesStatus(repo, status))
    ) {
      return false;
    }

    if (filters.groupId === 'ungrouped') {
      const grouped = groups.some((candidate) =>
        candidate.repoPaths.includes(repo.path),
      );
      if (grouped) return false;
    } else if (group && !group.repoPaths.includes(repo.path)) {
      return false;
    }

    if (filters.kind !== 'all' && repo.kind !== filters.kind) return false;

    if (search) {
      const haystack =
        `${repo.name} ${repo.path} ${repo.state.currentBranch}`.toLowerCase();
      if (!haystack.includes(search)) return false;
    }

    return true;
  });
}

export function countByStatus(
  repos: Repository[],
  status: RepoStatusFilter,
): number {
  return repos.filter((repo) => matchesStatus(repo, status)).length;
}
