import type { Repository } from '../types/domain';

export interface StatusBadge {
  label: string;
  className: string;
}

export function statusBadges(repo: Repository): StatusBadge[] {
  if (!repo.isAccessible) {
    return [{ label: 'Folder not found', className: 'bg-rose-500/15 text-rose-300' }];
  }

  const badges: StatusBadge[] = [];

  if (repo.state.hasConflicts) {
    badges.push({ label: 'Needs attention', className: 'bg-rose-500/15 text-rose-300' });
  }

  if (repo.state.operationState !== 'idle') {
    badges.push({
      label: operationLabel(repo.state.operationState),
      className: 'bg-amber-500/15 text-amber-300',
    });
  }

  if (!repo.state.isClean) {
    badges.push({ label: 'Changes not saved', className: 'bg-amber-500/10 text-amber-300' });
  }

  if (repo.state.behindCount > 0) {
    badges.push({
      label: `${repo.state.behindCount} ${repo.state.behindCount === 1 ? 'update' : 'updates'} available`,
      className: 'bg-sky-500/10 text-sky-300',
    });
  }

  if (repo.state.aheadCount > 0) {
    badges.push({
      label: `${repo.state.aheadCount} to send`,
      className: 'bg-emerald-500/10 text-emerald-300',
    });
  }

  if (repo.state.remotes.length === 0) {
    badges.push({ label: 'On this computer only', className: 'bg-slate-800 text-slate-400' });
  } else if (repo.state.isClean && repo.state.aheadCount === 0 && repo.state.behindCount === 0) {
    badges.push({ label: 'Up to date', className: 'bg-emerald-500/10 text-emerald-300' });
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
