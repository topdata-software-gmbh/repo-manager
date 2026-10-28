<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import {
  Archive,
  ArchiveRestore,
  ArrowDownToLine,
  GitBranch,
  HelpCircle,
  Loader2,
  MoreHorizontal,
  RefreshCw,
  Save,
  Upload,
} from 'lucide-vue-next';
import { useRepoStore } from '../stores/repoStore';

const store = useRepoStore();
const selectedBranch = ref('');
const switching = ref(false);
const activeAction = ref<string | null>(null);
const moreOpen = ref(false);
const moreRef = ref<HTMLElement | null>(null);

const busy = computed(() => store.isLoading || store.isScanning);
const canSwitchBranch = computed(
  () => !busy.value && !switching.value && store.groupBranches.length > 0,
);
const showBranchSelector = computed(
  () => !store.currentGroup || store.currentGroup.settings.syncBranching,
);
const dirtyCount = computed(() => store.dirtyRepositories.length);
const branchSelection = computed({
  get: () => selectedBranch.value || store.synchronizedBranch || '',
  set: (value: string) => {
    selectedBranch.value = value;
  },
});

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

function handleOutsideClick(event: MouseEvent): void {
  if (!moreOpen.value) return;
  if (moreRef.value?.contains(event.target as Node)) return;
  moreOpen.value = false;
}

onMounted(() => document.addEventListener('click', handleOutsideClick));
onUnmounted(() => document.removeEventListener('click', handleOutsideClick));

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

async function runAction(
  action: 'fetch' | 'pull' | 'push' | 'stash' | 'stash_pop',
): Promise<void> {
  if (activeAction.value || busy.value) return;
  moreOpen.value = false;
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
    class="flex min-h-16 shrink-0 select-none flex-wrap items-center justify-between gap-3 border-b border-slate-800 bg-slate-900 px-4 py-2.5"
  >
    <div class="flex min-w-0 flex-wrap items-center gap-3">
      <span class="max-w-48 truncate text-sm font-semibold text-slate-100">
        {{ store.currentGroup ? store.currentGroup.name : 'All projects' }}
      </span>
      <span class="rounded-full bg-slate-800 px-2 py-0.5 font-mono text-[11px] text-slate-300">
        {{ store.filteredRepositories.length }}
        {{ store.filteredRepositories.length === 1 ? 'project' : 'projects' }}
      </span>
      <span
        v-if="store.filteredRepositories.length > 0"
        class="hidden items-center gap-1 text-[11px] text-emerald-400/90 lg:flex"
        title="Projects with no changes waiting"
      >
        <span class="h-1.5 w-1.5 rounded-full bg-emerald-400"></span>
        {{ store.cleanRepositoryCount }} up to date
      </span>

      <div
        v-if="showBranchSelector"
        class="flex items-center gap-2 rounded-lg border border-slate-700 bg-slate-800/70 px-2.5 py-1.5"
        title="Choose which version of the project you want to work on"
      >
        <Loader2
          v-if="switching || store.isBranchListLoading"
          class="h-3.5 w-3.5 animate-spin text-cyan-400"
        />
        <GitBranch v-else class="h-3.5 w-3.5 text-cyan-400" />
        <span class="text-[11px] font-medium text-slate-400">Working on</span>
        <select
          v-model="branchSelection"
          :disabled="!canSwitchBranch"
          class="max-w-44 cursor-pointer bg-transparent font-mono text-xs text-slate-100 outline-none disabled:cursor-not-allowed disabled:opacity-50"
          @change="onBranchChange"
        >
          <option value="" disabled>
            {{
              store.filteredRepositories.length === 0
                ? 'No projects yet'
                : 'Choose a branch…'
            }}
          </option>
          <option v-for="branch in store.groupBranches" :key="branch.name" :value="branch.name">
            {{ branch.name }} ({{ branch.repoCount }}/{{ store.filteredRepositories.length }})
          </option>
        </select>
      </div>
    </div>

    <div class="flex flex-wrap items-center gap-1.5">
      <button
        class="flex items-center gap-1.5 rounded-md bg-cyan-500 px-3 py-1.5 text-xs font-semibold text-slate-950 transition-colors hover:bg-cyan-400 disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="busy || dirtyCount === 0"
        title="Create a checkpoint of your changed files in Git"
        @click="store.openSaveChanges()"
      >
        <Save class="h-3.5 w-3.5" />
        <span>Save to Git</span>
        <span v-if="dirtyCount > 0" class="rounded-full bg-slate-950/15 px-1.5 text-[10px]">
          {{ dirtyCount }}
        </span>
      </button>

      <button
        class="flex items-center gap-1.5 rounded-md bg-slate-800 px-3 py-1.5 text-xs text-slate-200 transition-colors hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="busy || store.filteredRepositories.length === 0"
        title="Download new work from the server and update your project folders"
        @click="runAction('pull')"
      >
        <Loader2 v-if="activeAction === 'pull'" class="h-3.5 w-3.5 animate-spin" />
        <ArrowDownToLine v-else class="h-3.5 w-3.5 text-emerald-400" />
        <span>Get latest</span>
      </button>

      <button
        class="flex items-center gap-1.5 rounded-md bg-slate-800 px-3 py-1.5 text-xs text-slate-200 transition-colors hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="busy || store.filteredRepositories.length === 0"
        title="Upload your saved checkpoints to the server"
        @click="runAction('push')"
      >
        <Loader2 v-if="activeAction === 'push'" class="h-3.5 w-3.5 animate-spin" />
        <Upload v-else class="h-3.5 w-3.5 text-sky-400" />
        <span>Send changes</span>
      </button>

      <button
        class="rounded-md p-1.5 text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-200 disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="busy"
        title="Rescan your project folders"
        @click="store.scanRepositories"
      >
        <Loader2 v-if="store.isScanning" class="h-3.5 w-3.5 animate-spin" />
        <RefreshCw v-else class="h-3.5 w-3.5" />
      </button>

      <div ref="moreRef" class="relative">
        <button
          class="rounded-md p-1.5 text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-200"
          title="More Git actions"
          @click="moreOpen = !moreOpen"
        >
          <MoreHorizontal class="h-4 w-4" />
        </button>

        <div
          v-if="moreOpen"
          class="absolute right-0 top-9 z-30 w-64 overflow-hidden rounded-lg border border-slate-700 bg-slate-900 py-1 shadow-xl"
        >
          <button
            class="flex w-full items-start gap-2.5 px-3 py-2 text-left transition-colors hover:bg-slate-800 disabled:opacity-50"
            :disabled="busy || store.filteredRepositories.length === 0"
            @click="runAction('fetch')"
          >
            <RefreshCw class="mt-0.5 h-3.5 w-3.5 shrink-0 text-slate-400" />
            <span>
              <span class="block text-xs text-slate-200">Check for updates</span>
              <span class="block text-[10px] text-slate-500">
                Looks for new work without changing your files.
              </span>
            </span>
          </button>
          <button
            class="flex w-full items-start gap-2.5 px-3 py-2 text-left transition-colors hover:bg-slate-800 disabled:opacity-50"
            :disabled="busy || store.filteredRepositories.length === 0"
            @click="runAction('stash')"
          >
            <Archive class="mt-0.5 h-3.5 w-3.5 shrink-0 text-slate-400" />
            <span>
              <span class="block text-xs text-slate-200">Set aside changes</span>
              <span class="block text-[10px] text-slate-500">
                Temporarily store unfinished work in all projects.
              </span>
            </span>
          </button>
          <button
            class="flex w-full items-start gap-2.5 px-3 py-2 text-left transition-colors hover:bg-slate-800 disabled:opacity-50"
            :disabled="busy || store.filteredRepositories.length === 0"
            @click="runAction('stash_pop')"
          >
            <ArchiveRestore class="mt-0.5 h-3.5 w-3.5 shrink-0 text-slate-400" />
            <span>
              <span class="block text-xs text-slate-200">Bring back set-aside changes</span>
              <span class="block text-[10px] text-slate-500">
                Restore the last set of changes in all projects.
              </span>
            </span>
          </button>
        </div>
      </div>

      <button
        class="rounded-md p-1.5 text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-200"
        title="How this works"
        @click="store.openHelp"
      >
        <HelpCircle class="h-4 w-4" />
      </button>
    </div>
  </header>
</template>
