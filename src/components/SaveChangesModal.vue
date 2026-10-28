<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { FileText, Loader2, Save, X } from 'lucide-vue-next';
import { backend } from '../lib/backend';
import { useRepoStore } from '../stores/repoStore';
import type { FileChange, FileChangeStatus, Repository } from '../types/domain';

const store = useRepoStore();

const repos = ref<Repository[]>([]);
const changes = ref<Record<string, FileChange[]>>({});
const message = ref('');
const loading = ref(false);
const saving = ref(false);
const error = ref('');

const totalFiles = computed(() =>
  repos.value.reduce((sum, repo) => sum + (changes.value[repo.path]?.length ?? 0), 0),
);

const canSave = computed(
  () => message.value.trim().length > 0 && repos.value.length > 0 && !loading.value && !saving.value,
);

watch(
  () => store.saveModalOpen,
  (open) => {
    if (!open) return;

    repos.value = store.dirtyRepositories.filter(
      (repo) => !store.saveModalPaths || store.saveModalPaths.includes(repo.path),
    );
    changes.value = {};
    message.value = '';
    error.value = '';
    saving.value = false;
    void loadChanges();
  },
);

async function loadChanges(): Promise<void> {
  loading.value = true;
  error.value = '';

  const results = await Promise.allSettled(
    repos.value.map(async (repo) => ({
      path: repo.path,
      changes: await backend.listRepoChanges(repo.path),
    })),
  );

  const next: Record<string, FileChange[]> = {};
  const failed: string[] = [];
  results.forEach((result, index) => {
    const repo = repos.value[index];
    if (!repo) return;
    if (result.status === 'fulfilled') {
      next[repo.path] = result.value.changes;
    } else {
      failed.push(repo.name);
      next[repo.path] = [];
    }
  });

  changes.value = next;
  if (failed.length > 0) {
    error.value = `Could not read the changes in: ${failed.join(', ')}.`;
  }
  loading.value = false;
}

function statusLabel(status: FileChangeStatus): string {
  switch (status) {
    case 'new':
      return 'New file';
    case 'deleted':
      return 'Deleted';
    case 'renamed':
      return 'Renamed';
    case 'conflict':
      return 'Conflict';
    default:
      return 'Changed';
  }
}

function statusClass(status: FileChangeStatus): string {
  switch (status) {
    case 'new':
      return 'text-emerald-300';
    case 'deleted':
      return 'text-rose-300';
    case 'conflict':
      return 'text-rose-400';
    case 'renamed':
      return 'text-sky-300';
    default:
      return 'text-amber-300';
  }
}

async function save(): Promise<void> {
  if (!canSave.value) return;

  saving.value = true;
  error.value = '';
  const paths = repos.value.map((repo) => repo.path);
  const results = await store.commitRepositories(paths, message.value.trim());
  const failures = results.filter((result) => !result.success);

  if (failures.length === 0) {
    store.showToast(
      'success',
      'Saved to Git',
      `${repos.value.length} ${repos.value.length === 1 ? 'project' : 'projects'} updated.`,
    );
    saving.value = false;
    store.closeSaveChanges();
    return;
  }

  const failedPaths = new Set(failures.map((failure) => failure.repoPath));
  repos.value = repos.value.filter((repo) => failedPaths.has(repo.path));
  message.value = '';
  error.value = failures
    .map((failure) => `${failure.repoPath.split(/[\\/]/).pop()}: ${failure.error ?? 'Unknown error'}`)
    .join('\n\n');
  saving.value = false;
  await loadChanges();
}
</script>

<template>
  <div
    v-if="store.saveModalOpen"
    class="fixed inset-0 z-40 flex items-center justify-center bg-slate-950/70 p-6 backdrop-blur-sm"
    @click.self="!saving && store.closeSaveChanges()"
  >
    <form
      class="flex max-h-[88vh] w-full max-w-2xl flex-col rounded-xl border border-slate-700 bg-slate-900 shadow-2xl"
      @submit.prevent="save"
    >
      <header class="flex items-center justify-between border-b border-slate-800 px-5 py-4">
        <div class="flex items-center gap-2">
          <Save class="h-4 w-4 text-cyan-400" />
          <div>
            <h2 class="text-sm font-semibold text-slate-100">Save to Git</h2>
            <p class="text-[11px] text-slate-500">
              Create a checkpoint of your work. Nothing is deleted or moved.
            </p>
          </div>
        </div>
        <button
          type="button"
          class="rounded p-1 text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-200 disabled:opacity-50"
          :disabled="saving"
          @click="store.closeSaveChanges"
        >
          <X class="h-4 w-4" />
        </button>
      </header>

      <div class="flex-1 space-y-4 overflow-auto px-5 py-4">
        <div
          v-if="loading"
          class="flex items-center gap-2 rounded-lg border border-slate-800 bg-slate-950/60 px-4 py-3 text-xs text-slate-400"
        >
          <Loader2 class="h-3.5 w-3.5 animate-spin" />
          Reading your changes…
        </div>

        <div
          v-for="repo in repos"
          :key="repo.id"
          class="overflow-hidden rounded-lg border border-slate-800 bg-slate-950/60"
        >
          <div class="flex items-center justify-between gap-3 px-4 py-3">
            <div class="min-w-0">
              <p class="truncate text-sm font-medium text-slate-100">{{ repo.name }}</p>
              <p class="truncate text-[11px] text-slate-500">
                Working on <span class="font-mono text-slate-400">{{ repo.state.currentBranch }}</span>
              </p>
            </div>
            <span class="shrink-0 rounded-full bg-slate-800 px-2 py-0.5 text-[10px] text-slate-300">
              {{ changes[repo.path]?.length ?? 0 }} changed
            </span>
          </div>

          <div v-if="(changes[repo.path]?.length ?? 0) > 0" class="border-t border-slate-800/70">
            <div class="max-h-40 overflow-auto px-4 py-2">
              <div
                v-for="change in changes[repo.path]"
                :key="`${change.status}:${change.path}`"
                class="flex items-center gap-2 py-0.5 font-mono text-[11px]"
              >
                <FileText class="h-3 w-3 shrink-0 text-slate-600" />
                <span class="min-w-0 flex-1 truncate text-slate-400" :title="change.path">
                  {{ change.path }}
                </span>
                <span class="shrink-0 text-[10px]" :class="statusClass(change.status)">
                  {{ statusLabel(change.status) }}
                </span>
              </div>
            </div>
          </div>
        </div>

        <div>
          <label for="commit-message" class="mb-1.5 block text-xs font-medium text-slate-300">
            What did you change?
          </label>
          <textarea
            id="commit-message"
            v-model="message"
            rows="3"
            autofocus
            placeholder="For example: Updated the header layout and product card colors"
            class="w-full resize-none rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 outline-none transition-colors placeholder:text-slate-600 focus:border-cyan-500"
          ></textarea>
          <p class="mt-1.5 text-[11px] leading-relaxed text-slate-500">
            Write a short note for your future self. It will appear in the project history.
            <span v-if="repos.length > 1">
              One checkpoint will be created in each of the {{ repos.length }} projects.
            </span>
          </p>
        </div>

        <pre
          v-if="error"
          class="max-h-40 overflow-auto whitespace-pre-wrap rounded-lg border border-rose-500/30 bg-rose-500/10 px-3 py-2 text-[11px] leading-relaxed text-rose-200"
        >{{ error }}</pre>
      </div>

      <footer class="flex items-center justify-between border-t border-slate-800 px-5 py-3">
        <p class="text-[11px] text-slate-500">
          {{ totalFiles }} {{ totalFiles === 1 ? 'file' : 'files' }} across
          {{ repos.length }} {{ repos.length === 1 ? 'project' : 'projects' }}
        </p>
        <div class="flex items-center gap-2">
          <button
            type="button"
            class="rounded-lg px-3 py-1.5 text-xs text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-200 disabled:opacity-50"
            :disabled="saving"
            @click="store.closeSaveChanges"
          >
            Cancel
          </button>
          <button
            type="submit"
            :disabled="!canSave"
            class="flex items-center gap-1.5 rounded-lg bg-cyan-500 px-3.5 py-1.5 text-xs font-semibold text-slate-950 transition-colors hover:bg-cyan-400 disabled:cursor-not-allowed disabled:opacity-50"
          >
            <Loader2 v-if="saving" class="h-3.5 w-3.5 animate-spin" />
            <Save v-else class="h-3.5 w-3.5" />
            <span>Save to Git</span>
          </button>
        </div>
      </footer>
    </form>
  </div>
</template>
