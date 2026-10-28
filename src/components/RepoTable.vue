<script setup lang="ts">
import { computed, ref } from 'vue';
import {
  AlertCircle,
  BookOpen,
  FolderPlus,
  GitBranch,
  GitMerge,
  Loader2,
  RefreshCw,
  Save,
  Search,
} from 'lucide-vue-next';
import { useRepoStore } from '../stores/repoStore';
import type { Repository } from '../types/domain';

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

interface StatusBadge {
  label: string;
  className: string;
}

function statusBadges(repo: Repository): StatusBadge[] {
  if (!repo.isAccessible) {
    return [{ label: 'Folder not found', className: 'bg-rose-500/15 text-rose-300' }];
  }

  const badges: StatusBadge[] = [];

  if (repo.state.hasConflicts) {
    badges.push({ label: 'Needs attention', className: 'bg-rose-500/15 text-rose-300' });
  }

  if (repo.state.operationState !== 'idle') {
    badges.push({
      label: operationLabel(repo.state.operationState),
      className: 'bg-amber-500/15 text-amber-300',
    });
  }

  if (!repo.state.isClean) {
    badges.push({ label: 'Changes not saved', className: 'bg-amber-500/10 text-amber-300' });
  }

  if (repo.state.behindCount > 0) {
    badges.push({
      label: `${repo.state.behindCount} ${repo.state.behindCount === 1 ? 'update' : 'updates'} available`,
      className: 'bg-sky-500/10 text-sky-300',
    });
  }

  if (repo.state.aheadCount > 0) {
    badges.push({
      label: `${repo.state.aheadCount} to send`,
      className: 'bg-emerald-500/10 text-emerald-300',
    });
  }

  if (repo.state.remotes.length === 0) {
    badges.push({ label: 'On this computer only', className: 'bg-slate-800 text-slate-400' });
  } else if (repo.state.isClean && repo.state.aheadCount === 0 && repo.state.behindCount === 0) {
    badges.push({ label: 'Up to date', className: 'bg-emerald-500/10 text-emerald-300' });
  }

  return badges;
}

function operationLabel(operation: string): string {
  switch (operation) {
    case 'merging':
      return 'Merge in progress';
    case 'rebasing':
      return 'Rebase in progress';
    case 'cherry-picking':
      return 'Cherry-pick in progress';
    default:
      return 'Git operation in progress';
  }
}

function branchLabel(repo: Repository): string {
  return repo.state.isDetachedHead ? 'Not on a branch' : repo.state.currentBranch || 'Unknown';
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col bg-slate-900">
    <div class="flex shrink-0 items-center justify-between gap-4 border-b border-slate-800/70 px-4 py-2.5">
      <div class="relative w-full max-w-sm">
        <Search
          class="pointer-events-none absolute left-3 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-slate-500"
        />
        <input
          v-model="search"
          type="text"
          placeholder="Search projects…"
          class="w-full rounded-lg border border-slate-800 bg-slate-950 py-1.5 pl-9 pr-3 text-xs text-slate-200 outline-none transition-colors placeholder:text-slate-600 focus:border-cyan-600"
        />
      </div>
      <span class="shrink-0 font-mono text-[11px] text-slate-500">
        {{ rows.length }} of {{ store.filteredRepositories.length }} shown
      </span>
    </div>

    <div class="min-h-0 flex-1 overflow-auto">
      <table v-if="rows.length > 0" class="w-full min-w-[780px] border-collapse text-left">
        <thead
          class="sticky top-0 z-10 border-b border-slate-800 bg-slate-950/95 text-[10px] font-semibold uppercase tracking-wider text-slate-400 backdrop-blur"
        >
          <tr>
            <th class="px-4 py-2.5">Project</th>
            <th class="px-4 py-2.5">Status</th>
            <th class="px-4 py-2.5">Latest checkpoint</th>
            <th class="w-36 px-3 py-2.5"><span class="sr-only">Actions</span></th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60">
          <tr
            v-for="repo in rows"
            :key="repo.id"
            class="transition-colors hover:bg-slate-800/40"
            :class="{ 'opacity-60': !repo.isAccessible }"
          >
            <td class="px-4 py-3">
              <div class="flex items-center gap-2 text-sm font-medium text-slate-100">
                <span class="truncate">{{ repo.name }}</span>
                <span
                  v-if="repo.state.isDetachedHead"
                  class="rounded bg-amber-500/10 px-1.5 py-0.5 text-[10px] font-normal text-amber-300"
                >
                  not on a branch
                </span>
              </div>
              <div class="mt-1 flex flex-wrap items-center gap-2">
                <span
                  class="inline-flex items-center gap-1 rounded border border-slate-700/60 bg-slate-800 px-1.5 py-0.5 font-mono text-[10px]"
                  :class="
                    repo.state.isDetachedHead
                      ? 'text-amber-300'
                      : 'text-cyan-300'
                  "
                  title="The branch (version) you are working on"
                >
                  <GitBranch class="h-3 w-3 shrink-0" />
                  {{ branchLabel(repo) }}
                </span>
                <span class="max-w-[28rem] truncate text-[10px] text-slate-500" :title="repo.path">
                  {{ repo.path }}
                </span>
              </div>
            </td>

            <td class="px-4 py-3">
              <div class="flex max-w-xs flex-wrap items-center gap-1.5">
                <span
                  v-for="badge in statusBadges(repo)"
                  :key="badge.label"
                  class="inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-medium"
                  :class="badge.className"
                >
                  <AlertCircle
                    v-if="badge.label === 'Needs attention'"
                    class="h-3 w-3"
                  />
                  <GitMerge
                    v-else-if="badge.label.includes('in progress')"
                    class="h-3 w-3"
                  />
                  {{ badge.label }}
                </span>
              </div>
            </td>

            <td class="px-4 py-3">
              <div v-if="repo.state.headCommitHash" class="min-w-0">
                <p class="max-w-md truncate text-xs text-slate-300" :title="repo.state.headCommitMessage">
                  {{ repo.state.headCommitMessage }}
                </p>
                <p class="mt-0.5 font-mono text-[10px] text-slate-500">
                  {{ repo.state.headCommitHash }}
                </p>
              </div>
              <span v-else class="text-[11px] italic text-slate-600">
                No checkpoints saved yet
              </span>
            </td>

            <td class="px-3 py-3">
              <div class="flex items-center justify-end gap-1.5">
                <button
                  v-if="!repo.state.isClean"
                  class="flex items-center gap-1 rounded-md bg-slate-800 px-2.5 py-1.5 text-[11px] text-slate-200 transition-colors hover:bg-slate-700 disabled:opacity-50"
                  :disabled="store.isLoading"
                  :title="
                    repo.state.hasConflicts
                      ? 'Review the changed files (resolve conflicts before saving)'
                      : 'Save changes in this project to Git'
                  "
                  @click="store.openSaveChanges([repo.path])"
                >
                  <Save class="h-3 w-3" />
                  <span>{{ repo.state.hasConflicts ? 'Review' : 'Save' }}</span>
                </button>
                <button
                  class="rounded p-1.5 text-slate-500 transition-colors hover:bg-slate-700 hover:text-slate-200 disabled:opacity-50"
                  :disabled="refreshing === repo.path"
                  title="Refresh this project"
                  @click="refresh(repo.path)"
                >
                  <Loader2
                    v-if="refreshing === repo.path"
                    class="h-3.5 w-3.5 animate-spin"
                  />
                  <RefreshCw v-else class="h-3.5 w-3.5" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>

      <div v-else-if="store.isScanning" class="space-y-2 p-6">
        <div
          v-for="index in 5"
          :key="index"
          class="h-14 animate-pulse rounded-lg border border-slate-800 bg-slate-800/30"
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
          <h2 class="text-base font-semibold text-slate-100">Let's find your projects</h2>
          <p class="mt-1.5 max-w-md text-sm leading-relaxed text-slate-400">
            Choose the folder where your themes and projects live. We will look inside it and add
            every Git project we find.
          </p>
          <div class="mt-4 flex items-center gap-2">
            <button
              class="rounded-lg bg-cyan-500 px-4 py-2 text-xs font-semibold text-slate-950 transition-colors hover:bg-cyan-400"
              @click="store.openAddScanPath"
            >
              Add project folder
            </button>
            <button
              class="flex items-center gap-1.5 rounded-lg px-3 py-2 text-xs text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-200"
              @click="store.openHelp"
            >
              <BookOpen class="h-3.5 w-3.5" />
              How this works
            </button>
          </div>
        </template>

        <template
          v-else-if="store.selectedGroupId !== 'all' && store.filteredRepositories.length === 0"
        >
          <h2 class="text-base font-semibold text-slate-100">No projects in this group</h2>
          <p class="mt-1.5 max-w-md text-sm leading-relaxed text-slate-400">
            Add projects to <span class="text-slate-200">{{ store.currentGroup?.name }}</span> to use
            the buttons above for the whole group.
          </p>
          <button
            class="mt-4 rounded-lg bg-cyan-500 px-4 py-2 text-xs font-semibold text-slate-950 transition-colors hover:bg-cyan-400"
            @click="store.openGroupEditor(store.currentGroup)"
          >
            Edit group
          </button>
        </template>

        <template v-else-if="store.filteredRepositories.length === 0">
          <h2 class="text-base font-semibold text-slate-100">No Git projects found</h2>
          <p class="mt-1.5 max-w-md text-sm leading-relaxed text-slate-400">
            We looked inside your folders but did not find a Git project. Check that the folder is
            correct, or add another folder.
          </p>
          <div class="mt-4 flex gap-2">
            <button
              class="rounded-lg bg-slate-800 px-4 py-2 text-xs text-slate-200 transition-colors hover:bg-slate-700"
              @click="store.scanRepositories"
            >
              Scan again
            </button>
            <button
              class="rounded-lg bg-cyan-500 px-4 py-2 text-xs font-semibold text-slate-950 transition-colors hover:bg-cyan-400"
              @click="store.openAddScanPath"
            >
              Add project folder
            </button>
          </div>
        </template>

        <template v-else>
          <h2 class="text-base font-semibold text-slate-100">No matching projects</h2>
          <p class="mt-1.5 text-sm text-slate-400">Try a different search, or clear the filter.</p>
          <button
            class="mt-4 rounded-lg px-3 py-1.5 text-xs text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-200"
            @click="search = ''"
          >
            Clear search
          </button>
        </template>
      </div>
    </div>
  </section>
</template>
