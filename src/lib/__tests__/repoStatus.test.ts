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
