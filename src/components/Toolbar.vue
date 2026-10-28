<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import {
  Archive,
  ArrowDownToLine,
  GitBranch,
  Loader2,
  RefreshCw,
  Upload,
} from 'lucide-vue-next';
import { useRepoStore } from '../stores/repoStore';

const store = useRepoStore();
const selectedBranch = ref('');
const switching = ref(false);
const activeAction = ref<string | null>(null);

const busy = computed(() => store.isLoading || store.isScanning);
const canSwitchBranch = computed(
  () => !busy.value && !switching.value && store.groupBranches.length > 0,
);
const showBranchSelector = computed(
  () => !store.currentGroup || store.currentGroup.settings.syncBranching,
);

watch(
  () => store.selectedGroupId,
  () => {
    selectedBranch.value = '';
  },
);

watch(
  () => store.groupBranches,
  (branches) => {
    if (selectedBranch.value && !branches.some((branch) => branch.name === selectedBranch.value)) {
      selectedBranch.value = '';
    }
  },
);

async function onBranchChange(): Promise<void> {
  const branch = selectedBranch.value;
  if (!branch || switching.value) return;

  switching.value = true;
  selectedBranch.value = '';
  try {
    const autoStash = store.currentGroup?.settings.autoStashOnSwitch ?? true;
    await store.switchGroupBranch(branch, autoStash);
  } finally {
    switching.value = false;
  }
}

async function runAction(action: 'fetch' | 'pull' | 'push' | 'stash'): Promise<void> {
  if (activeAction.value || busy.value) return;
  activeAction.value = action;
  try {
    await store.executeGroupAction(action);
  } finally {
    activeAction.value = null;
  }
}
</script>

<template>
  <header
    class="flex h-14 shrink-0 select-none items-center justify-between border-b border-slate-800 bg-slate-900 px-4"
  >
    <div class="flex min-w-0 items-center gap-3">
      <span class="max-w-44 truncate text-xs font-semibold uppercase tracking-wider text-slate-400">
        {{ store.currentGroup ? store.currentGroup.name : 'All Repositories' }}
      </span>
      <span class="rounded-full bg-slate-800 px-2 py-0.5 font-mono text-xs text-slate-300">
        {{ store.filteredRepositories.length }} repos
      </span>
      <span
        v-if="store.filteredRepositories.length > 0"
        class="hidden items-center gap-1 text-[11px] text-emerald-400/90 xl:flex"
        title="Repositories without local changes"
      >
        <span class="h-1.5 w-1.5 rounded-full bg-emerald-400"></span>
        {{ store.cleanRepositoryCount }} clean
      </span>

      <div
        v-if="store.synchronizedBranch"
        class="hidden items-center gap-1.5 rounded border border-cyan-500/30 bg-cyan-500/10 px-2 py-0.5 text-[11px] text-cyan-300 2xl:flex"
        title="Every repository in this selection is on the same branch"
      >
        <GitBranch class="h-3 w-3" />
        <span class="font-mono">{{ store.synchronizedBranch }}</span>
      </div>
      <span
        v-else-if="store.commonBranches.length > 1"
        class="hidden font-mono text-[11px] text-amber-400/90 2xl:block"
        title="Repositories are spread across multiple branches"
      >
        {{ store.commonBranches.length }} branches
      </span>

      <div
        v-if="showBranchSelector"
        class="ml-2 flex items-center gap-1.5 rounded border border-slate-700 bg-slate-800/80 px-2 py-1"
        :title="
          store.currentGroup?.settings.autoStashOnSwitch === false
            ? 'Switch every repository to a local branch (auto-stash disabled)'
            : 'Switch every repository to a local branch (auto-stash enabled)'
        "
      >
        <Loader2 v-if="switching || store.isBranchListLoading" class="h-3.5 w-3.5 animate-spin text-cyan-400" />
        <GitBranch v-else class="h-3.5 w-3.5 text-cyan-400" />
        <select
          v-model="selectedBranch"
          :disabled="!canSwitchBranch"
          class="cursor-pointer bg-transparent pr-2 text-xs text-slate-200 outline-none disabled:cursor-not-allowed disabled:opacity-50"
          @change="onBranchChange"
        >
          <option value="" disabled>
            {{
              store.filteredRepositories.length === 0
                ? 'No repositories'
                : 'Switch group branch…'
            }}
          </option>
          <option
            v-for="branch in store.groupBranches"
            :key="branch.name"
            :value="branch.name"
          >
            {{ branch.name }} ({{ branch.repoCount }}/{{ store.filteredRepositories.length }})
          </option>
        </select>
      </div>
    </div>

    <div class="flex items-center gap-1.5">
      <button
        class="flex items-center gap-1 rounded-md bg-slate-800 px-2.5 py-1.5 text-xs text-slate-200 transition-colors hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="busy"
        title="Rescan configured paths"
        @click="store.scanRepositories"
      >
        <Loader2 v-if="store.isScanning" class="h-3.5 w-3.5 animate-spin" />
        <RefreshCw v-else class="h-3.5 w-3.5" />
      </button>

      <button
        class="flex items-center gap-1.5 rounded-md bg-slate-800 px-3 py-1.5 text-xs text-slate-200 transition-colors hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="busy || store.filteredRepositories.length === 0"
        title="Fetch all repositories in this selection"
        @click="runAction('fetch')"
      >
        <Loader2 v-if="activeAction === 'fetch'" class="h-3.5 w-3.5 animate-spin" />
        <RefreshCw v-else class="h-3.5 w-3.5" />
        <span>Fetch</span>
      </button>

      <button
        class="flex items-center gap-1.5 rounded-md bg-slate-800 px-3 py-1.5 text-xs text-slate-200 transition-colors hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="busy || store.filteredRepositories.length === 0"
        title="Fast-forward pull for all repositories in this selection"
        @click="runAction('pull')"
      >
        <Loader2 v-if="activeAction === 'pull'" class="h-3.5 w-3.5 animate-spin" />
        <ArrowDownToLine v-else class="h-3.5 w-3.5 text-emerald-400" />
        <span>Pull</span>
      </button>

      <button
        class="flex items-center gap-1.5 rounded-md bg-slate-800 px-3 py-1.5 text-xs text-slate-200 transition-colors hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="busy || store.filteredRepositories.length === 0"
        title="Push all repositories in this selection"
        @click="runAction('push')"
      >
        <Loader2 v-if="activeAction === 'push'" class="h-3.5 w-3.5 animate-spin" />
        <Upload v-else class="h-3.5 w-3.5 text-sky-400" />
        <span>Push</span>
      </button>

      <button
        class="flex items-center gap-1.5 rounded-md bg-slate-800 px-3 py-1.5 text-xs text-slate-200 transition-colors hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="busy || store.filteredRepositories.length === 0"
        title="Stash dirty changes in all repositories in this selection"
        @click="runAction('stash')"
      >
        <Loader2 v-if="activeAction === 'stash'" class="h-3.5 w-3.5 animate-spin" />
        <Archive v-else class="h-3.5 w-3.5 text-amber-400" />
        <span>Stash</span>
      </button>
    </div>
  </header>
</template>
