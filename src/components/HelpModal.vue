<script setup lang="ts">
import {
  Archive,
  BookOpen,
  Download,
  GitBranch,
  Save,
  Upload,
  X,
} from 'lucide-vue-next';
import { useRepoStore } from '../stores/repoStore';

const store = useRepoStore();

const steps = [
  {
    title: 'Add your project folder once',
    text: 'Choose the folder where your themes and projects live. The app finds the Git projects inside it.',
  },
  {
    title: 'Edit your files like you normally do',
    text: 'Use your editor. The app watches each project and shows when something changed.',
  },
  {
    title: 'Save to Git when you reach a checkpoint',
    text: 'A checkpoint is a commit: a snapshot of your work that you can return to later.',
  },
  {
    title: 'Get latest, then send changes',
    text: 'Get latest before you start working. Send your saved changes when you are finished.',
  },
];

const words = [
  {
    icon: Save,
    name: 'Save to Git',
    text: 'Creates a checkpoint (commit) of your changes in that project. Your files stay in the same folder.',
  },
  {
    icon: Download,
    name: 'Get latest',
    text: 'Downloads new work from the server and updates your project folder. Local changes are set aside and put back afterwards.',
  },
  {
    icon: Upload,
    name: 'Send changes',
    text: 'Uploads your saved checkpoints so other people can see them.',
  },
  {
    icon: GitBranch,
    name: 'Branch',
    text: 'A separate line of work. Switching branches changes the files in your project folder to that version.',
  },
  {
    icon: Archive,
    name: 'Set aside changes',
    text: 'Temporarily stores unfinished changes (a stash) so the folder is clean. Bring them back when you need them.',
  },
];
</script>

<template>
  <div
    v-if="store.helpModalOpen"
    class="fixed inset-0 z-40 flex items-center justify-center bg-slate-950/70 p-6 backdrop-blur-sm"
    @click.self="store.closeHelp"
  >
    <div
      class="flex max-h-[88vh] w-full max-w-2xl flex-col rounded-xl border border-slate-700 bg-slate-900 shadow-2xl"
    >
      <header class="flex items-center justify-between border-b border-slate-800 px-5 py-4">
        <div class="flex items-center gap-2">
          <BookOpen class="h-4 w-4 text-cyan-400" />
          <div>
            <h2 class="text-sm font-semibold text-slate-100">How this works</h2>
            <p class="text-[11px] text-slate-500">
              A short guide to working with Git without the jargon.
            </p>
          </div>
        </div>
        <button
          type="button"
          class="rounded p-1 text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-200"
          @click="store.closeHelp"
        >
          <X class="h-4 w-4" />
        </button>
      </header>

      <div class="flex-1 space-y-6 overflow-auto px-5 py-5">
        <ol class="space-y-3">
          <li v-for="(step, index) in steps" :key="step.title" class="flex gap-3">
            <span
              class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-cyan-500/15 text-[11px] font-semibold text-cyan-300"
            >
              {{ index + 1 }}
            </span>
            <div>
              <p class="text-sm font-medium text-slate-100">{{ step.title }}</p>
              <p class="mt-0.5 text-xs leading-relaxed text-slate-400">{{ step.text }}</p>
            </div>
          </li>
        </ol>

        <div>
          <h3 class="mb-2 text-[11px] font-semibold uppercase tracking-wider text-slate-500">
            Git words in plain English
          </h3>
          <div class="space-y-2">
            <div
              v-for="word in words"
              :key="word.name"
              class="flex gap-3 rounded-lg border border-slate-800 bg-slate-950/60 px-3 py-2.5"
            >
              <component :is="word.icon" class="mt-0.5 h-4 w-4 shrink-0 text-cyan-400" />
              <div>
                <p class="text-xs font-medium text-slate-200">{{ word.name }}</p>
                <p class="text-[11px] leading-relaxed text-slate-500">{{ word.text }}</p>
              </div>
            </div>
          </div>
        </div>

        <div class="rounded-lg border border-amber-500/25 bg-amber-500/10 px-4 py-3">
          <p class="text-xs font-medium text-amber-200">If something goes wrong</p>
          <p class="mt-1 text-[11px] leading-relaxed text-amber-100/80">
            Read the message at the bottom of the screen. It always explains the next step in plain
            language. Your work is not deleted by the buttons in this app.
          </p>
        </div>
      </div>

      <footer class="flex justify-end border-t border-slate-800 px-5 py-3">
        <button
          class="rounded-lg bg-cyan-500 px-3.5 py-1.5 text-xs font-semibold text-slate-950 transition-colors hover:bg-cyan-400"
          @click="store.closeHelp"
        >
          Got it
        </button>
      </footer>
    </div>
  </div>
</template>
