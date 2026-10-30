<script setup lang="ts">
import { computed } from 'vue';
import {
  AlertOctagon,
  AlertTriangle,
  ArrowDownToLine,
  CheckCircle2,
  Info,
  Save,
  Upload,
} from 'lucide-vue-next';
import { TdAlert, TdButton } from '@topdata-software-gmbh/ui-kit-web-vue3';
import { useRepoStore } from '../stores/repoStore';

const store = useRepoStore();

const icon = computed(() => {
  switch (store.insight?.tone) {
    case 'danger':
      return AlertOctagon;
    case 'warning':
      return AlertTriangle;
    case 'info':
      return Info;
    default:
      return CheckCircle2;
  }
});

const alertVariant = computed<'error' | 'warning' | 'info' | 'success'>(() => {
  switch (store.insight?.tone) {
    case 'danger':
      return 'error';
    case 'warning':
      return 'warning';
    case 'info':
      return 'info';
    default:
      return 'success';
  }
});

const actionIcon = computed(() => {
  switch (store.insight?.action) {
    case 'save':
      return Save;
    case 'pull':
      return ArrowDownToLine;
    case 'push':
      return Upload;
    default:
      return null;
  }
});

const actionBusy = computed(() => store.isLoading);

async function runAction(): Promise<void> {
  if (!store.insight?.action || actionBusy.value) return;

  if (store.insight.action === 'save') {
    store.openSaveChanges();
    return;
  }

  await store.executeGroupAction(store.insight.action);
}
</script>

<template>
  <div v-if="store.insight" class="border-b border-border px-4 py-3">
    <TdAlert :variant="alertVariant" :title="store.insight.title">
      <template #icon><component :is="icon" /></template>
      {{ store.insight.message }}
      <template #actions>
        <TdButton variant="ghost" size="sm" @click="store.openHelp">
          How this works
        </TdButton>
        <TdButton
          v-if="store.insight.action && actionIcon"
          variant="secondary"
          size="sm"
          :disabled="actionBusy"
          @click="runAction"
        >
          <template #icon-left><component :is="actionIcon" /></template>
          <span>{{ store.insight.actionLabel }}</span>
        </TdButton>
      </template>
    </TdAlert>
  </div>
</template>
