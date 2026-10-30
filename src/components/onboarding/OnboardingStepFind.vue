<script setup lang="ts">
import { computed, ref } from 'vue';
import { FolderSearch } from 'lucide-vue-next';
import {
  TdAlert,
  TdButton,
  TdFormField,
  TdInput,
  TdSelect,
  TdSpinner,
} from '@topdata-software-gmbh/ui-kit-web-vue3';
import { useRepoStore } from '../../stores/repoStore';
import type { GroupingStyle } from '../../types/domain';

const store = useRepoStore();
const submitting = ref(false);

const maxDepth = computed({
  get: () => String(store.onboardingDraft.maxDepth),
  set: (value: string) => {
    const parsed = Number.parseInt(value, 10);
    store.onboardingDraft.maxDepth = Number.isFinite(parsed) ? parsed : 1;
  },
});

const groupingStyle = computed({
  get: () => store.onboardingDraft.groupingStyle,
  set: (value: string) => {
    store.onboardingDraft.groupingStyle = value as GroupingStyle;
  },
});

const groupingOptions = [
  { value: 'topLevel', label: 'By top-level folder' },
  { value: 'single', label: 'One group for everything' },
];

async function submit(): Promise<void> {
  if (submitting.value || store.isScanning) return;
  submitting.value = true;
  try {
    const found = await store.scanOnboardingFolder();
    if (found) {
      await store.goToOnboardingStep(1);
    }
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <form class="mx-auto max-w-2xl space-y-6" @submit.prevent="submit">
    <div>
      <h2 class="text-lg font-semibold text-fg">Where do your projects live?</h2>
      <p class="mt-1.5 text-sm leading-relaxed text-fg-muted">
        We look inside this folder for Git projects and skip dependency and build folders such as
        node_modules, vendor, and dist. You can add more folders later.
      </p>
    </div>

    <div class="space-y-4 rounded-xl border border-border bg-surface p-5">
      <TdFormField
        label="Project folder"
        hint="Paste the folder where your themes live, for example ~/devel or /home/you/projects."
        required
        v-slot="{ id, 'aria-describedby': describedBy, invalid, required }"
      >
        <TdInput
          :id="id"
          v-model="store.onboardingDraft.scanPath"
          type="text"
          spellcheck="false"
          placeholder="~/devel"
          :aria-describedby="describedBy"
          :invalid="invalid"
          :required="required"
        />
      </TdFormField>

      <TdFormField
        label="Remote owner names (optional)"
        hint="Used only to pre-select projects you own. This stays on this computer."
        v-slot="{ id, 'aria-describedby': describedBy }"
      >
        <TdInput
          :id="id"
          v-model="store.onboardingDraft.ownerNames"
          type="text"
          spellcheck="false"
          placeholder="acme, my-company"
          :aria-describedby="describedBy"
        />
      </TdFormField>
    </div>

    <details class="rounded-xl border border-border bg-surface px-5 py-4">
      <summary class="cursor-pointer text-xs font-medium text-fg">Advanced</summary>
      <div class="mt-4 grid gap-4 sm:grid-cols-2">
        <TdFormField
          label="How deep to look"
          hint="Nested clones can sit several folders deep. The default of 8 finds most setups."
          v-slot="{ id, 'aria-describedby': describedBy }"
        >
          <TdInput
            :id="id"
            v-model="maxDepth"
            type="number"
            min="1"
            max="12"
            :aria-describedby="describedBy"
          />
        </TdFormField>

        <TdFormField
          label="How to suggest groups"
          hint="You can rename, skip, or change group membership later."
          v-slot="{ id, 'aria-describedby': describedBy }"
        >
          <TdSelect
            :id="id"
            v-model="groupingStyle"
            :options="groupingOptions"
            :aria-describedby="describedBy"
          />
        </TdFormField>
      </div>
    </details>

    <TdAlert v-if="store.onboardingError" variant="error">{{ store.onboardingError }}</TdAlert>

    <p
      v-if="store.isScanning || submitting"
      class="flex items-center gap-2 text-sm text-fg-muted"
      role="status"
    >
      <TdSpinner size="sm" />
      Looking for Git projects. This can take a moment in large folders…
    </p>

    <div class="flex items-center gap-3">
      <TdButton
        type="submit"
        variant="primary"
        :disabled="submitting || store.isScanning"
        :loading="submitting || store.isScanning"
      >
        <template #icon-left><FolderSearch /></template>
        <span>Find my projects</span>
      </TdButton>
    </div>
  </form>
</template>
