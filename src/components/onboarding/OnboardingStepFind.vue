<script setup lang="ts">
import { ref } from 'vue';
import { FolderSearch, Loader2 } from 'lucide-vue-next';
import { useRepoStore } from '../../stores/repoStore';

const store = useRepoStore();
const submitting = ref(false);

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
      <h2 class="text-lg font-semibold text-slate-100">Where do your projects live?</h2>
      <p class="mt-1.5 text-sm leading-relaxed text-slate-400">
        We look inside this folder for Git projects and skip dependency and build folders such as
        node_modules, vendor, and dist. You can add more folders later.
      </p>
    </div>

    <div class="space-y-4 rounded-xl border border-slate-800 bg-slate-900/60 p-5">
      <div>
        <label for="onboarding-scan-path" class="mb-1.5 block text-xs font-medium text-slate-300">
          Project folder
        </label>
        <input
          id="onboarding-scan-path"
          v-model="store.onboardingDraft.scanPath"
          type="text"
          spellcheck="false"
          placeholder="~/devel"
          class="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 outline-none transition-colors placeholder:text-slate-600 focus:border-cyan-500"
        />
      </div>

      <div>
        <label for="onboarding-owner-names" class="mb-1.5 block text-xs font-medium text-slate-300">
          Remote owner names <span class="font-normal text-slate-500">(optional)</span>
        </label>
        <input
          id="onboarding-owner-names"
          v-model="store.onboardingDraft.ownerNames"
          type="text"
          spellcheck="false"
          placeholder="acme, my-company"
          class="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 outline-none transition-colors placeholder:text-slate-600 focus:border-cyan-500"
        />
        <p class="mt-1.5 text-[11px] leading-relaxed text-slate-500">
          Used only to pre-select projects you own. This stays on this computer.
        </p>
      </div>
    </div>

    <details
      class="rounded-xl border border-slate-800 bg-slate-900/40 px-5 py-4"
    >
      <summary class="cursor-pointer text-xs font-medium text-slate-300">Advanced</summary>
      <div class="mt-4 grid gap-4 sm:grid-cols-2">
        <div>
          <label for="onboarding-max-depth" class="mb-1.5 block text-xs font-medium text-slate-300">
            How deep to look
          </label>
          <input
            id="onboarding-max-depth"
            v-model.number="store.onboardingDraft.maxDepth"
            type="number"
            min="1"
            max="12"
            class="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 outline-none focus:border-cyan-500"
          />
          <p class="mt-1.5 text-[11px] text-slate-500">
            Nested clones can sit several folders deep. The default of 8 finds most setups.
          </p>
        </div>

        <div>
          <label
            for="onboarding-grouping-style"
            class="mb-1.5 block text-xs font-medium text-slate-300"
          >
            How to suggest groups
          </label>
          <select
            id="onboarding-grouping-style"
            v-model="store.onboardingDraft.groupingStyle"
            class="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 outline-none focus:border-cyan-500"
          >
            <option value="topLevel">By top-level folder</option>
            <option value="single">One group for everything</option>
          </select>
          <p class="mt-1.5 text-[11px] text-slate-500">
            You can rename, skip, or change group membership later.
          </p>
        </div>
      </div>
    </details>

    <p v-if="store.onboardingError" role="alert" class="text-sm text-rose-400">
      {{ store.onboardingError }}
    </p>

    <p v-if="store.isScanning || submitting" class="text-sm text-slate-400" role="status">
      Looking for Git projects. This can take a moment in large folders…
    </p>

    <div class="flex items-center gap-3">
      <button
        type="submit"
        class="flex items-center gap-2 rounded-lg bg-cyan-500 px-4 py-2 text-sm font-semibold text-slate-950 transition-colors hover:bg-cyan-400 disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="submitting || store.isScanning"
      >
        <Loader2 v-if="submitting || store.isScanning" class="h-4 w-4 animate-spin" />
        <FolderSearch v-else class="h-4 w-4" />
        <span>Find my projects</span>
      </button>
    </div>
  </form>
</template>
