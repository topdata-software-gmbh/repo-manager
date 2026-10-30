<script setup lang="ts">
import { ref } from 'vue';
import {
  AlertCircle,
  ArrowDownToLine,
  GitBranch,
  GitMerge,
  Loader2,
  RefreshCw,
  Save,
  Upload,
} from 'lucide-vue-next';
import { useRepoStore } from '../stores/repoStore';
import { branchLabel, statusBadges } from '../lib/repoStatus';
import type { Repository } from '../types/domain';

const props = withDefaults(
  defineProps<{
    repos: Repository[];
    actions?: 'group' | 'browse';
  }>(),
  { actions: 'group' },
);

const store = useRepoStore();
const refreshing = ref<string | null>(null);
const pendingAction = ref<{ path: string; action: 'pull' | 'push' } | null>(null);

async function refresh(path: string): Promise<void> {
  if (refreshing.value) return;
  refreshing.value = path;
  try {
    await store.refreshSingleRepo(path);
  } finally {
    refreshing.value = null;
  }
}

async function runAction(repo: Repository, action: 'pull' | 'push'): Promise<void> {
  if (pendingAction.value || refreshing.value) return;
  pendingAction.value = { path: repo.path, action };
  try {
    await store.runRepoAction(repo.path, action);
  } finally {
    pendingAction.value = null;
  }
}

function isRowBusy(repo: Repository): boolean {
  return refreshing.value === repo.path || pendingAction.value?.path === repo.path;
}
</script>

<template>
  <div class="min-h-0 flex-1 overflow-auto">
    <table class="w-full min-w-[780px] border-collapse text-left">
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
          v-for="repo in props.repos"
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
                :class="repo.state.isDetachedHead ? 'text-amber-300' : 'text-cyan-300'"
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
                <AlertCircle v-if="badge.label === 'Needs attention'" class="h-3 w-3" />
                <GitMerge v-else-if="badge.label.includes('in progress')" class="h-3 w-3" />
                {{ badge.label }}
              </span>
            </div>
          </td>

          <td class="px-4 py-3">
            <div v-if="repo.state.headCommitHash" class="min-w-0">
              <p
                class="max-w-md truncate text-xs text-slate-300"
                :title="repo.state.headCommitMessage"
              >
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

              <template v-if="props.actions === 'browse'">
                <button
                  class="rounded p-1.5 text-emerald-400 transition-colors hover:bg-slate-700 disabled:opacity-50"
                  :disabled="!repo.isAccessible || store.isLoading || isRowBusy(repo)"
                  :aria-label="`Get latest for ${repo.name}`"
                  title="Get latest for this project"
                  @click="runAction(repo, 'pull')"
                >
                  <Loader2
                    v-if="pendingAction?.path === repo.path && pendingAction.action === 'pull'"
                    class="h-3.5 w-3.5 animate-spin"
                  />
                  <ArrowDownToLine v-else class="h-3.5 w-3.5" />
                </button>
                <button
                  class="rounded p-1.5 text-sky-400 transition-colors hover:bg-slate-700 disabled:opacity-50"
                  :disabled="!repo.isAccessible || store.isLoading || isRowBusy(repo)"
                  :aria-label="`Send changes for ${repo.name}`"
                  title="Send changes for this project"
                  @click="runAction(repo, 'push')"
                >
                  <Loader2
                    v-if="pendingAction?.path === repo.path && pendingAction.action === 'push'"
                    class="h-3.5 w-3.5 animate-spin"
                  />
                  <Upload v-else class="h-3.5 w-3.5" />
                </button>
              </template>

              <button
                class="rounded p-1.5 text-slate-500 transition-colors hover:bg-slate-700 hover:text-slate-200 disabled:opacity-50"
                :disabled="isRowBusy(repo)"
                :aria-label="`Refresh ${repo.name}`"
                title="Refresh this project"
                @click="refresh(repo.path)"
              >
                <Loader2 v-if="refreshing === repo.path" class="h-3.5 w-3.5 animate-spin" />
                <RefreshCw v-else class="h-3.5 w-3.5" />
              </button>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
