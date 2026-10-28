<script setup lang="ts">
import { computed, ref } from 'vue';
import {
  AlertCircle,
  ArrowDown,
  ArrowUp,
  Check,
  FolderPlus,
  GitBranch,
  GitMerge,
  Loader2,
  RefreshCw,
  Search,
} from 'lucide-vue-next';
import { useRepoStore } from '../stores/repoStore';

const store = useRepoStore();
const search = ref('');
const refreshing = ref<string | null>(null);

const rows = computed(() => {
  const term = search.value.trim().toLowerCase();
  if (!term) return store.filteredRepositories;

  return store.filteredRepositories.filter((repo) =>
    `${repo.name} ${repo.path} ${repo.state.currentBranch}`.toLowerCase().includes(term),
  );
});

const hasScanPaths = computed(() => store.config.scanPaths.length > 0);

async function refresh(path: string): Promise<void> {
  if (refreshing.value) return;
  refreshing.value = path;
  try {
    await store.refreshSingleRepo(path);
  } finally {
    refreshing.value = null;
  }
}

function operationLabel(operation: string): string {
  switch (operation) {
    case 'merging':
      return 'Merging';
    case 'rebasing':
      return 'Rebasing';
    case 'cherry-picking':
      return 'Cherry-picking';
    default:
      return '';
  }
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col bg-slate-900">
    <div class="flex shrink-0 items-center justify-between border-b border-slate-800/70 px-4 py-2">
      <div class="relative w-72">
        <Search class="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-slate-500" />
        <input
          v-model="search"
          type="text"
          placeholder="Filter repositories…"
          class="w-full rounded-md border border-slate-800 bg-slate-950 py-1.5 pl-8 pr-3 text-xs text-slate-200 outline-none transition-colors placeholder:text-slate-600 focus:border-cyan-600"
        />
      </div>
      <span class="font-mono text-[11px] text-slate-500">
        {{ rows.length }} of {{ store.filteredRepositories.length }} shown
      </span>
    </div>

    <div class="min-h-0 flex-1 overflow-auto">
      <table v-if="rows.length > 0" class="w-full min-w-[900px] border-collapse text-left text-xs">
        <thead
          class="sticky top-0 z-10 border-b border-slate-800 bg-slate-950/95 text-[10px] font-semibold uppercase tracking-wider text-slate-400 backdrop-blur"
        >
          <tr>
            <th class="px-4 py-2.5">Repository</th>
            <th class="px-4 py-2.5">Current Branch</th>
            <th class="px-4 py-2.5">Sync Status</th>
            <th class="px-4 py-2.5">Changes</th>
            <th class="px-4 py-2.5">Last Commit</th>
            <th class="w-12 px-2 py-2.5"><span class="sr-only">Actions</span></th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60 font-mono">
          <tr
            v-for="repo in rows"
            :key="repo.id"
            class="transition-colors hover:bg-slate-800/40"
            :class="{ 'opacity-60': !repo.isAccessible }"
          >
            <td class="px-4 py-2.5">
              <div class="flex items-center gap-2 font-sans font-medium text-slate-100">
                <span>{{ repo.name }}</span>
                <span
                  v-if="!repo.isAccessible"
                  class="rounded bg-rose-500/10 px-1.5 py-0.5 text-[10px] font-normal text-rose-400"
                >
                  missing
                </span>
              </div>
              <div class="max-w-xs truncate text-[11px] text-slate-500" :title="repo.path">
                {{ repo.path }}
              </div>
            </td>

            <td class="px-4 py-2.5">
              <div
                class="inline-flex items-center gap-1.5 rounded border border-slate-700/60 bg-slate-800 px-2 py-0.5 text-cyan-300"
                :title="
                  repo.state.isDetachedHead
                    ? 'Detached HEAD'
                    : repo.state.currentBranch
                "
              >
                <GitBranch class="h-3 w-3 shrink-0 text-cyan-400" />
                <span class="max-w-40 truncate">{{ repo.state.currentBranch || '—' }}</span>
              </div>
              <div
                v-if="repo.state.operationState !== 'idle'"
                class="mt-1 inline-flex items-center gap-1 rounded bg-amber-500/10 px-1.5 py-0.5 text-[10px] text-amber-400"
              >
                <GitMerge class="h-3 w-3" />
                <span>{{ operationLabel(repo.state.operationState) }}</span>
              </div>
            </td>

            <td class="px-4 py-2.5">
              <div class="flex items-center gap-3">
                <span
                  class="flex items-center gap-0.5 text-[11px]"
                  :class="repo.state.aheadCount > 0 ? 'font-bold text-emerald-400' : 'text-slate-600'"
                  title="Commits ahead of upstream"
                >
                  <ArrowUp class="h-3 w-3" />
                  <span>{{ repo.state.aheadCount }}</span>
                </span>
                <span
                  class="flex items-center gap-0.5 text-[11px]"
                  :class="repo.state.behindCount > 0 ? 'font-bold text-amber-400' : 'text-slate-600'"
                  title="Commits behind upstream"
                >
                  <ArrowDown class="h-3 w-3" />
                  <span>{{ repo.state.behindCount }}</span>
                </span>
              </div>
              <div
                v-if="repo.state.remotes.length === 0"
                class="mt-0.5 text-[10px] text-slate-600"
              >
                no remote
              </div>
            </td>

            <td class="px-4 py-2.5 font-sans">
              <span
                v-if="repo.state.isClean"
                class="inline-flex items-center gap-1 text-xs text-emerald-400"
              >
                <Check class="h-3.5 w-3.5" />
                <span>Clean</span>
              </span>
              <span
                v-else-if="repo.state.hasConflicts"
                class="inline-flex items-center gap-1 text-xs font-semibold text-rose-400"
              >
                <AlertCircle class="h-3.5 w-3.5" />
                <span>Conflict</span>
              </span>
              <div v-else class="flex flex-wrap items-center gap-x-2 gap-y-0.5 text-[11px]">
                <span v-if="repo.state.modifiedFiles > 0" class="text-amber-300">
                  {{ repo.state.modifiedFiles }} modified
                </span>
                <span v-if="repo.state.untrackedFiles > 0" class="text-slate-400">
                  {{ repo.state.untrackedFiles }} untracked
                </span>
                <span v-if="repo.state.stagedFiles > 0" class="text-cyan-300">
                  {{ repo.state.stagedFiles }} staged
                </span>
              </div>
            </td>

            <td class="px-4 py-2.5 font-sans text-slate-400">
              <div v-if="repo.state.headCommitHash" class="flex items-center gap-1.5">
                <span class="font-mono text-[11px] text-cyan-400">
                  {{ repo.state.headCommitHash }}
                </span>
                <span
                  class="max-w-sm truncate text-xs text-slate-300"
                  :title="repo.state.headCommitMessage"
                >
                  {{ repo.state.headCommitMessage }}
                </span>
              </div>
              <span v-else class="text-[11px] italic text-slate-600">No commits yet</span>
            </td>

            <td class="px-2 py-2.5">
              <button
                class="rounded p-1 text-slate-500 transition-colors hover:bg-slate-700 hover:text-slate-200 disabled:opacity-50"
                :disabled="refreshing === repo.path"
                title="Refresh this repository"
                @click="refresh(repo.path)"
              >
                <Loader2
                  v-if="refreshing === repo.path"
                  class="h-3.5 w-3.5 animate-spin"
                />
                <RefreshCw v-else class="h-3.5 w-3.5" />
              </button>
            </td>
          </tr>
        </tbody>
      </table>

      <div
        v-else-if="store.isScanning"
        class="space-y-2 p-6"
      >
        <div
          v-for="index in 5"
          :key="index"
          class="h-12 animate-pulse rounded-lg border border-slate-800 bg-slate-800/30"
        ></div>
      </div>

      <div
        v-else
        class="flex h-full min-h-72 flex-col items-center justify-center px-6 text-center"
      >
        <div class="mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-slate-800/70">
          <FolderPlus v-if="!hasScanPaths" class="h-5 w-5 text-cyan-400" />
          <GitBranch v-else class="h-5 w-5 text-slate-400" />
        </div>

        <template v-if="!hasScanPaths">
          <h2 class="text-sm font-semibold text-slate-200">No scan paths configured</h2>
          <p class="mt-1 max-w-sm text-xs leading-relaxed text-slate-500">
            Point the manager at a directory containing your Git repositories. Repositories are
            discovered automatically, including nested services.
          </p>
          <button
            class="mt-4 rounded-lg bg-cyan-500 px-3.5 py-2 text-xs font-semibold text-slate-950 transition-colors hover:bg-cyan-400"
            @click="store.openAddScanPath"
          >
            Add scan path
          </button>
        </template>

        <template v-else-if="store.selectedGroupId !== 'all' && store.filteredRepositories.length === 0">
          <h2 class="text-sm font-semibold text-slate-200">This group is empty</h2>
          <p class="mt-1 max-w-sm text-xs leading-relaxed text-slate-500">
            Assign repositories to
            <span class="text-slate-300">{{ store.currentGroup?.name }}</span>
            to run group-wide fetch, pull, push and branch switches.
          </p>
          <button
            class="mt-4 rounded-lg bg-cyan-500 px-3.5 py-2 text-xs font-semibold text-slate-950 transition-colors hover:bg-cyan-400"
            @click="store.openGroupEditor(store.currentGroup)"
          >
            Edit group
          </button>
        </template>

        <template v-else-if="store.filteredRepositories.length === 0">
          <h2 class="text-sm font-semibold text-slate-200">No repositories found</h2>
          <p class="mt-1 max-w-sm text-xs leading-relaxed text-slate-500">
            The configured directories do not contain a Git repository within the scan depth.
            Adjust the scan depth or add another path.
          </p>
          <div class="mt-4 flex gap-2">
            <button
              class="rounded-lg bg-slate-800 px-3.5 py-2 text-xs text-slate-200 transition-colors hover:bg-slate-700"
              @click="store.scanRepositories"
            >
              Rescan
            </button>
            <button
              class="rounded-lg bg-cyan-500 px-3.5 py-2 text-xs font-semibold text-slate-950 transition-colors hover:bg-cyan-400"
              @click="store.openAddScanPath"
            >
              Add scan path
            </button>
          </div>
        </template>

        <template v-else>
          <h2 class="text-sm font-semibold text-slate-200">No matching repositories</h2>
          <p class="mt-1 text-xs text-slate-500">Clear the filter to see all repositories.</p>
          <button
            class="mt-4 rounded-lg px-3 py-1.5 text-xs text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-200"
            @click="search = ''"
          >
            Clear filter
          </button>
        </template>
      </div>
    </div>
  </section>
</template>
