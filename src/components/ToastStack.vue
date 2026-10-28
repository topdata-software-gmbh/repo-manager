<script setup lang="ts">
import { AlertCircle, CheckCircle2, Info, X } from 'lucide-vue-next';
import { useRepoStore } from '../stores/repoStore';

const store = useRepoStore();
</script>

<template>
  <div class="pointer-events-none fixed bottom-4 right-4 z-50 flex w-[24rem] flex-col gap-2">
    <div
      v-for="toast in store.toasts"
      :key="toast.id"
      class="pointer-events-auto flex items-start gap-2.5 rounded-lg border px-3.5 py-3 shadow-2xl backdrop-blur"
      :class="{
        'border-emerald-500/40 bg-emerald-950/90 text-emerald-100': toast.kind === 'success',
        'border-rose-500/40 bg-rose-950/90 text-rose-100': toast.kind === 'error',
        'border-sky-500/40 bg-sky-950/90 text-sky-100': toast.kind === 'info',
      }"
    >
      <CheckCircle2 v-if="toast.kind === 'success'" class="mt-0.5 h-4 w-4 shrink-0" />
      <AlertCircle v-else-if="toast.kind === 'error'" class="mt-0.5 h-4 w-4 shrink-0" />
      <Info v-else class="mt-0.5 h-4 w-4 shrink-0" />

      <div class="min-w-0 flex-1">
        <p class="text-xs font-semibold">{{ toast.title }}</p>
        <p
          v-if="toast.message"
          class="mt-1 whitespace-pre-line break-words font-mono text-[11px] leading-relaxed opacity-80"
        >
          {{ toast.message }}
        </p>
      </div>

      <button
        class="rounded p-0.5 opacity-60 transition-opacity hover:opacity-100"
        title="Dismiss"
        @click="store.dismissToast(toast.id)"
      >
        <X class="h-3.5 w-3.5" />
      </button>
    </div>
  </div>
</template>
