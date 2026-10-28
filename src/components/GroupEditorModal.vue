<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { Layers, Loader2, Trash2, X } from 'lucide-vue-next';
import { useRepoStore } from '../stores/repoStore';
import type { RepoGroup } from '../types/domain';

const store = useRepoStore();

const name = ref('');
const color = ref('#38bdf8');
const repoPaths = ref<string[]>([]);
const syncBranching = ref(true);
const autoStashOnSwitch = ref(true);
const defaultBaseBranch = ref('main');
const submitting = ref(false);
const deleting = ref(false);
const error = ref('');

const palette = ['#38bdf8', '#22c55e', '#f59e0b', '#f43f5e', '#a855f7', '#14b8a6'];

const isEditing = computed(() => store.editingGroup !== null);
const allSelected = computed(
  () =>
    store.repositories.length > 0 &&
    store.repositories.every((repo) => repoPaths.value.includes(repo.path)),
);

watch(
  () => store.groupEditorOpen,
  (open) => {
    if (!open) return;

    const group = store.editingGroup;
    name.value = group?.name ?? '';
    color.value = group?.color ?? '#38bdf8';
    repoPaths.value = group ? [...group.repoPaths] : [];
    syncBranching.value = group?.settings.syncBranching ?? true;
    autoStashOnSwitch.value = group?.settings.autoStashOnSwitch ?? true;
    defaultBaseBranch.value = group?.settings.defaultBaseBranch ?? 'main';
    submitting.value = false;
    deleting.value = false;
    error.value = '';
  },
);

function newId(): string {
  return `grp_${typeof crypto.randomUUID === 'function' ? crypto.randomUUID() : Date.now()}`;
}

function toggleRepository(path: string): void {
  if (repoPaths.value.includes(path)) {
    repoPaths.value = repoPaths.value.filter((existing) => existing !== path);
  } else {
    repoPaths.value.push(path);
  }
}

function toggleAll(): void {
  repoPaths.value = allSelected.value
    ? []
    : store.repositories.map((repo) => repo.path);
}

async function submit(): Promise<void> {
  const trimmed = name.value.trim();
  if (!trimmed) {
    error.value = 'Give the group a name.';
    return;
  }
  if (submitting.value) return;

  error.value = '';
  submitting.value = true;
  const group: RepoGroup = {
    id: store.editingGroup?.id ?? newId(),
    name: trimmed,
    color: color.value,
    repoPaths: [...repoPaths.value],
    settings: {
      syncBranching: syncBranching.value,
      autoStashOnSwitch: autoStashOnSwitch.value,
      defaultBaseBranch: defaultBaseBranch.value.trim() || 'main',
    },
  };

  try {
    await store.saveGroup(group);
    store.closeGroupEditor();
  } finally {
    submitting.value = false;
  }
}

async function remove(): Promise<void> {
  const group = store.editingGroup;
  if (!group || deleting.value) return;
  if (!window.confirm(`Delete group "${group.name}"? Repositories are not touched.`)) return;

  deleting.value = true;
  try {
    await store.deleteGroup(group.id);
    store.closeGroupEditor();
  } finally {
    deleting.value = false;
  }
}
</script>

<template>
  <div
    v-if="store.groupEditorOpen"
    class="fixed inset-0 z-40 flex items-center justify-center bg-slate-950/70 p-6 backdrop-blur-sm"
    @click.self="store.closeGroupEditor"
  >
    <form
      class="flex max-h-[85vh] w-full max-w-2xl flex-col rounded-xl border border-slate-700 bg-slate-900 shadow-2xl"
      @submit.prevent="submit"
    >
      <header class="flex items-center justify-between border-b border-slate-800 px-5 py-4">
        <div class="flex items-center gap-2">
          <Layers class="h-4 w-4 text-cyan-400" />
          <h2 class="text-sm font-semibold text-slate-100">
            {{ isEditing ? 'Edit group' : 'New group' }}
          </h2>
        </div>
        <button
          type="button"
          class="rounded p-1 text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-200"
          @click="store.closeGroupEditor"
        >
          <X class="h-4 w-4" />
        </button>
      </header>

      <div class="flex-1 space-y-4 overflow-auto px-5 py-4">
        <div class="grid grid-cols-[1fr_auto] gap-4">
          <div>
            <label for="group-name" class="mb-1.5 block text-xs font-medium text-slate-300">
              Group name
            </label>
            <input
              id="group-name"
              v-model="name"
              type="text"
              autofocus
              placeholder="Shop microservices"
              class="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-xs text-slate-100 outline-none transition-colors placeholder:text-slate-600 focus:border-cyan-500"
            />
          </div>

          <div>
            <span class="mb-1.5 block text-xs font-medium text-slate-300">Color</span>
            <div class="flex items-center gap-1.5 pt-1">
              <button
                v-for="swatch in palette"
                :key="swatch"
                type="button"
                class="h-5 w-5 rounded-full border-2 transition-transform hover:scale-110"
                :class="color === swatch ? 'border-slate-100' : 'border-transparent'"
                :style="{ backgroundColor: swatch }"
                :title="swatch"
                @click="color = swatch"
              />
            </div>
          </div>
        </div>

        <div>
          <div class="mb-1.5 flex items-center justify-between">
            <span class="text-xs font-medium text-slate-300">Repositories</span>
            <button
              v-if="store.repositories.length > 0"
              type="button"
              class="text-[11px] text-cyan-400 hover:text-cyan-300"
              @click="toggleAll"
            >
              {{ allSelected ? 'Clear all' : 'Select all' }}
            </button>
          </div>

          <div
            v-if="store.repositories.length === 0"
            class="rounded-lg border border-dashed border-slate-700 px-4 py-6 text-center text-[11px] text-slate-500"
          >
            No repositories discovered yet. Add a scan path and rescan first.
          </div>
          <div
            v-else
            class="max-h-56 divide-y divide-slate-800 overflow-auto rounded-lg border border-slate-700 bg-slate-950"
          >
            <label
              v-for="repo in store.repositories"
              :key="repo.id"
              class="flex cursor-pointer items-center gap-3 px-3 py-2 transition-colors hover:bg-slate-900"
            >
              <input
                type="checkbox"
                :checked="repoPaths.includes(repo.path)"
                class="h-3.5 w-3.5 rounded border-slate-600 bg-slate-900 accent-cyan-500"
                @change="toggleRepository(repo.path)"
              />
              <span class="min-w-0 flex-1">
                <span class="block truncate text-xs text-slate-200">{{ repo.name }}</span>
                <span class="block truncate font-mono text-[10px] text-slate-500">
                  {{ repo.path }}
                </span>
              </span>
              <span class="shrink-0 font-mono text-[10px] text-cyan-400/80">
                {{ repo.state.currentBranch }}
              </span>
            </label>
          </div>
          <p class="mt-1.5 text-[11px] text-slate-500">
            {{ repoPaths.length }} of {{ store.repositories.length }} repositories selected
          </p>
        </div>

        <div class="rounded-lg border border-slate-800 bg-slate-950/60 px-4 py-3">
          <p class="mb-2 text-xs font-medium text-slate-300">Group behavior</p>
          <div class="grid grid-cols-2 gap-3">
            <label class="flex items-start gap-2 text-[11px] text-slate-400">
              <input
                v-model="autoStashOnSwitch"
                type="checkbox"
                class="mt-0.5 h-3.5 w-3.5 rounded border-slate-600 bg-slate-900 accent-cyan-500"
              />
              <span>
                Auto-stash before branch switch
                <span class="block text-[10px] text-slate-500">
                  Dirty changes are restored if checkout fails.
                </span>
              </span>
            </label>

            <label class="flex items-start gap-2 text-[11px] text-slate-400">
              <input
                v-model="syncBranching"
                type="checkbox"
                class="mt-0.5 h-3.5 w-3.5 rounded border-slate-600 bg-slate-900 accent-cyan-500"
              />
              <span>
                Synchronous branching
                <span class="block text-[10px] text-slate-500">
                  Offer group-wide branch switching in the toolbar.
                </span>
              </span>
            </label>
          </div>

          <div class="mt-3 w-52">
            <label for="base-branch" class="mb-1 block text-[11px] text-slate-400">
              Default base branch
            </label>
            <input
              id="base-branch"
              v-model="defaultBaseBranch"
              type="text"
              placeholder="main"
              class="w-full rounded border border-slate-700 bg-slate-950 px-2.5 py-1.5 font-mono text-[11px] text-slate-200 outline-none focus:border-cyan-500"
            />
          </div>
        </div>

        <p v-if="error" class="text-[11px] text-rose-400">{{ error }}</p>
      </div>

      <footer class="flex items-center justify-between border-t border-slate-800 px-5 py-3">
        <button
          v-if="isEditing"
          type="button"
          class="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs text-rose-400 transition-colors hover:bg-rose-950/60 hover:text-rose-300 disabled:opacity-50"
          :disabled="deleting"
          @click="remove"
        >
          <Loader2 v-if="deleting" class="h-3.5 w-3.5 animate-spin" />
          <Trash2 v-else class="h-3.5 w-3.5" />
          <span>Delete group</span>
        </button>
        <span v-else></span>

        <div class="flex items-center gap-2">
          <button
            type="button"
            class="rounded-lg px-3 py-1.5 text-xs text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-200"
            @click="store.closeGroupEditor"
          >
            Cancel
          </button>
          <button
            type="submit"
            :disabled="submitting"
            class="flex items-center gap-1.5 rounded-lg bg-cyan-500 px-3.5 py-1.5 text-xs font-semibold text-slate-950 transition-colors hover:bg-cyan-400 disabled:cursor-not-allowed disabled:opacity-50"
          >
            <Loader2 v-if="submitting" class="h-3.5 w-3.5 animate-spin" />
            <span>{{ isEditing ? 'Save changes' : 'Create group' }}</span>
          </button>
        </div>
      </footer>
    </form>
  </div>
</template>
