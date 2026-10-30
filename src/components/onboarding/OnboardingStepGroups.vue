<script setup lang="ts">
import { computed, ref } from 'vue';
import { ArrowLeft, ArrowRight, Layers } from 'lucide-vue-next';
import {
  TdAlert,
  TdBadge,
  TdButton,
  TdCheckbox,
  TdInput,
} from '@topdata-software-gmbh/ui-kit-web-vue3';
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
      <h2 class="text-lg font-semibold text-fg">Confirm your groups</h2>
      <p class="mt-1.5 text-sm leading-relaxed text-fg-muted">
        We suggested these from your folders. Rename them or leave one out; you can fine-tune
        membership later.
      </p>
    </div>

    <p class="font-mono text-xs text-fg-muted">
      {{ store.onboardingSelectedRepositories.length }}
      {{ store.onboardingSelectedRepositories.length === 1 ? 'project' : 'projects' }} accepted
    </p>

    <div v-if="store.onboardingGroups.length > 0" class="space-y-3">
      <div
        v-for="(group, index) in store.onboardingGroups"
        :key="index"
        class="rounded-xl border border-border bg-canvas px-4 py-3"
        :class="{ 'opacity-60': !group.included }"
      >
        <div class="flex items-start gap-3">
          <TdCheckbox
            :model-value="group.included"
            :aria-label="`Include ${group.name}`"
            class="mt-1"
            @update:model-value="store.toggleOnboardingGroup(index)"
          />
          <div class="min-w-0 flex-1 space-y-2">
            <div class="flex flex-wrap items-center gap-2">
              <Layers class="h-3.5 w-3.5 shrink-0 text-primary" />
              <div class="min-w-0 flex-1">
                <TdInput
                  :model-value="group.name"
                  type="text"
                  :aria-label="`Name for group ${index + 1}`"
                  @update:model-value="store.updateOnboardingGroupName(index, $event)"
                />
              </div>
              <TdBadge variant="neutral">
                {{ group.repoPaths.length }}
                {{ group.repoPaths.length === 1 ? 'project' : 'projects' }}
              </TdBadge>
            </div>
            <p class="text-[11px] text-fg-muted">{{ repoSummary(group.repoPaths) }}</p>
          </div>
        </div>
      </div>
    </div>

    <div
      v-else
      class="rounded-xl border border-dashed border-border px-5 py-10 text-center text-sm text-fg-muted"
    >
      No groups to suggest. Go back and select at least one project.
    </div>

    <TdAlert v-if="errorMessage" variant="error">{{ errorMessage }}</TdAlert>

    <div class="flex items-center justify-between gap-3">
      <TdButton variant="ghost" @click="store.goToOnboardingStep(1)">
        <template #icon-left><ArrowLeft /></template>
        Back
      </TdButton>
      <TdButton
        variant="primary"
        :disabled="includedGroups.length === 0 || submitting"
        :loading="submitting"
        @click="finish"
      >
        Finish setup
        <template #icon-right><ArrowRight /></template>
      </TdButton>
    </div>
  </div>
</template>
