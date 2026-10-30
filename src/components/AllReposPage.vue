<script setup lang="ts">
import { computed } from 'vue';
import { FolderSearch, Search, SlidersHorizontal } from 'lucide-vue-next';
import RepoListTable from './RepoListTable.vue';
import { useRepoStore } from '../stores/repoStore';
import type { RepoStatusFilter } from '../lib/repoFilters';

const store = useRepoStore();

const statusOptions: Array<{ value: RepoStatusFilter; label: string }> = [
  { value: 'needsAttention', label: 'Needs attention' },
  { value: 'changesNotSaved', label: 'Changes not saved' },
  { value: 'updatesAvailable', label: 'Updates available' },
  { value: 'toSend', label: 'To send' },
  { value: 'upToDate', label: 'Up to date' },
  { value: 'localOnly', label: 'On this computer only' },
];

const groupOptions = computed(() => [
  { value: 'all', label: 'Any group' },
  { value: 'ungrouped', label: 'Ungrouped' },
  ...store.config.groups.map((group) => ({ value: group.id, label: group.name })),
]);

const hasFilters = computed(() => {
  const filters = store.allReposFilters;
  return (
    filters.statuses.length > 0 ||
    filters.groupId !== 'all' ||
    filters.kind !== 'all' ||
    filters.search.trim() !== ''
  );
});
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col bg-slate-900">
    <header class="border-b border-slate-800/70 px-4 py-3">
      <div class="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 class="text-base font-semibold text-slate-100">All repos</h1>
          <p class="mt-0.5 text-xs text-slate-400">
            Browse every project. Save, update, or send work from a single row — or open a group to
            act on several at once.
          </p>
        </div>
        <button
          class="flex items-center gap-1.5 rounded-lg bg-slate-800 px-3 py-1.5 text-xs text-slate-200 hover:bg-slate-700"
          @click="store.openAddScanPath"
        >
          <FolderSearch class="h-3.5 w-3.5" />
          Add project folder
        </button>
      </div>

      <div class="mt-3 flex flex-wrap items-center gap-2">
        <label class="relative min-w-56 flex-1">
          <span class="sr-only">Search all repositories</span>
          <Search
            class="pointer-events-none absolute left-3 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-slate-500"
          />
          <input
            v-model="store.allReposFilters.search"
            type="search"
            placeholder="Search name, path, or branch…"
            class="w-full rounded-lg border border-slate-800 bg-slate-950 py-1.5 pl-9 pr-3 text-xs text-slate-200 placeholder:text-slate-600 focus:border-cyan-600"
          />
        </label>

        <label class="flex items-center gap-2 text-[11px] text-slate-400">
          Group
          <select
            v-model="store.allReposFilters.groupId"
            class="rounded-lg border border-slate-800 bg-slate-950 px-2 py-1.5 text-xs text-slate-200"
          >
            <option v-for="option in groupOptions" :key="option.value" :value="option.value">
              {{ option.label }}
            </option>
          </select>
        </label>

        <label class="flex items-center gap-2 text-[11px] text-slate-400">
          Type
          <select
            v-model="store.allReposFilters.kind"
            class="rounded-lg border border-slate-800 bg-slate-950 px-2 py-1.5 text-xs text-slate-200"
          >
            <option value="all">Any type</option>
            <option value="theme">Theme</option>
            <option value="plugin">Plugin</option>
            <option value="other">Other</option>
          </select>
        </label>
      </div>

      <div class="mt-3 flex flex-wrap items-center gap-1.5" aria-label="Filter by status">
        <button
          v-for="option in statusOptions"
          :key="option.value"
          type="button"
          class="rounded-full border px-2.5 py-1 text-[11px] transition-colors"
          :class="
            store.allReposFilters.statuses.includes(option.value)
              ? 'border-cyan-500/50 bg-cyan-500/15 text-cyan-200'
              : 'border-slate-800 bg-slate-900 text-slate-400 hover:text-slate-200'
          "
          :aria-pressed="store.allReposFilters.statuses.includes(option.value)"
          @click="store.toggleAllReposStatus(option.value)"
        >
          {{ option.label }}
          <span class="ml-1 font-mono text-[10px] text-slate-500">
            {{ store.allReposStatusCounts[option.value] }}
          </span>
        </button>

        <button
          v-if="hasFilters"
          type="button"
          class="ml-1 flex items-center gap-1 text-[11px] text-slate-400 hover:text-slate-200"
          @click="store.clearAllReposFilters"
        >
          <SlidersHorizontal class="h-3 w-3" />
          Clear filters
        </button>
      </div>
    </header>

    <div class="flex items-center justify-between border-b border-slate-800/70 px-4 py-2">
      <span class="font-mono text-[11px] text-slate-500">
        {{ store.allReposFiltered.length }} of {{ store.repositories.length }} repos
      </span>
    </div>

    <RepoListTable
      v-if="store.allReposFiltered.length > 0"
      :repos="store.allReposFiltered"
      actions="browse"
      class="min-h-0 flex-1"
    />

    <div v-else class="flex flex-1 flex-col items-center justify-center px-6 text-center">
      <template v-if="store.repositories.length === 0">
        <h2 class="text-base font-semibold text-slate-100">No projects yet</h2>
        <p class="mt-1.5 max-w-md text-sm text-slate-400">
          Find the folder where your projects live. We will add every Git project inside it.
        </p>
        <button
          class="mt-4 rounded-lg bg-cyan-500 px-4 py-2 text-xs font-semibold text-slate-950"
          @click="store.openAddScanPath"
        >
          Add project folder
        </button>
      </template>
      <template v-else>
        <h2 class="text-base font-semibold text-slate-100">No repos match</h2>
        <p class="mt-1.5 text-sm text-slate-400">
          Try another filter or clear everything to see all projects.
        </p>
        <button
          class="mt-4 rounded-lg px-3 py-1.5 text-xs text-slate-300 hover:bg-slate-800"
          @click="store.clearAllReposFilters"
        >
          Clear filters
        </button>
      </template>
    </div>
  </section>
</template>
