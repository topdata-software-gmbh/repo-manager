import type { RepoGroup } from '../types/domain';

export const GROUP_COLOR_TOKENS = [
  'primary',
  'accent',
  'success',
  'warning',
  'error',
  'info',
] as const;

export type GroupColorToken = (typeof GROUP_COLOR_TOKENS)[number];

export const GROUP_COLORS = GROUP_COLOR_TOKENS;

export function groupColorStyle(color: string): Record<string, string> {
  if (color.startsWith('#')) {
    return { backgroundColor: color };
  }
  return { backgroundColor: `var(--td-color-${color})` };
}

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
