<script setup lang="ts">
import { computed, ref } from 'vue';
import { BookOpen, FolderPlus, GitBranch, Search } from 'lucide-vue-next';
import RepoListTable from './RepoListTable.vue';
import { useRepoStore } from '../stores/repoStore';

const store = useRepoStore();
const search = ref('');

const rows = computed(() => {
  const term = search.value.trim().toLowerCase();
  if (!term) return store.filteredRepositories;

  return store.filteredRepositories.filter((repo) =>
    `${repo.name} ${repo.path} ${repo.state.currentBranch}`.toLowerCase().includes(term),
  );
});

const hasScanPaths = computed(() => store.config.scanPaths.length > 0);
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
      <RepoListTable v-if="rows.length > 0" :repos="rows" actions="group" />

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
