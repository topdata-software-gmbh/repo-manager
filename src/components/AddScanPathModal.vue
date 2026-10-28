<script setup lang="ts">
import { ref, watch } from 'vue';
import { FolderPlus, Loader2, X } from 'lucide-vue-next';
import { useRepoStore } from '../stores/repoStore';

const store = useRepoStore();
const path = ref('');
const maxDepth = ref(3);
const submitting = ref(false);

watch(
  () => store.scanModalOpen,
  (open) => {
    if (open) {
      path.value = '';
      maxDepth.value = 3;
      submitting.value = false;
    }
  },
);

function newId(prefix: string): string {
  return `${prefix}_${typeof crypto.randomUUID === 'function' ? crypto.randomUUID() : Date.now()}`;
}

async function submit(): Promise<void> {
  const value = path.value.trim();
  if (!value || submitting.value) return;

  submitting.value = true;
  try {
    await store.addScanPath({
      id: newId('scan'),
      path: value,
      maxDepth: Math.min(Math.max(Math.round(maxDepth.value), 1), 12),
      enabled: true,
      excludePatterns: [],
    });
    store.closeScanPathModal();
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div
    v-if="store.scanModalOpen"
    class="fixed inset-0 z-40 flex items-center justify-center bg-slate-950/70 p-6 backdrop-blur-sm"
    @click.self="store.closeScanPathModal"
  >
    <form
      class="w-full max-w-lg rounded-xl border border-slate-700 bg-slate-900 shadow-2xl"
      @submit.prevent="submit"
    >
      <header class="flex items-center justify-between border-b border-slate-800 px-5 py-4">
        <div class="flex items-center gap-2">
          <FolderPlus class="h-4 w-4 text-cyan-400" />
          <h2 class="text-sm font-semibold text-slate-100">Add scan path</h2>
        </div>
        <button
          type="button"
          class="rounded p-1 text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-200"
          @click="store.closeScanPathModal"
        >
          <X class="h-4 w-4" />
        </button>
      </header>

      <div class="space-y-4 px-5 py-4">
        <div>
          <label for="scan-path" class="mb-1.5 block text-xs font-medium text-slate-300">
            Directory containing your repositories
          </label>
          <input
            id="scan-path"
            v-model="path"
            type="text"
            autofocus
            spellcheck="false"
            placeholder="/home/user/devel"
            class="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 font-mono text-xs text-slate-100 outline-none transition-colors placeholder:text-slate-600 focus:border-cyan-500"
          />
          <p class="mt-1.5 text-[11px] leading-relaxed text-slate-500">
            The directory is scanned recursively. Hidden directories are included, while
            <span class="font-mono">node_modules</span>, <span class="font-mono">vendor</span>,
            build output and caches are skipped.
          </p>
        </div>

        <div class="w-40">
          <label for="scan-depth" class="mb-1.5 block text-xs font-medium text-slate-300">
            Max scan depth
          </label>
          <input
            id="scan-depth"
            v-model.number="maxDepth"
            type="number"
            min="1"
            max="12"
            class="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-xs text-slate-100 outline-none transition-colors focus:border-cyan-500"
          />
        </div>
      </div>

      <footer class="flex items-center justify-end gap-2 border-t border-slate-800 px-5 py-3">
        <button
          type="button"
          class="rounded-lg px-3 py-1.5 text-xs text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-200"
          @click="store.closeScanPathModal"
        >
          Cancel
        </button>
        <button
          type="submit"
          :disabled="!path.trim() || submitting"
          class="flex items-center gap-1.5 rounded-lg bg-cyan-500 px-3.5 py-1.5 text-xs font-semibold text-slate-950 transition-colors hover:bg-cyan-400 disabled:cursor-not-allowed disabled:opacity-50"
        >
          <Loader2 v-if="submitting" class="h-3.5 w-3.5 animate-spin" />
          <span>Add &amp; scan</span>
        </button>
      </footer>
    </form>
  </div>
</template>
