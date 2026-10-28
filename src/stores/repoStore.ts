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
        `${results.length} ${results.length === 1 ? 'repository' : 'repositories'} updated.`,
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
      showToast('error', 'Repository scan failed', String(error));
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

    isLoading.value = true;
    try {
      const results = await backend.executeGroupAction(paths, action, 5);
      await Promise.all(paths.map((path) => refreshSingleRepo(path)));
      await loadGroupBranches();
      summarizeResults(results, action === 'stash_pop' ? 'Stash pop' : capitalize(action));
    } catch (error) {
      showToast('error', `${capitalize(action)} failed`, String(error));
    } finally {
      isLoading.value = false;
    }
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

  async function addScanPath(scanPath: ScanPath): Promise<void> {
    try {
      config.value = await backend.addScanPath(scanPath);
      showToast('success', 'Scan path added', scanPath.path);
      await scanRepositories();
    } catch (error) {
      showToast('error', 'Could not add scan path', String(error));
    }
  }

  async function removeScanPath(id: string): Promise<void> {
    try {
      config.value = await backend.removeScanPath(id);
      await scanRepositories();
      showToast('success', 'Scan path removed');
    } catch (error) {
      showToast('error', 'Could not remove scan path', String(error));
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

  function openGroupEditor(group: RepoGroup | null): void {
    editingGroup.value = group;
    groupEditorOpen.value = true;
  }

  function closeGroupEditor(): void {
    groupEditorOpen.value = false;
    editingGroup.value = null;
  }

  function capitalize(value: string): string {
    return value.charAt(0).toUpperCase() + value.slice(1);
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
    groupBranches,
    isLoading,
    isScanning,
    isBranchListLoading,
    toasts,
    scanModalOpen,
    groupEditorOpen,
    editingGroup,
    loadConfig,
    scanRepositories,
    refreshSingleRepo,
    loadGroupBranches,
    executeGroupAction,
    switchGroupBranch,
    saveGroup,
    deleteGroup,
    addScanPath,
    removeScanPath,
    toggleScanPath,
    selectGroup,
    openAddScanPath,
    closeScanPathModal,
    openGroupEditor,
    closeGroupEditor,
    showToast,
    dismissToast,
  };
});
