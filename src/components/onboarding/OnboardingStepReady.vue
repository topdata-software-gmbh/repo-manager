<script setup lang="ts">
import { computed, ref } from 'vue';
import { ArrowRight, CheckCircle2, FolderSearch } from 'lucide-vue-next';
import { useRepoStore } from '../../stores/repoStore';

const store = useRepoStore();
const busy = ref(false);

const groups = computed(() => store.onboardingPendingGroups);
const firstGroup = computed(() => groups.value[0] ?? null);

async function open(groupId: string | null): Promise<void> {
  if (busy.value) return;
  busy.value = true;
  try {
    await store.completeOnboarding(groupId);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-start gap-3">
      <CheckCircle2 class="mt-1 h-6 w-6 shrink-0 text-emerald-400" />
      <div>
        <h2 class="text-lg font-semibold text-slate-100">You're ready to go</h2>
        <p class="mt-1.5 text-sm leading-relaxed text-slate-400">
          {{ store.onboardingSelectedRepositories.length }}
          {{ store.onboardingSelectedRepositories.length === 1 ? 'project' : 'projects' }} accepted
          and {{ groups.length }} {{ groups.length === 1 ? 'group' : 'groups' }} ready.
        </p>
      </div>
    </div>

    <div class="grid gap-3 sm:grid-cols-3">
      <div class="rounded-xl border border-slate-800 bg-slate-900/60 p-4">
        <h3 class="text-sm font-semibold text-slate-100">Save to Git</h3>
        <p class="mt-1.5 text-xs leading-relaxed text-slate-400">
          Saves a checkpoint of your changed files. Your files stay where they are.
        </p>
      </div>
      <div class="rounded-xl border border-slate-800 bg-slate-900/60 p-4">
        <h3 class="text-sm font-semibold text-slate-100">Get latest</h3>
        <p class="mt-1.5 text-xs leading-relaxed text-slate-400">
          Brings new work from the server into your project folders.
        </p>
      </div>
      <div class="rounded-xl border border-slate-800 bg-slate-900/60 p-4">
        <h3 class="text-sm font-semibold text-slate-100">Send changes</h3>
        <p class="mt-1.5 text-xs leading-relaxed text-slate-400">
          Uploads your saved checkpoints so others can see them.
        </p>
      </div>
    </div>

    <p v-if="store.onboardingError" role="alert" class="text-sm text-rose-400">
      {{ store.onboardingError }}
    </p>

    <div class="space-y-2 rounded-xl border border-slate-800 bg-slate-950 p-4">
      <p class="text-xs font-medium uppercase tracking-wider text-slate-500">Open a workspace</p>
      <div class="flex flex-wrap items-center gap-2">
        <button
          v-if="firstGroup"
          type="button"
          class="flex items-center gap-2 rounded-lg bg-cyan-500 px-4 py-2 text-sm font-semibold text-slate-950 transition-colors hover:bg-cyan-400 disabled:cursor-not-allowed disabled:opacity-50"
          :disabled="busy"
          @click="open(firstGroup.id)"
        >
          Open {{ firstGroup.name }}
          <ArrowRight class="h-4 w-4" />
        </button>
        <button
          type="button"
          class="flex items-center gap-2 rounded-lg bg-slate-800 px-4 py-2 text-sm text-slate-200 transition-colors hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-50"
          :disabled="busy"
          @click="open(null)"
        >
          <FolderSearch class="h-4 w-4" />
          Open All repos
        </button>
      </div>

      <div v-if="groups.length > 1" class="flex flex-wrap gap-2 pt-1">
        <button
          v-for="group in groups.slice(1)"
          :key="group.id"
          type="button"
          class="rounded-lg border border-slate-800 px-3 py-1.5 text-xs text-slate-300 transition-colors hover:bg-slate-800 disabled:opacity-50"
          :disabled="busy"
          @click="open(group.id)"
        >
          {{ group.name }}
        </button>
      </div>
    </div>
  </div>
</template>
