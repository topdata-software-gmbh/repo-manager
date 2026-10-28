<script setup lang="ts">
import {
  Eye,
  EyeOff,
  FolderGit2,
  FolderSearch,
  Layers,
  Loader2,
  Pencil,
  Plus,
  RefreshCw,
  X,
} from 'lucide-vue-next';
import { backendMode } from '../lib/backend';
import { useRepoStore } from '../stores/repoStore';
import type { RepoGroup } from '../types/domain';

const store = useRepoStore();

function repoCount(group: RepoGroup): number {
  return store.repositories.filter((repo) => group.repoPaths.includes(repo.path)).length;
}

async function removeScanPath(id: string, path: string): Promise<void> {
  if (
    !window.confirm(
      `Stop looking for projects in "${path}"? Your projects and files stay exactly where they are.`,
    )
  ) {
    return;
  }
  await store.removeScanPath(id);
}
</script>

<template>
  <aside class="flex w-72 shrink-0 flex-col border-r border-slate-800 bg-slate-950">
    <div class="flex-1 overflow-y-auto p-3">
      <div class="mb-3 flex items-center gap-2 px-2 py-2">
        <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-cyan-500/10">
          <FolderGit2 class="h-4 w-4 text-cyan-400" />
        </div>
        <div class="min-w-0">
          <p class="truncate text-sm font-bold tracking-wide text-slate-100">Git Repo Manager</p>
          <p class="text-[10px] uppercase tracking-widest text-slate-500">your project workspace</p>
        </div>
      </div>

      <nav class="space-y-1">
        <button
          class="flex w-full items-center justify-between rounded-md px-2.5 py-1.5 text-xs transition-colors"
          :class="
            store.selectedGroupId === 'all'
              ? 'bg-slate-800 font-semibold text-cyan-400'
              : 'text-slate-400 hover:bg-slate-900 hover:text-slate-200'
          "
          @click="store.selectGroup('all')"
        >
          <span class="flex items-center gap-2">
            <Layers class="h-3.5 w-3.5" />
            <span>All projects</span>
          </span>
          <span class="font-mono text-[10px] text-slate-500">{{ store.repositories.length }}</span>
        </button>

        <div
          class="flex items-center justify-between px-2.5 pb-1 pt-4 text-[11px] font-semibold uppercase tracking-wider text-slate-500"
        >
          <span>Project groups</span>
          <button
            class="rounded p-0.5 transition-colors hover:bg-slate-800 hover:text-slate-200"
            title="New project group"
            @click="store.openGroupEditor(null)"
          >
            <Plus class="h-3.5 w-3.5" />
          </button>
        </div>

        <p v-if="store.config.groups.length === 0" class="px-2.5 py-1 text-[11px] leading-relaxed text-slate-600">
          Groups let you save or update several projects at once.
        </p>

        <div
          v-for="group in store.config.groups"
          :key="group.id"
          class="group flex items-center gap-1 rounded-md pr-1 transition-colors"
          :class="store.selectedGroupId === group.id ? 'bg-slate-800' : 'hover:bg-slate-900'"
        >
          <button
            class="flex min-w-0 flex-1 items-center justify-between px-2.5 py-1.5 text-xs transition-colors"
            :class="
              store.selectedGroupId === group.id
                ? 'font-semibold text-cyan-400'
                : 'text-slate-400 hover:text-slate-200'
            "
            @click="store.selectGroup(group.id)"
          >
            <span class="flex min-w-0 items-center gap-2">
              <span
                class="h-2 w-2 shrink-0 rounded-full"
                :style="{ backgroundColor: group.color }"
              ></span>
              <span class="truncate">{{ group.name }}</span>
            </span>
            <span class="ml-2 shrink-0 font-mono text-[10px] text-slate-500">
              {{ repoCount(group) }}
            </span>
          </button>
          <button
            class="rounded p-1 text-slate-600 opacity-0 transition-all hover:bg-slate-700 hover:text-slate-200 group-hover:opacity-100"
            title="Edit project group"
            @click.stop="store.openGroupEditor(group)"
          >
            <Pencil class="h-3 w-3" />
          </button>
        </div>

        <div
          class="flex items-center justify-between px-2.5 pb-1 pt-4 text-[11px] font-semibold uppercase tracking-wider text-slate-500"
        >
          <span>Project folders</span>
          <button
            class="rounded p-0.5 transition-colors hover:bg-slate-800 hover:text-slate-200"
            title="Add project folder"
            @click="store.openAddScanPath"
          >
            <Plus class="h-3.5 w-3.5" />
          </button>
        </div>

        <p
          v-if="store.config.scanPaths.length === 0"
          class="px-2.5 py-1 text-[11px] leading-relaxed text-slate-600"
        >
          Add the folder where your projects live. We will find the Git projects inside it.
        </p>

        <div
          v-for="scanPath in store.config.scanPaths"
          :key="scanPath.id"
          class="group flex items-center gap-1 rounded-md px-1.5 py-1 transition-colors hover:bg-slate-900"
        >
          <button
            class="rounded p-0.5 transition-colors hover:bg-slate-800"
            :class="scanPath.enabled ? 'text-cyan-500' : 'text-slate-600'"
            :title="scanPath.enabled ? 'Stop looking in this folder' : 'Look in this folder again'"
            @click="store.toggleScanPath(scanPath)"
          >
            <Eye v-if="scanPath.enabled" class="h-3.5 w-3.5" />
            <EyeOff v-else class="h-3.5 w-3.5" />
          </button>
          <span
            class="min-w-0 flex-1 truncate font-mono text-[10px]"
            :class="scanPath.enabled ? 'text-slate-400' : 'text-slate-600 line-through'"
            :title="scanPath.path"
          >
            {{ scanPath.path }}
          </span>
          <button
            class="rounded p-0.5 text-slate-600 opacity-0 transition-all hover:bg-slate-800 hover:text-rose-400 group-hover:opacity-100"
            title="Remove this folder from the list"
            @click="removeScanPath(scanPath.id, scanPath.path)"
          >
            <X class="h-3 w-3" />
          </button>
        </div>
      </nav>
    </div>

    <div class="border-t border-slate-800/80 p-3">
      <button
        class="flex w-full items-center justify-center gap-1.5 rounded-md border border-slate-800 bg-slate-900 px-3 py-2 text-xs text-slate-300 transition-colors hover:bg-slate-800 disabled:opacity-50"
        :disabled="store.isScanning"
        @click="store.scanRepositories"
      >
        <Loader2 v-if="store.isScanning" class="h-3.5 w-3.5 animate-spin" />
        <RefreshCw v-else class="h-3.5 w-3.5" />
        <span>Look for new projects</span>
      </button>
      <button
        class="mt-2 flex w-full items-center justify-center gap-1.5 rounded-md px-3 py-1.5 text-[11px] text-slate-500 transition-colors hover:bg-slate-900 hover:text-slate-300"
        @click="store.openAddScanPath"
      >
        <FolderSearch class="h-3 w-3" />
        <span>Add project folder</span>
      </button>
      <p class="mt-2 text-center text-[10px] uppercase tracking-widest text-slate-700">
        {{ backendMode === 'web' ? 'Web browser version' : 'Desktop app' }}
      </p>
    </div>
  </aside>
</template>
