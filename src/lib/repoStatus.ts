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
