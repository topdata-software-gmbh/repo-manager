import type { RepoGroup } from '../types/domain';

export const GROUP_COLORS = [
  '#38bdf8',
  '#22c55e',
  '#f59e0b',
  '#f43f5e',
  '#a855f7',
  '#14b8a6',
] as const;

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
