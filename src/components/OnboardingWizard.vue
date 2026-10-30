<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { X } from 'lucide-vue-next';
import { useRepoStore } from '../stores/repoStore';
import OnboardingStepChoose from './onboarding/OnboardingStepChoose.vue';
import OnboardingStepFind from './onboarding/OnboardingStepFind.vue';
import OnboardingStepGroups from './onboarding/OnboardingStepGroups.vue';
import OnboardingStepReady from './onboarding/OnboardingStepReady.vue';

const store = useRepoStore();
const heading = ref<HTMLElement | null>(null);

const steps = ['Find projects', 'Choose repos', 'Confirm groups', 'Ready'] as const;
const currentLabel = computed(() => steps[store.onboardingStep] ?? steps[0]);

onMounted(() => heading.value?.focus());

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') void store.closeOnboarding();
}
</script>

<template>
  <div
    class="fixed inset-0 z-50 flex flex-col overflow-y-auto bg-slate-950 text-slate-100"
    role="dialog"
    aria-modal="true"
    aria-labelledby="onboarding-title"
    @keydown="onKeydown"
  >
    <header class="border-b border-slate-800 px-6 py-4">
      <div class="mx-auto flex w-full max-w-4xl items-center justify-between gap-4">
        <div class="min-w-0">
          <p class="text-[11px] font-semibold uppercase tracking-widest text-cyan-400">
            Step {{ store.onboardingStep + 1 }} of 4 · {{ currentLabel }}
          </p>
          <h1
            id="onboarding-title"
            ref="heading"
            tabindex="-1"
            class="mt-1 text-xl font-semibold outline-none"
          >
            Let's set up your workspace
          </h1>
        </div>
        <button
          type="button"
          class="rounded-lg p-2 text-slate-400 hover:bg-slate-800 hover:text-slate-200"
          aria-label="Close setup and resume later"
          @click="store.closeOnboarding"
        >
          <X class="h-5 w-5" />
        </button>
      </div>
      <ol class="mx-auto mt-4 flex w-full max-w-4xl gap-2" aria-label="Setup progress">
        <li
          v-for="(step, index) in steps"
          :key="step"
          class="h-1 flex-1 rounded-full"
          :class="index <= store.onboardingStep ? 'bg-cyan-500' : 'bg-slate-800'"
        >
          <span class="sr-only">{{ step }}</span>
        </li>
      </ol>
    </header>

    <main class="mx-auto w-full max-w-4xl flex-1 px-6 py-8">
      <OnboardingStepFind v-if="store.onboardingStep === 0" />
      <OnboardingStepChoose v-else-if="store.onboardingStep === 1" />
      <OnboardingStepGroups v-else-if="store.onboardingStep === 2" />
      <OnboardingStepReady v-else />
    </main>
  </div>
</template>
