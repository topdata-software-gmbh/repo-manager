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

const toneClasses = computed(() => {
  switch (store.insight?.tone) {
    case 'danger':
      return 'border-rose-500/30 bg-rose-500/10 text-rose-200';
    case 'warning':
      return 'border-amber-500/30 bg-amber-500/10 text-amber-100';
    case 'info':
      return 'border-sky-500/30 bg-sky-500/10 text-sky-100';
    default:
      return 'border-emerald-500/25 bg-emerald-500/10 text-emerald-100';
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
  <div v-if="store.insight" class="border-b border-slate-800/70 px-4 py-3">
    <div
      class="flex flex-col gap-3 rounded-xl border px-4 py-3 sm:flex-row sm:items-center"
      :class="toneClasses"
      aria-live="polite"
    >
      <component :is="icon" class="h-5 w-5 shrink-0" />

      <div class="min-w-0 flex-1">
        <p class="text-sm font-semibold text-slate-50">{{ store.insight.title }}</p>
        <p class="mt-0.5 text-xs leading-relaxed text-slate-300">
          {{ store.insight.message }}
        </p>
      </div>

      <div class="flex shrink-0 items-center gap-2">
        <button
          class="rounded-lg px-2.5 py-1.5 text-xs text-slate-300 transition-colors hover:bg-white/10 hover:text-white"
          @click="store.openHelp"
        >
          How this works
        </button>
        <button
          v-if="store.insight.action && actionIcon"
          class="flex items-center gap-1.5 rounded-lg bg-white/10 px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-white/20 disabled:cursor-not-allowed disabled:opacity-50"
          :disabled="actionBusy"
          @click="runAction"
        >
          <component :is="actionIcon" class="h-3.5 w-3.5" />
          <span>{{ store.insight.actionLabel }}</span>
        </button>
      </div>
    </div>
  </div>
</template>
