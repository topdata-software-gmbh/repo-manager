export interface ScanPath {
  id: string;
  path: string;
  maxDepth: number;
  enabled: boolean;
  excludePatterns: string[];
}

export interface RemoteInfo {
  name: string;
  url: string;
}

export type OperationState = 'idle' | 'merging' | 'rebasing' | 'cherry-picking';

export type RepoKind = 'theme' | 'plugin' | 'other';
export type GroupingStyle = 'topLevel' | 'single';

export interface OnboardingState {
  completed: boolean;
  started: boolean;
  currentStep: number;
  scanPath: string | null;
  ownerNames: string[];
  groupingStyle: GroupingStyle;
  selectedRepoPaths: string[];
}

export interface GitRepoState {
  currentBranch: string;
  isDetachedHead: boolean;
  headCommitHash: string;
  headCommitMessage: string;
  isClean: boolean;
  aheadCount: number;
  behindCount: number;
  modifiedFiles: number;
  untrackedFiles: number;
  stagedFiles: number;
  hasConflicts: boolean;
  operationState: OperationState;
  remotes: RemoteInfo[];
}

export interface Repository {
  id: string;
  name: string;
  path: string;
  isAccessible: boolean;
  state: GitRepoState;
  groupIds: string[];
  kind: RepoKind;
}

export interface GroupSettings {
  syncBranching: boolean;
  autoStashOnSwitch: boolean;
  defaultBaseBranch: string;
}

export interface RepoGroup {
  id: string;
  name: string;
  color: string;
  repoPaths: string[];
  settings: GroupSettings;
}

export interface AppConfig {
  scanPaths: ScanPath[];
  groups: RepoGroup[];
  onboarding: OnboardingState;
}

export interface OperationResult {
  repoPath: string;
  success: boolean;
  output: string;
  error: string | null;
}

export interface BranchInfo {
  name: string;
  repoCount: number;
}

export type FileChangeStatus = 'modified' | 'new' | 'deleted' | 'renamed' | 'conflict';

export interface FileChange {
  path: string;
  status: FileChangeStatus;
}

export interface OperationProgressEvent {
  repoPath: string;
  operation: 'fetch' | 'pull' | 'push' | 'checkout' | 'stash';
  status: 'pending' | 'running' | 'success' | 'failed';
  message?: string;
}
