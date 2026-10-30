import { describe, expect, it } from 'vitest';
import { groupGitRuns, shopName } from '../gitLog';
import type { GitStepEvent } from '../../types/domain';

function step(overrides: Partial<GitStepEvent>): GitStepEvent {
  return {
    runId: 'run-1',
    repoPath: '/dev/shop-a',
    macroId: 'pull',
    title: 'Got the latest',
    stepIndex: 0,
    comment: 'Download the new commits',
    command: 'git -C /dev/shop-a pull --ff-only',
    status: 'ok',
    durationMs: 10,
    message: null,
    alias: 'gl',
    ...overrides,
  };
}

describe('groupGitRuns', () => {
  it('folds identical commands across shops into one row', () => {
    const runs = groupGitRuns([
      step({}),
      step({ repoPath: '/dev/shop-b' }),
      step({ repoPath: '/dev/shop-c' }),
    ]);
    expect(runs).toHaveLength(1);
    expect(runs[0].rows).toHaveLength(1);
    expect(runs[0].rows[0].repoPaths).toEqual([
      '/dev/shop-a',
      '/dev/shop-b',
      '/dev/shop-c',
    ]);
  });

  it('folds the same step even when each clone renders its own path', () => {
    const runs = groupGitRuns([
      step({ command: 'git -C /dev/shop-a pull --ff-only' }),
      step({ repoPath: '/dev/shop-b', command: 'git -C /dev/shop-b pull --ff-only' }),
      step({ repoPath: '/dev/shop-c', command: "git -C '/dev/shop c' pull --ff-only" }),
    ]);
    expect(runs).toHaveLength(1);
    expect(runs[0].rows).toHaveLength(1);
    expect(runs[0].rows[0].repoPaths).toHaveLength(3);
  });

  it('keeps distinct steps of one run in order', () => {
    const runs = groupGitRuns([
      step({ stepIndex: 0, comment: 'Set your unfinished work aside' }),
      step({ stepIndex: 1, comment: 'Download the new commits' }),
    ]);
    expect(runs[0].rows.map((row) => row.comment)).toEqual([
      'Set your unfinished work aside',
      'Download the new commits',
    ]);
  });

  it('promotes the worst status to the run', () => {
    const runs = groupGitRuns([
      step({}),
      step({ repoPath: '/dev/shop-b', status: 'failed' }),
    ]);
    expect(runs[0].status).toBe('failed');
    expect(runs[0].rows[0].status).toBe('failed');
  });

  it('lets a finished step replace its running event', () => {
    const runs = groupGitRuns([
      step({ status: 'running' }),
      step({ status: 'ok' }),
    ]);
    expect(runs[0].status).toBe('ok');
    expect(runs[0].rows[0].status).toBe('ok');
  });

  it('lists the newest run first', () => {
    const runs = groupGitRuns([
      step({ runId: 'older' }),
      step({ runId: 'newer' }),
    ]);
    expect(runs.map((run) => run.runId)).toEqual(['newer', 'older']);
  });

  it('derives a shop label from a path', () => {
    expect(shopName('/dev/shops/shop-a')).toBe('shop-a');
  });
});
