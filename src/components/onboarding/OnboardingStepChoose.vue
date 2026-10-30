<script setup lang="ts">
import { computed } from 'vue';
import { ArrowLeft, ArrowRight, Search } from 'lucide-vue-next';
import { useRepoStore } from '../../stores/repoStore';
import type { RepoKind } from '../../types/domain';

const store = useRepoStore();

const visibleRepositories = computed(() => {
  const term = store.onboardingSearch.trim().toLowerCase();
  if (!term) return store.repositories;
  return store.repositories.filter((repo) =>
    `${repo.name} ${repo.path} ${repo.state.currentBranch}`.toLowerCase().includes(term),
  );
});

const selectedCount = computed(() => store.onboardingSelectedPaths.length);

function kindLabel(kind: RepoKind): string {
  switch (kind) {
    case 'theme':
      return 'Theme';
    case 'plugin':
      return 'Plugin';
    default:
      return 'Other';
  }
}

function isSelected(path: string): boolean {
  return store.onboardingSelectedPaths.includes(path);
}

function selectVisible(): void {
  store.selectAllOnboardingRepos([
    ...store.onboardingSelectedPaths,
    ...visibleRepositories.value.map((repo) => repo.path),
  ]);
}
</script>

<template>
  <div class="space-y-5">
    <div>
      <h2 class="text-lg font-semibold text-slate-100">Choose the projects to work with</h2>
      <p class="mt-1.5 text-sm leading-relaxed text-slate-400">
        Everything we found is listed here. Projects you own are already selected; adjust the list
        as you like.
      </p>
    </div>

    <div class="flex flex-wrap items-center gap-2">
      <label class="relative min-w-56 flex-1">
        <span class="sr-only">Search projects</span>
        <Search
          class="pointer-events-none absolute left-3 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-slate-500"
        />
        <input
          v-model="store.onboardingSearch"
          type="search"
          placeholder="Search name, path, or branch…"
          class="w-full rounded-lg border border-slate-800 bg-slate-950 py-2 pl-9 pr-3 text-sm text-slate-200 placeholder:text-slate-600 focus:border-cyan-600"
        />
      </label>
      <button
        type="button"
        class="rounded-lg bg-slate-800 px-3 py-2 text-xs text-slate-200 hover:bg-slate-700"
        @click="selectVisible"
      >
        Select all visible
      </button>
      <button
        type="button"
        class="rounded-lg px-3 py-2 text-xs text-slate-400 hover:bg-slate-800 hover:text-slate-200"
        @click="store.clearOnboardingSelection"
      >
        Clear selection
      </button>
    </div>

    <p class="font-mono text-xs text-slate-500">
      {{ selectedCount }} of {{ store.repositories.length }}
      {{ store.repositories.length === 1 ? 'project' : 'projects' }} selected
    </p>

    <div v-if="store.isScanning" class="space-y-2">
      <div
        v-for="index in 5"
        :key="index"
        class="h-14 animate-pulse rounded-lg border border-slate-800 bg-slate-800/30"
      ></div>
    </div>

    <div
      v-else-if="visibleRepositories.length > 0"
      class="max-h-[50vh] divide-y divide-slate-800 overflow-auto rounded-xl border border-slate-800 bg-slate-950"
    >
      <label
        v-for="repo in visibleRepositories"
        :key="repo.id"
        class="flex cursor-pointer items-center gap-3 px-4 py-2.5 transition-colors hover:bg-slate-900"
      >
        <input
          type="checkbox"
          :checked="isSelected(repo.path)"
          class="h-4 w-4 rounded border-slate-600 bg-slate-900 accent-cyan-500"
          @change="store.toggleOnboardingRepo(repo.path)"
        />
        <span class="min-w-0 flex-1">
          <span class="flex items-center gap-2">
            <span class="truncate text-sm text-slate-200">{{ repo.name }}</span>
            <span class="rounded bg-slate-800 px-1.5 py-0.5 text-[10px] text-slate-400">
              {{ kindLabel(repo.kind) }}
            </span>
          </span>
          <span class="block truncate font-mono text-[10px] text-slate-500">{{ repo.path }}</span>
        </span>
        <span class="shrink-0 font-mono text-[10px] text-cyan-400/80">
          {{ repo.state.currentBranch }}
        </span>
      </label>
    </div>

    <div
      v-else-if="store.onboardingSearch.trim()"
      class="rounded-xl border border-dashed border-slate-700 px-5 py-10 text-center"
    >
      <h3 class="text-sm font-semibold text-slate-100">No projects match</h3>
      <p class="mt-1 text-xs text-slate-400">Try a different search term.</p>
      <button
        type="button"
        class="mt-3 rounded-lg px-3 py-1.5 text-xs text-slate-300 hover:bg-slate-800"
        @click="store.onboardingSearch = ''"
      >
        Clear search
      </button>
    </div>

    <div
      v-else
      class="rounded-xl border border-dashed border-slate-700 px-5 py-10 text-center"
    >
      <h3 class="text-sm font-semibold text-slate-100">No Git projects found</h3>
      <p class="mt-1 text-xs text-slate-400">
        Check the folder, or go back and choose another one.
      </p>
    </div>

    <p v-if="store.onboardingError" role="alert" class="text-sm text-rose-400">
      {{ store.onboardingError }}
    </p>

    <div class="flex items-center justify-between gap-3">
      <button
        type="button"
        class="flex items-center gap-1.5 rounded-lg px-3 py-2 text-xs text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-200"
        @click="store.goToOnboardingStep(0)"
      >
        <ArrowLeft class="h-3.5 w-3.5" />
        Back
      </button>
      <button
        type="button"
        class="flex items-center gap-2 rounded-lg bg-cyan-500 px-4 py-2 text-sm font-semibold text-slate-950 transition-colors hover:bg-cyan-400 disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="selectedCount === 0 || store.isScanning"
        @click="store.goToOnboardingStep(2)"
      >
        Continue
        <ArrowRight class="h-4 w-4" />
      </button>
    </div>
  </div>
</template>
