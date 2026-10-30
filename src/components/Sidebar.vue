<script setup lang="ts">
import {
  Eye,
  EyeOff,
  FolderGit2,
  FolderSearch,
  Layers,
  Pencil,
  Plus,
  RefreshCw,
  Sparkles,
  X,
} from 'lucide-vue-next';
import { TdBadge, TdButton, TdSpinner } from '@topdata-software-gmbh/ui-kit-web-vue3';
import { backendMode } from '../lib/backend';
import { groupColorStyle } from '../lib/groupDefaults';
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
  <aside class="flex w-72 shrink-0 flex-col border-r border-border bg-canvas">
    <div class="flex-1 overflow-y-auto p-3">
      <div class="mb-3 flex items-center gap-2 px-2 py-2">
        <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-primary/10">
          <FolderGit2 class="h-4 w-4 text-primary" />
        </div>
        <div class="min-w-0">
          <p class="truncate text-sm font-bold tracking-wide text-fg">Git Repo Manager</p>
          <p class="text-[10px] uppercase tracking-widest text-fg-muted">your project workspace</p>
        </div>
      </div>

      <nav class="space-y-1">
        <TdButton
          variant="ghost"
          block
          class="justify-between px-2.5 py-1.5 text-xs"
          :class="
            store.selectedGroupId === 'all'
              ? 'bg-surface-raised font-semibold text-primary'
              : 'text-fg-muted hover:bg-surface hover:text-fg'
          "
          @click="store.selectGroup('all')"
        >
          <template #icon-left><Layers /></template>
          <span class="flex w-full items-center justify-between">
            <span>All repos</span>
            <TdBadge variant="neutral">{{ store.repositories.length }}</TdBadge>
          </span>
        </TdButton>

        <div
          class="flex items-center justify-between px-2.5 pb-1 pt-4 text-[11px] font-semibold uppercase tracking-wider text-fg-muted"
        >
          <span>Project groups</span>
          <TdButton
            variant="ghost"
            size="sm"
            class="p-0.5"
            title="New project group"
            aria-label="New project group"
            @click="store.openGroupEditor(null)"
          >
            <template #icon-left><Plus /></template>
          </TdButton>
        </div>

        <p
          v-if="store.config.groups.length === 0"
          class="px-2.5 py-1 text-[11px] leading-relaxed text-fg-dim"
        >
          Groups let you save or update several projects at once.
        </p>

        <div
          v-for="group in store.config.groups"
          :key="group.id"
          class="group flex items-center gap-1 rounded-md pr-1 transition-colors"
          :class="store.selectedGroupId === group.id ? 'bg-surface-raised' : 'hover:bg-surface'"
        >
          <TdButton
            variant="ghost"
            block
            class="min-w-0 flex-1 justify-between px-2.5 py-1.5 text-xs"
            :class="
              store.selectedGroupId === group.id
                ? 'font-semibold text-primary'
                : 'text-fg-muted hover:text-fg'
            "
            @click="store.selectGroup(group.id)"
          >
            <span class="flex min-w-0 items-center gap-2">
              <span
                class="h-2 w-2 shrink-0 rounded-full"
                :style="groupColorStyle(group.color)"
              ></span>
              <span class="truncate">{{ group.name }}</span>
            </span>
            <TdBadge variant="neutral">{{ repoCount(group) }}</TdBadge>
          </TdButton>
          <TdButton
            variant="ghost"
            size="sm"
            class="p-1 opacity-0 transition-all group-hover:opacity-100"
            title="Edit project group"
            aria-label="Edit project group"
            @click.stop="store.openGroupEditor(group)"
          >
            <template #icon-left><Pencil /></template>
          </TdButton>
        </div>

        <div
          class="flex items-center justify-between px-2.5 pb-1 pt-4 text-[11px] font-semibold uppercase tracking-wider text-fg-muted"
        >
          <span>Project folders</span>
          <TdButton
            variant="ghost"
            size="sm"
            class="p-0.5"
            title="Add project folder"
            aria-label="Add project folder"
            @click="store.openAddScanPath"
          >
            <template #icon-left><Plus /></template>
          </TdButton>
        </div>

        <p
          v-if="store.config.scanPaths.length === 0"
          class="px-2.5 py-1 text-[11px] leading-relaxed text-fg-dim"
        >
          Add the folder where your projects live. We will find the Git projects inside it.
        </p>

        <div
          v-for="scanPath in store.config.scanPaths"
          :key="scanPath.id"
          class="group flex items-center gap-1 rounded-md px-1.5 py-1 transition-colors hover:bg-surface"
        >
          <TdButton
            variant="ghost"
            size="sm"
            class="p-0.5"
            :class="scanPath.enabled ? 'text-primary' : 'text-fg-dim'"
            :title="scanPath.enabled ? 'Stop looking in this folder' : 'Look in this folder again'"
            :aria-label="
              scanPath.enabled ? 'Stop looking in this folder' : 'Look in this folder again'
            "
            @click="store.toggleScanPath(scanPath)"
          >
            <template #icon-left>
              <Eye v-if="scanPath.enabled" />
              <EyeOff v-else />
            </template>
          </TdButton>
          <span
            class="min-w-0 flex-1 truncate font-mono text-[10px]"
            :class="scanPath.enabled ? 'text-fg-muted' : 'text-fg-dim line-through'"
            :title="scanPath.path"
          >
            {{ scanPath.path }}
          </span>
          <TdButton
            variant="ghost"
            size="sm"
            class="p-0.5 text-fg-dim opacity-0 transition-all hover:text-error-fg group-hover:opacity-100"
            title="Remove this folder from the list"
            aria-label="Remove this folder from the list"
            @click="removeScanPath(scanPath.id, scanPath.path)"
          >
            <template #icon-left><X /></template>
          </TdButton>
        </div>
      </nav>
    </div>

    <div class="border-t border-border p-3">
      <TdButton
        variant="secondary"
        block
        class="text-xs"
        :disabled="store.isScanning"
        @click="store.scanRepositories"
      >
        <template #icon-left>
          <TdSpinner v-if="store.isScanning" size="sm" />
          <RefreshCw v-else />
        </template>
        <span>Look for new projects</span>
      </TdButton>
      <TdButton
        variant="ghost"
        block
        size="sm"
        class="mt-2 py-1.5 text-[11px]"
        title="Reopen the setup wizard"
        @click="store.openOnboarding(true)"
      >
        <template #icon-left><Sparkles /></template>
        <span>Setup</span>
      </TdButton>
      <TdButton
        variant="ghost"
        block
        size="sm"
        class="mt-2 py-1.5 text-[11px]"
        @click="store.openAddScanPath"
      >
        <template #icon-left><FolderSearch /></template>
        <span>Add project folder</span>
      </TdButton>
      <p class="mt-2 text-center text-[10px] uppercase tracking-widest text-fg-dim">
        {{ backendMode === 'web' ? 'Web browser version' : 'Desktop app' }}
      </p>
    </div>
  </aside>
</template>
