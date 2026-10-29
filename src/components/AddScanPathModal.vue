<script setup lang="ts">
import { ref, watch } from 'vue';
import { TdButton, TdFormField, TdInput, TdModal } from '@topdata-software-gmbh/ui';
import { useRepoStore } from '../stores/repoStore';

const store = useRepoStore();
const path = ref('');
const maxDepth = ref('3');
const submitting = ref(false);

watch(
  () => store.scanModalOpen,
  (open) => {
    if (!open) return;
    path.value = '';
    maxDepth.value = '3';
    submitting.value = false;
  },
);

function newId(prefix: string): string {
  return `${prefix}_${typeof crypto.randomUUID === 'function' ? crypto.randomUUID() : Date.now()}`;
}

function onOpenChange(open: boolean): void {
  if (!open) store.closeScanPathModal();
}

async function submit(): Promise<void> {
  const value = path.value.trim();
  if (!value || submitting.value) return;

  submitting.value = true;
  try {
    const parsed = Number(maxDepth.value);
    const depth = Number.isFinite(parsed) ? Math.round(parsed) : 3;
    const added = await store.addScanPath({
      id: newId('scan'),
      path: value,
      maxDepth: Math.min(Math.max(depth, 1), 12),
      enabled: true,
      excludePatterns: [],
    });
    if (added) {
      store.closeScanPathModal();
    }
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <TdModal
    :open="store.scanModalOpen"
    title="Add project folder"
    @update:open="onOpenChange"
  >
    <form id="scan-path-form" class="scan-form" @submit.prevent="submit">
      <TdFormField
        label="Where are your projects?"
        hint="Paste the folder where your themes live, for example ~/devel or /home/you/projects. We look inside it for Git projects and skip things like node_modules and build output."
        required
        v-slot="{ id, 'aria-describedby': describedBy, invalid, required }"
      >
        <TdInput
          :id="id"
          v-model="path"
          type="text"
          spellcheck="false"
          placeholder="~/devel"
          :aria-describedby="describedBy"
          :invalid="invalid"
          :required="required"
          data-autofocus
        />
      </TdFormField>

      <div class="scan-form__depth">
        <TdFormField label="How deep to look" v-slot="{ id }">
          <TdInput :id="id" v-model="maxDepth" type="number" min="1" max="12" />
        </TdFormField>
      </div>
    </form>

    <template #footer>
      <TdButton variant="ghost" @click="store.closeScanPathModal">Cancel</TdButton>
      <TdButton
        variant="primary"
        type="submit"
        form="scan-path-form"
        :disabled="!path.trim() || submitting"
        :loading="submitting"
      >
        Find my projects
      </TdButton>
    </template>
  </TdModal>
</template>

<style scoped>
.scan-form {
  display: grid;
  gap: var(--td-space-4);
}

.scan-form__depth {
  max-width: 10rem;
}
</style>
