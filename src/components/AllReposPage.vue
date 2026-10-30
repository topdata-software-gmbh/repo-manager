<script setup lang="ts">
import { computed } from 'vue';
import { FolderSearch, Search, SlidersHorizontal } from 'lucide-vue-next';
import {
  TdButton,
  TdFormField,
  TdInput,
  TdSegmentedControl,
  TdSelect,
} from '@topdata-software-gmbh/ui-kit-web-vue3';
import RepoListTable from './RepoListTable.vue';
import { useRepoStore } from '../stores/repoStore';
import type { RepoStatusFilter } from '../lib/repoFilters';
import type { RepoKind } from '../types/domain';

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

const kindOptions: Array<{ value: RepoKind | 'all'; label: string }> = [
  { value: 'all', label: 'Any type' },
  { value: 'theme', label: 'Theme' },
  { value: 'plugin', label: 'Plugin' },
  { value: 'other', label: 'Other' },
];

const kindFilter = computed({
  get: () => store.allReposFilters.kind,
  set: (value: string) => {
    store.allReposFilters.kind = value as RepoKind | 'all';
  },
});

const statusFilter = computed({
  get: () => store.allReposFilters.statuses,
  set: (value: string[]) => {
    store.allReposFilters.statuses = value as RepoStatusFilter[];
  },
});

const statusOptionsWithCounts = computed(() =>
  statusOptions.map((option) => ({
    ...option,
    count: store.allReposStatusCounts[option.value],
  })),
);

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
  <section class="flex min-h-0 flex-1 flex-col bg-surface">
    <header class="border-b border-border px-4 py-3">
      <div class="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 class="text-base font-semibold text-fg">All repos</h1>
          <p class="mt-0.5 text-xs text-fg-muted">
            Browse every project. Save, update, or send work from a single row — or open a group to
            act on several at once.
          </p>
        </div>
        <TdButton variant="secondary" @click="store.openAddScanPath">
          <template #icon-left><FolderSearch /></template>
          Add project folder
        </TdButton>
      </div>

      <div class="mt-3 flex flex-wrap items-end gap-2">
        <div class="min-w-56 flex-1">
          <TdInput
            v-model="store.allReposFilters.search"
            type="search"
            placeholder="Search name, path, or branch…"
            aria-label="Search all repositories"
          >
            <template #prefix><Search class="h-3.5 w-3.5 text-fg-muted" /></template>
          </TdInput>
        </div>

        <div class="w-44">
          <TdFormField label="Group">
            <template #default="{ id }">
              <TdSelect :id="id" v-model="store.allReposFilters.groupId" :options="groupOptions" />
            </template>
          </TdFormField>
        </div>

        <div class="w-36">
          <TdFormField label="Type">
            <template #default="{ id }">
              <TdSelect :id="id" v-model="kindFilter" :options="kindOptions" />
            </template>
          </TdFormField>
        </div>
      </div>

      <div class="mt-3 flex flex-wrap items-center gap-1.5">
        <TdSegmentedControl
          v-model="statusFilter"
          :options="statusOptionsWithCounts"
          aria-label="Filter by status"
        />

        <TdButton
          v-if="hasFilters"
          variant="ghost"
          size="sm"
          class="ml-1"
          @click="store.clearAllReposFilters"
        >
          <template #icon-left><SlidersHorizontal /></template>
          Clear filters
        </TdButton>
      </div>
    </header>

    <div class="flex items-center justify-between border-b border-border px-4 py-2">
      <span class="font-mono text-[11px] text-fg-muted">
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
        <h2 class="text-base font-semibold text-fg">No projects yet</h2>
        <p class="mt-1.5 max-w-md text-sm text-fg-muted">
          Find the folder where your projects live. We will add every Git project inside it.
        </p>
        <TdButton class="mt-4" variant="primary" @click="store.openAddScanPath">
          Add project folder
        </TdButton>
      </template>
      <template v-else>
        <h2 class="text-base font-semibold text-fg">No repos match</h2>
        <p class="mt-1.5 text-sm text-fg-muted">
          Try another filter or clear everything to see all projects.
        </p>
        <TdButton class="mt-4" variant="ghost" @click="store.clearAllReposFilters">
          Clear filters
        </TdButton>
      </template>
    </div>
  </section>
</template>
