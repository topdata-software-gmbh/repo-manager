import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import { backend } from '../lib/backend';
import type {
  AppConfig,
  BranchInfo,
  OperationResult,
  RepoGroup,
  Repository,
  ScanPath,
} from '../types/domain';

export type ToastKind = 'success' | 'error' | 'info';

export interface Toast {
  id: number;
  kind: ToastKind;
  title: string;
  message?: string;
}

export type InsightAction = 'save' | 'pull' | 'push';

export interface GroupInsight {
  tone: 'success' | 'info' | 'warning' | 'danger';
  title: string;
  message: string;
  action: InsightAction | null;
  actionLabel?: string;
}

let nextToastId = 0;
let branchRequestId = 0;

export const useRepoStore = defineStore('repo', () => {
  const config = ref<AppConfig>({ scanPaths: [], groups: [] });
  const repositories = ref<Repository[]>([]);
  const selectedGroupId = ref<string>('all');
  const isLoading = ref(false);
  const isScanning = ref(false);
  const isBranchListLoading = ref(false);
  const groupBranches = ref<BranchInfo[]>([]);
  const toasts = ref<Toast[]>([]);
  const scanModalOpen = ref(false);
  const saveModalOpen = ref(false);
  const saveModalPaths = ref<string[] | null>(null);
  const helpModalOpen = ref(false);
  const groupEditorOpen = ref(false);
  const editingGroup = ref<RepoGroup | null>(null);

  const currentGroup = computed(() => {
    if (selectedGroupId.value === 'all') return null;
    return config.value.groups.find((group) => group.id === selectedGroupId.value) ?? null;
  });

  const filteredRepositories = computed(() => {
    if (selectedGroupId.value === 'all') return repositories.value;
    const group = currentGroup.value;
    if (!group) return [];
    return repositories.value.filter((repo) => group.repoPaths.includes(repo.path));
  });

  const commonBranches = computed(() => {
    const list = filteredRepositories.value;
    if (list.length === 0) return [];

    const counts: Record<string, number> = {};
    for (const repo of list) {
      const branch = repo.state.currentBranch;
      if (branch) counts[branch] = (counts[branch] ?? 0) + 1;
    }

    return Object.entries(counts)
      .map(([branch, count]) => ({
        branch,
        count,
        isSynchronized: count === list.length,
      }))
      .sort((a, b) => b.count - a.count || a.branch.localeCompare(b.branch));
  });

  const synchronizedBranch = computed(() => {
    const list = filteredRepositories.value;
    if (list.length === 0) return null;
    const [first] = commonBranches.value;
    if (!first || first.count !== list.length) return null;
    return first.branch;
  });

  const cleanRepositoryCount = computed(
    () => filteredRepositories.value.filter((repo) => repo.state.isClean).length,
  );

  const dirtyRepositories = computed(() =>
    filteredRepositories.value.filter((repo) => !repo.state.isClean),
  );

  const conflictedRepositories = computed(() =>
    filteredRepositories.value.filter((repo) => repo.state.hasConflicts),
  );

  const insight = computed<GroupInsight | null>(() => {
    const list = filteredRepositories.value;
    if (list.length === 0) return null;

    const count = (predicate: (repo: Repository) => boolean): number =>
      list.filter(predicate).length;
    const projectWord = (value: number): string => (value === 1 ? 'project' : 'projects');

    const missing = count((repo) => !repo.isAccessible);
    if (missing > 0) {
      return {
        tone: 'danger',
        title: `${missing} ${projectWord(missing)} cannot be found`,
        message:
          'The folder may have been moved or renamed. Update the project folder or remove it from the scan list.',
        action: null,
      };
    }

    const conflicts = count((repo) => repo.state.hasConflicts);
    if (conflicts > 0) {
      return {
        tone: 'danger',
        title: `${conflicts} ${projectWord(conflicts)} need attention`,
        message:
          'Some files have conflicts. Open the project in your editor, fix the marked files, then click "Save to Git".',
        action: 'save',
        actionLabel: 'Review changes',
      };
    }

    const busy = count((repo) => repo.state.operationState !== 'idle');
    if (busy > 0) {
      return {
        tone: 'warning',
        title: 'A Git operation is still in progress',
        message:
          'Finish or cancel the merge, rebase, or cherry-pick in your editor before using the buttons here.',
        action: null,
      };
    }

    const detached = count((repo) => repo.state.isDetachedHead);
    if (detached > 0) {
      return {
        tone: 'warning',
        title: 'You are not on a branch',
        message:
          'Switch to a branch below before saving changes. Otherwise new work can be hard to find later.',
        action: null,
      };
    }

    const dirty = count((repo) => !repo.state.isClean);
    if (dirty > 0) {
      return {
        tone: 'warning',
        title:
          dirty === 1
            ? 'You have changes that are not saved to Git'
            : `${dirty} projects have changes that are not saved to Git`,
        message:
          'Save them to Git to create a checkpoint. Your files stay where they are, and you can keep working afterwards.',
        action: 'save',
        actionLabel: dirty === 1 ? 'Save to Git' : 'Save all to Git',
      };
    }

    const behind = count((repo) => repo.state.behindCount > 0);
    if (behind > 0) {
      return {
        tone: 'info',
        title: behind === 1 ? 'An update is available' : 'Updates are available',
        message: `${behind} ${projectWord(
          behind,
        )} can receive new work from the server. Get the latest changes before you start editing.`,
        action: 'pull',
        actionLabel: 'Get latest',
      };
    }

    const ahead = count((repo) => repo.state.aheadCount > 0);
    if (ahead > 0) {
      return {
        tone: 'info',
        title: 'You have work ready to send',
        message: `${ahead} ${projectWord(
          ahead,
        )} have saved changes that are not on the server yet. Send them when you are ready.`,
        action: 'push',
        actionLabel: 'Send changes',
      };
    }

    const withoutRemote = count((repo) => repo.state.remotes.length === 0);
    if (withoutRemote === list.length) {
      return {
        tone: 'success',
        title: 'Your work is saved on this computer',
        message:
          'This project is not connected to a server, so there is nothing to send or receive.',
        action: null,
      };
    }

    return {
      tone: 'success',
      title: 'Everything is up to date',
      message: 'No unsaved changes and no updates waiting.',
      action: null,
    };
  });

  function showToast(kind: ToastKind, title: string, message?: string): void {
    const id = ++nextToastId;
    toasts.value.push({ id, kind, title, message });
    window.setTimeout(() => dismissToast(id), kind === 'error' ? 9000 : 5000);
  }

  function dismissToast(id: number): void {
    toasts.value = toasts.value.filter((toast) => toast.id !== id);
  }

  function repoLabel(path: string): string {
    const parts = path.split(/[\\/]/).filter(Boolean);
    return parts[parts.length - 1] ?? path;
  }

  function summarizeResults(results: OperationResult[], verb: string): void {
    if (results.length === 0) return;

    const failures = results.filter((result) => !result.success);
    if (failures.length === 0) {
      showToast(
        'success',
        `${verb} complete`,
        `${results.length} ${results.length === 1 ? 'project' : 'projects'} updated.`,
      );
      return;
    }

    const details = failures
      .slice(0, 4)
      .map((failure) => {
        const firstLine = (failure.error ?? 'Unknown error').trim().split('\n')[0];
        return `${repoLabel(failure.repoPath)}: ${firstLine}`;
      })
      .join('\n');
    const suffix = failures.length > 4 ? `\n…and ${failures.length - 4} more.` : '';

    showToast(
      'error',
      `${verb} failed in ${failures.length}/${results.length}`,
      details + suffix,
    );
  }

  async function loadConfig(): Promise<void> {
    try {
      config.value = await backend.getConfig();
    } catch (error) {
      showToast('error', 'Could not load configuration', String(error));
    }
  }

  async function scanRepositories(): Promise<void> {
    isScanning.value = true;
    try {
      const paths = config.value.scanPaths.filter((scanPath) => scanPath.enabled);
      if (paths.length === 0) {
        repositories.value = [];
        groupBranches.value = [];
        return;
      }

      const maxDepth = Math.max(...paths.map((scanPath) => scanPath.maxDepth), 3);
      const found = await backend.discoverRepositories(
        paths.map((scanPath) => scanPath.path),
        maxDepth,
      );
      repositories.value = found;
      await loadGroupBranches();
    } catch (error) {
      showToast('error', 'Could not look for projects', String(error));
    } finally {
      isScanning.value = false;
    }
  }

  async function refreshSingleRepo(path: string): Promise<void> {
    const current = repositories.value.find((repo) => repo.path === path);
    const groupIds = current?.groupIds ?? [];

    try {
      const updated = await backend.refreshRepository(path, groupIds);
      const index = repositories.value.findIndex((repo) => repo.path === path);
      if (index !== -1) {
        repositories.value[index] = updated;
      } else {
        repositories.value.push(updated);
        repositories.value.sort((a, b) => a.path.localeCompare(b.path));
      }
    } catch {
      // A repository can disappear between a filesystem event and the refresh.
    }
  }

  async function loadGroupBranches(): Promise<void> {
    const paths = filteredRepositories.value.map((repo) => repo.path);
    const requestId = ++branchRequestId;

    if (paths.length === 0) {
      groupBranches.value = [];
      return;
    }

    isBranchListLoading.value = true;
    try {
      const branches = await backend.listGroupBranches(paths);
      if (requestId === branchRequestId) {
        groupBranches.value = branches;
      }
    } catch (error) {
      if (requestId === branchRequestId) {
        groupBranches.value = [];
        showToast('error', 'Could not list branches', String(error));
      }
    } finally {
      if (requestId === branchRequestId) {
        isBranchListLoading.value = false;
      }
    }
  }

  async function executeGroupAction(
    action: 'fetch' | 'pull' | 'push' | 'stash' | 'stash_pop',
  ): Promise<void> {
    const paths = filteredRepositories.value.map((repo) => repo.path);
    if (paths.length === 0) return;

    const actionLabel: Record<typeof action, string> = {
      fetch: 'Check for updates',
      pull: 'Get latest',
      push: 'Send changes',
      stash: 'Set aside changes',
      stash_pop: 'Bring back set-aside changes',
    };

    isLoading.value = true;
    try {
      const results = await backend.executeGroupAction(paths, action, 5);
      await Promise.all(paths.map((path) => refreshSingleRepo(path)));
      await loadGroupBranches();
      summarizeResults(results, actionLabel[action]);
    } catch (error) {
      showToast('error', `${actionLabel[action]} failed`, String(error));
    } finally {
      isLoading.value = false;
    }
  }

  async function commitRepositories(
    paths: string[],
    message: string,
  ): Promise<OperationResult[]> {
    const results: OperationResult[] = [];

    for (const path of paths) {
      try {
        results.push(await backend.commitRepoChanges(path, message));
      } catch (error) {
        results.push({
          repoPath: path,
          success: false,
          output: '',
          error: String(error),
        });
      }
    }

    await Promise.all(paths.map((path) => refreshSingleRepo(path)));
    await loadGroupBranches();
    return results;
  }

  async function switchGroupBranch(branch: string, autoStash = true): Promise<void> {
    const paths = filteredRepositories.value.map((repo) => repo.path);
    if (paths.length === 0) return;

    isLoading.value = true;
    try {
      const results = await backend.switchGroupBranch(paths, branch, autoStash);
      await Promise.all(paths.map((path) => refreshSingleRepo(path)));
      await loadGroupBranches();
      summarizeResults(results, `Switch to ${branch}`);
    } catch (error) {
      showToast('error', `Switch to ${branch} failed`, String(error));
    } finally {
      isLoading.value = false;
    }
  }

  async function saveGroup(group: RepoGroup): Promise<void> {
    try {
      config.value = await backend.saveRepoGroup(group);
      selectedGroupId.value = group.id;
      await loadGroupBranches();
      showToast('success', 'Group saved', group.name);
    } catch (error) {
      showToast('error', 'Could not save group', String(error));
    }
  }

  async function deleteGroup(groupId: string): Promise<void> {
    try {
      config.value = await backend.deleteRepoGroup(groupId);
      if (selectedGroupId.value === groupId) {
        selectedGroupId.value = 'all';
      }
      await loadGroupBranches();
      showToast('success', 'Group deleted');
    } catch (error) {
      showToast('error', 'Could not delete group', String(error));
    }
  }

  async function addScanPath(scanPath: ScanPath): Promise<boolean> {
    try {
      config.value = await backend.addScanPath(scanPath);
      showToast('success', 'Project folder added', scanPath.path);
      await scanRepositories();
      return true;
    } catch (error) {
      showToast('error', 'Could not add project folder', String(error));
      return false;
    }
  }

  async function removeScanPath(id: string): Promise<void> {
    try {
      config.value = await backend.removeScanPath(id);
      await scanRepositories();
      showToast('success', 'Project folder removed');
    } catch (error) {
      showToast('error', 'Could not remove project folder', String(error));
    }
  }

  async function toggleScanPath(scanPath: ScanPath): Promise<void> {
    await addScanPath({ ...scanPath, enabled: !scanPath.enabled });
  }

  function selectGroup(groupId: string): void {
    selectedGroupId.value = groupId;
    void loadGroupBranches();
  }

  function openAddScanPath(): void {
    scanModalOpen.value = true;
  }

  function closeScanPathModal(): void {
    scanModalOpen.value = false;
  }

  function openSaveChanges(paths?: string[]): void {
    saveModalPaths.value = paths ?? null;
    saveModalOpen.value = true;
  }

  function closeSaveChanges(): void {
    saveModalOpen.value = false;
    saveModalPaths.value = null;
  }

  function openHelp(): void {
    helpModalOpen.value = true;
  }

  function closeHelp(): void {
    helpModalOpen.value = false;
  }

  function openGroupEditor(group: RepoGroup | null): void {
    editingGroup.value = group;
    groupEditorOpen.value = true;
  }

  function closeGroupEditor(): void {
    groupEditorOpen.value = false;
    editingGroup.value = null;
  }

  return {
    config,
    repositories,
    selectedGroupId,
    currentGroup,
    filteredRepositories,
    commonBranches,
    synchronizedBranch,
    cleanRepositoryCount,
    dirtyRepositories,
    conflictedRepositories,
    insight,
    groupBranches,
    isLoading,
    isScanning,
    isBranchListLoading,
    toasts,
    scanModalOpen,
    saveModalOpen,
    saveModalPaths,
    helpModalOpen,
    groupEditorOpen,
    editingGroup,
    loadConfig,
    scanRepositories,
    refreshSingleRepo,
    loadGroupBranches,
    executeGroupAction,
    commitRepositories,
    switchGroupBranch,
    saveGroup,
    deleteGroup,
    addScanPath,
    removeScanPath,
    toggleScanPath,
    selectGroup,
    openAddScanPath,
    closeScanPathModal,
    openSaveChanges,
    closeSaveChanges,
    openHelp,
    closeHelp,
    openGroupEditor,
    closeGroupEditor,
    showToast,
    dismissToast,
  };
});
