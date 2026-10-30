import type { GitLogRow, GitRun, GitStepEvent, GitStepStatus } from '../types/domain';

const STATUS_PRIORITY: Record<GitStepStatus, number> = {
  failed: 4,
  running: 3,
  ok: 2,
  skipped: 1,
};

function worst(statuses: GitStepStatus[]): GitStepStatus {
  return statuses.reduce<GitStepStatus>(
    (worstSoFar, status) =>
      STATUS_PRIORITY[status] > STATUS_PRIORITY[worstSoFar] ? status : worstSoFar,
    'skipped',
  );
}

export function shopName(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

/**
 * Replaces the project path of a rendered `git -C <path> ...` command with a
 * placeholder so the same step across several clones folds into one row.
 */
function commandKey(command: string): string {
  const prefix = 'git -C ';
  if (!command.startsWith(prefix)) return command;

  const rest = command.slice(prefix.length);
  if (!rest.startsWith("'")) {
    const end = rest.indexOf(' ');
    return end === -1 ? command : `${prefix}{repo}${rest.slice(end)}`;
  }

  let index = 1;
  while (index < rest.length) {
    if (rest[index] === "'") {
      // Shell quoting renders an inner quote as the four characters '\''.
      if (rest.slice(index, index + 4) === "'\\''") {
        index += 4;
        continue;
      }
      return `${prefix}{repo}${rest.slice(index + 1)}`;
    }
    index += 1;
  }

  return command;
}

interface RowState {
  row: GitLogRow;
  statuses: Map<string, GitStepStatus>;
}

export function groupGitRuns(steps: GitStepEvent[]): GitRun[] {
  const runs = new Map<string, GitRun>();
  const rowStates = new Map<string, RowState>();

  for (const step of steps) {
    const run = runs.get(step.runId) ?? {
      runId: step.runId,
      title: step.title,
      macroId: step.macroId,
      status: 'skipped' as GitStepStatus,
      durationMs: 0,
      rows: [],
      finishedAt: '',
    };

    const key = `${step.comment}\u0000${commandKey(step.command)}`;
    const stateKey = `${step.runId}\u0000${key}`;
    let state = rowStates.get(stateKey);

    if (!state) {
      state = {
        row: {
          key,
          stepIndex: step.stepIndex,
          comment: step.comment,
          command: step.command,
          alias: step.alias,
          status: step.status,
          repoPaths: [],
          message: null,
        },
        statuses: new Map<string, GitStepStatus>(),
      };
      rowStates.set(stateKey, state);
      run.rows.push(state.row);
    }

    if (!state.row.repoPaths.includes(step.repoPath)) {
      state.row.repoPaths.push(step.repoPath);
    }
    // Keep only the latest status per clone: a terminal event must be able to
    // replace the `running` event that preceded it.
    state.statuses.set(step.repoPath, step.status);
    state.row.status = worst([...state.statuses.values()]);
    state.row.message = state.row.message ?? step.message;

    run.durationMs = Math.max(run.durationMs, step.durationMs);
    runs.set(step.runId, run);
  }

  for (const run of runs.values()) {
    run.status = worst(run.rows.map((row) => row.status));
    run.rows.sort(
      (a, b) => a.stepIndex - b.stepIndex || a.comment.localeCompare(b.comment),
    );
  }

  return [...runs.values()].reverse();
}
