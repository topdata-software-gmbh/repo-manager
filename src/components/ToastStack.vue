<script setup lang="ts">
import { computed, onBeforeUnmount, watch } from 'vue';
import { TdToastStack } from '@topdata-software-gmbh/ui';
import { useRepoStore } from '../stores/repoStore';

const store = useRepoStore();
const maxVisible = 3;

interface TimerState {
  timeout: ReturnType<typeof setTimeout> | null;
  remaining: number;
  startedAt: number;
  paused: boolean;
}

const timers = new Map<number, TimerState>();

const visibleToasts = computed(() => store.toasts.slice(0, maxVisible));
const visibleKey = computed(() =>
  visibleToasts.value.map((toast) => `${toast.id}:${toast.kind}`).join('|'),
);

function clearTimer(id: number): void {
  const state = timers.get(id);
  if (!state) return;
  if (state.timeout) clearTimeout(state.timeout);
  timers.delete(id);
}

function startTimer(id: number): void {
  if (timers.has(id)) return;
  const state: TimerState = {
    timeout: null,
    remaining: 5000,
    startedAt: Date.now(),
    paused: false,
  };
  timers.set(id, state);
  state.timeout = setTimeout(() => {
    timers.delete(id);
    store.dismissToast(id);
    syncTimers();
  }, 5000);
}

function syncTimers(): void {
  const visibleIds = new Set(visibleToasts.value.map((toast) => toast.id));
  for (const id of [...timers.keys()]) {
    if (!visibleIds.has(id) || !store.toasts.some((toast) => toast.id === id)) {
      clearTimer(id);
    }
  }
  for (const toast of visibleToasts.value) {
    if (toast.kind !== 'error') startTimer(toast.id);
  }
}

function dismiss(id: number): void {
  clearTimer(id);
  store.dismissToast(id);
  syncTimers();
}

function pause(id: number): void {
  const state = timers.get(id);
  if (!state || state.paused) return;
  state.remaining = Math.max(0, state.remaining - (Date.now() - state.startedAt));
  if (state.timeout) clearTimeout(state.timeout);
  state.timeout = null;
  state.paused = true;
}

function resume(id: number): void {
  const state = timers.get(id);
  if (!state || !state.paused) return;
  state.paused = false;
  state.startedAt = Date.now();
  state.timeout = setTimeout(() => {
    timers.delete(id);
    store.dismissToast(id);
    syncTimers();
  }, state.remaining);
}

watch(visibleKey, syncTimers, { immediate: true });

onBeforeUnmount(() => {
  for (const id of [...timers.keys()]) clearTimer(id);
});
</script>

<template>
  <TdToastStack
    :toasts="store.toasts"
    :max-visible="maxVisible"
    @dismiss="dismiss"
    @pause="pause"
    @resume="resume"
  />
</template>
