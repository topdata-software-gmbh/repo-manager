<script setup lang="ts">
import { computed, ref } from 'vue';
import { ArrowLeft, ArrowRight, Layers } from 'lucide-vue-next';
import { useRepoStore } from '../../stores/repoStore';
import { createGroup } from '../../lib/groupDefaults';

const store = useRepoStore();
const submitting = ref(false);
const localError = ref('');

const includedGroups = computed(() =>
  store.onboardingGroups.filter((group) => group.included),
);

const errorMessage = computed(() => localError.value || store.onboardingError);

function repoName(path: string): string {
  return store.repositories.find((repo) => repo.path === path)?.name ?? path.split(/[\\/]/).pop() ?? path;
}

function repoSummary(paths: string[]): string {
  const names = paths.slice(0, 3).map(repoName);
  const suffix = paths.length > 3 ? ` and ${paths.length - 3} more` : '';
  return names.join(', ') + suffix;
}

async function finish(): Promise<void> {
  localError.value = '';
  const names = includedGroups.value.map((group) => group.name.trim().toLowerCase());

  if (names.length === 0) {
    localError.value = 'Keep at least one group to continue.';
    return;
  }
  if (names.some((name) => name.length === 0)) {
    localError.value = 'Give every included group a name.';
    return;
  }
  if (new Set(names).size !== names.length) {
    localError.value = 'Give each group a different name.';
    return;
  }
  if (submitting.value) return;

  submitting.value = true;
  try {
    const groups = includedGroups.value.map((group, index) =>
      createGroup(group.name.trim(), group.repoPaths, index),
    );
    await store.prepareOnboardingGroups(groups);
  } catch (error) {
    localError.value = String(error);
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="space-y-5">
    <div>
      <h2 class="text-lg font-semibold text-slate-100">Confirm your groups</h2>
      <p class="mt-1.5 text-sm leading-relaxed text-slate-400">
        We suggested these from your folders. Rename them or leave one out; you can fine-tune
        membership later.
      </p>
    </div>

    <p class="font-mono text-xs text-slate-500">
      {{ store.onboardingSelectedRepositories.length }}
      {{ store.onboardingSelectedRepositories.length === 1 ? 'project' : 'projects' }} accepted
    </p>

    <div v-if="store.onboardingGroups.length > 0" class="space-y-3">
      <div
        v-for="(group, index) in store.onboardingGroups"
        :key="index"
        class="rounded-xl border border-slate-800 bg-slate-950 px-4 py-3"
        :class="{ 'opacity-60': !group.included }"
      >
        <div class="flex items-start gap-3">
          <input
            type="checkbox"
            :checked="group.included"
            :aria-label="`Include ${group.name}`"
            class="mt-1 h-4 w-4 rounded border-slate-600 bg-slate-900 accent-cyan-500"
            @change="store.toggleOnboardingGroup(index)"
          />
          <div class="min-w-0 flex-1 space-y-2">
            <div class="flex flex-wrap items-center gap-2">
              <Layers class="h-3.5 w-3.5 shrink-0 text-cyan-400" />
              <input
                :value="group.name"
                type="text"
                :aria-label="`Name for group ${index + 1}`"
                class="min-w-0 flex-1 rounded-lg border border-slate-700 bg-slate-900 px-3 py-1.5 text-sm text-slate-100 outline-none focus:border-cyan-500"
                @input="
                  store.updateOnboardingGroupName(
                    index,
                    ($event.target as HTMLInputElement).value,
                  )
                "
              />
              <span class="shrink-0 rounded-full bg-slate-800 px-2 py-0.5 font-mono text-[10px] text-slate-400">
                {{ group.repoPaths.length }}
                {{ group.repoPaths.length === 1 ? 'project' : 'projects' }}
              </span>
            </div>
            <p class="text-[11px] text-slate-500">{{ repoSummary(group.repoPaths) }}</p>
          </div>
        </div>
      </div>
    </div>

    <div
      v-else
      class="rounded-xl border border-dashed border-slate-700 px-5 py-10 text-center text-sm text-slate-400"
    >
      No groups to suggest. Go back and select at least one project.
    </div>

    <p v-if="errorMessage" role="alert" class="text-sm text-rose-400">{{ errorMessage }}</p>

    <div class="flex items-center justify-between gap-3">
      <button
        type="button"
        class="flex items-center gap-1.5 rounded-lg px-3 py-2 text-xs text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-200"
        @click="store.goToOnboardingStep(1)"
      >
        <ArrowLeft class="h-3.5 w-3.5" />
        Back
      </button>
      <button
        type="button"
        class="flex items-center gap-2 rounded-lg bg-cyan-500 px-4 py-2 text-sm font-semibold text-slate-950 transition-colors hover:bg-cyan-400 disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="includedGroups.length === 0 || submitting"
        @click="finish"
      >
        Finish setup
        <ArrowRight class="h-4 w-4" />
      </button>
    </div>
  </div>
</template>
