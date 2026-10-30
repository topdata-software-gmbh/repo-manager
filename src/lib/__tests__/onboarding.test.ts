import { describe, expect, it } from 'vitest';
import {
  defaultOnboardingState,
  normalizeOwnerNames,
  ownedRepoPaths,
  parseRemoteOwner,
  shouldShowOnboarding,
  suggestGroups,
} from '../onboarding';
import type { AppConfig, Repository } from '../../types/domain';

function makeRepo(overrides: Partial<Repository> = {}): Repository {
  return {
    id: overrides.id ?? overrides.path ?? 'repo',
    name: 'repo',
    path: '/projects/repo',
    isAccessible: true,
    groupIds: [],
    kind: 'other',
    state: {
      currentBranch: 'main',
      isDetachedHead: false,
      headCommitHash: '',
      headCommitMessage: '',
      isClean: true,
      aheadCount: 0,
      behindCount: 0,
      modifiedFiles: 0,
      untrackedFiles: 0,
      stagedFiles: 0,
      hasConflicts: false,
      operationState: 'idle',
      remotes: [],
    },
    ...overrides,
  };
}

function makeConfig(overrides: Partial<AppConfig> = {}): AppConfig {
  return {
    scanPaths: [],
    groups: [],
    onboarding: defaultOnboardingState(),
    ...overrides,
  };
}

describe('shouldShowOnboarding', () => {
  it('shows for a fresh default config', () => {
    expect(shouldShowOnboarding(makeConfig())).toBe(true);
  });

  it('hides when scan paths already exist', () => {
    const config = makeConfig({
      scanPaths: [
        {
          id: 'scan-1',
          path: '/projects',
          maxDepth: 3,
          enabled: true,
          excludePatterns: [],
        },
      ],
    });
    expect(shouldShowOnboarding(config)).toBe(false);
  });

  it('shows when setup was started but not completed', () => {
    const config = makeConfig({
      onboarding: { ...defaultOnboardingState(), started: true },
    });
    expect(shouldShowOnboarding(config)).toBe(true);
  });

  it('hides when onboarding is completed', () => {
    const config = makeConfig({
      onboarding: { ...defaultOnboardingState(), completed: true },
    });
    expect(shouldShowOnboarding(config)).toBe(false);
  });
});

describe('normalizeOwnerNames', () => {
  it('trims, strips @, lowercases and dedupes', () => {
    expect(normalizeOwnerNames(' @Acme , acme\n@BETA , ,')).toEqual([
      'acme',
      'beta',
    ]);
  });
});

describe('parseRemoteOwner', () => {
  it('parses SCP-like remotes', () => {
    expect(parseRemoteOwner('git@github.com:Acme/repo.git')).toBe('acme');
  });

  it('parses HTTPS remotes', () => {
    expect(parseRemoteOwner('https://gitlab.com/Acme/repo.git')).toBe('acme');
  });

  it('ignores non-remote paths', () => {
    expect(parseRemoteOwner('/home/me/repo')).toBeNull();
  });

  it('ignores empty remotes', () => {
    expect(parseRemoteOwner('   ')).toBeNull();
  });
});

describe('ownedRepoPaths', () => {
  it('matches any remote owner case-insensitively', () => {
    const repos: Repository[] = [
      makeRepo({
        path: '/projects/owned',
        state: {
          ...makeRepo().state,
          remotes: [{ name: 'origin', url: 'git@github.com:Acme/owned.git' }],
        },
      }),
      makeRepo({
        path: '/projects/other',
        state: {
          ...makeRepo().state,
          remotes: [{ name: 'origin', url: 'https://github.com/other/other.git' }],
        },
      }),
    ];

    expect(ownedRepoPaths(repos, ['ACME'])).toEqual(['/projects/owned']);
  });

  it('returns nothing without owner names', () => {
    expect(ownedRepoPaths([makeRepo()], [])).toEqual([]);
  });
});

describe('suggestGroups', () => {
  it('creates two groups for two nested shop folders', () => {
    const groups = suggestGroups(
      [
        '/projects/shop-a/vol/www/custom/plugins/theme-a',
        '/projects/shop-b/vol/www/custom/plugins/theme-b',
      ],
      '/projects',
      'topLevel',
    );

    expect(groups).toEqual([
      { name: 'shop-a', repoPaths: ['/projects/shop-a/vol/www/custom/plugins/theme-a'] },
      { name: 'shop-b', repoPaths: ['/projects/shop-b/vol/www/custom/plugins/theme-b'] },
    ]);
  });

  it('names the group after the root for a single nested container', () => {
    const groups = suggestGroups(
      ['/projects/shop-a/vol/www/custom/plugins/theme-a'],
      '/projects',
      'topLevel',
    );

    expect(groups).toEqual([
      { name: 'projects', repoPaths: ['/projects/shop-a/vol/www/custom/plugins/theme-a'] },
    ]);
  });

  it('names the group after the root for side-by-side repositories', () => {
    const groups = suggestGroups(
      ['/projects/theme-a', '/projects/theme-b'],
      '/projects',
      'topLevel',
    );

    expect(groups).toEqual([
      { name: 'projects', repoPaths: ['/projects/theme-a', '/projects/theme-b'] },
    ]);
  });

  it('always produces one group with the single style', () => {
    const groups = suggestGroups(
      ['/projects/a/one', '/projects/b/two'],
      '/projects',
      'single',
    );

    expect(groups).toHaveLength(1);
    expect(groups[0].name).toBe('projects');
    expect(groups[0].repoPaths).toEqual(['/projects/a/one', '/projects/b/two']);
  });
});
