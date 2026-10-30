<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { Trash2 } from 'lucide-vue-next';
import {
  TdAlert,
  TdButton,
  TdCheckbox,
  TdFormField,
  TdInput,
  TdModal,
} from '@topdata-software-gmbh/ui-kit-web-vue3';
import { useRepoStore } from '../stores/repoStore';
import { GROUP_COLORS, groupColorStyle } from '../lib/groupDefaults';
import type { RepoGroup } from '../types/domain';

const store = useRepoStore();
const name = ref('');
const color = ref<string>(GROUP_COLORS[0]);
const repoPaths = ref<string[]>([]);
const syncBranching = ref(true);
const autoStashOnSwitch = ref(true);
const defaultBaseBranch = ref('main');
const submitting = ref(false);
const deleting = ref(false);
const error = ref('');

const isEditing = computed(() => store.editingGroup !== null);
const allSelected = computed(
  () =>
    store.repositories.length > 0 &&
    store.repositories.every((repo) => repoPaths.value.includes(repo.path)),
);

watch(
  () => store.groupEditorOpen,
  (open) => {
    if (!open) return;
    const group = store.editingGroup;
    name.value = group?.name ?? '';
    color.value = group?.color ?? GROUP_COLORS[0];
    repoPaths.value = group ? [...group.repoPaths] : [];
    syncBranching.value = group?.settings.syncBranching ?? true;
    autoStashOnSwitch.value = group?.settings.autoStashOnSwitch ?? true;
    defaultBaseBranch.value = group?.settings.defaultBaseBranch ?? 'main';
    submitting.value = false;
    deleting.value = false;
    error.value = '';
  },
);

function toggleRepository(path: string): void {
  repoPaths.value = repoPaths.value.includes(path)
    ? repoPaths.value.filter((existing) => existing !== path)
    : [...repoPaths.value, path];
}

function toggleAll(): void {
  repoPaths.value = allSelected.value ? [] : store.repositories.map((repo) => repo.path);
}

function newId(): string {
  return `grp_${typeof crypto.randomUUID === 'function' ? crypto.randomUUID() : Date.now()}`;
}

async function submit(): Promise<void> {
  const trimmed = name.value.trim();
  if (!trimmed) {
    error.value = 'Give the group a name.';
    return;
  }
  if (submitting.value) return;

  error.value = '';
  submitting.value = true;
  const group: RepoGroup = {
    id: store.editingGroup?.id ?? newId(),
    name: trimmed,
    color: color.value,
    repoPaths: [...repoPaths.value],
    settings: {
      syncBranching: syncBranching.value,
      autoStashOnSwitch: autoStashOnSwitch.value,
      defaultBaseBranch: defaultBaseBranch.value.trim() || 'main',
    },
  };

  try {
    await store.saveGroup(group);
    store.closeGroupEditor();
  } finally {
    submitting.value = false;
  }
}

async function remove(): Promise<void> {
  const group = store.editingGroup;
  if (!group || deleting.value) return;
  if (!window.confirm(`Delete group "${group.name}"? Your projects and files are not touched.`)) {
    return;
  }

  deleting.value = true;
  try {
    await store.deleteGroup(group.id);
    store.closeGroupEditor();
  } finally {
    deleting.value = false;
  }
}
</script>

<template>
  <TdModal
    :open="store.groupEditorOpen"
    :title="isEditing ? 'Edit project group' : 'New project group'"
    description="Name the group and choose the projects it contains."
    size="lg"
    @close="store.closeGroupEditor"
  >
    <form id="group-editor-form" class="space-y-4" @submit.prevent="submit">
      <div class="grid grid-cols-[1fr_auto] gap-4">
        <TdFormField label="Group name" required>
          <template #default="{ id, required }">
            <TdInput :id="id" v-model="name" :required="required" placeholder="Theme projects" autofocus />
          </template>
        </TdFormField>

        <div>
          <span class="mb-1.5 block text-xs font-medium text-fg">Color</span>
          <div class="flex items-center gap-1.5 pt-1">
            <TdButton
              v-for="swatch in GROUP_COLORS"
              :key="swatch"
              variant="ghost"
              class="h-5 w-5 rounded-full border-2 p-0 transition-transform hover:scale-110"
              :class="color === swatch ? 'border-fg' : 'border-transparent'"
              :style="groupColorStyle(swatch)"
              :title="swatch"
              :aria-label="`Use ${swatch} for this group`"
              :aria-pressed="color === swatch"
              @click="color = swatch"
            />
          </div>
        </div>
      </div>

      <div>
        <div class="mb-1.5 flex items-center justify-between">
          <span class="text-xs font-medium text-fg">Projects</span>
          <TdButton
            v-if="store.repositories.length > 0"
            variant="ghost"
            size="sm"
            @click="toggleAll"
          >
            {{ allSelected ? 'Clear all' : 'Select all' }}
          </TdButton>
        </div>

        <p
          v-if="store.repositories.length === 0"
          class="rounded-lg border border-dashed border-border px-4 py-6 text-center text-xs text-fg-muted"
        >
          No projects yet. Add the folder where your projects live, then look for projects.
        </p>
        <div
          v-else
          class="max-h-56 divide-y divide-border overflow-auto rounded-lg border border-border bg-surface"
        >
          <TdCheckbox
            v-for="repo in store.repositories"
            :key="repo.id"
            :model-value="repoPaths.includes(repo.path)"
            class="w-full px-3 py-2 hover:bg-surface-hover"
            @update:model-value="toggleRepository(repo.path)"
          >
            <span class="flex min-w-0 flex-1 items-center gap-3">
              <span class="min-w-0 flex-1">
                <span class="block truncate text-xs text-fg">{{ repo.name }}</span>
                <span class="block truncate font-mono text-xs text-fg-muted">{{ repo.path }}</span>
              </span>
              <span class="shrink-0 font-mono text-xs text-primary">{{ repo.state.currentBranch }}</span>
            </span>
          </TdCheckbox>
        </div>
        <p class="mt-1.5 text-xs text-fg-muted">
          {{ repoPaths.length }} of {{ store.repositories.length }} projects selected
        </p>
      </div>

      <details class="rounded-lg border border-border bg-surface px-4 py-3">
        <summary class="cursor-pointer text-xs font-medium text-fg">Advanced settings</summary>
        <div class="mt-3 grid grid-cols-2 gap-3">
          <TdCheckbox
            v-model="autoStashOnSwitch"
            label="Keep unsaved changes when switching branches"
            description="Changes are set aside and put back automatically."
          />
          <TdCheckbox
            v-model="syncBranching"
            label="Show the branch chooser for the group"
            description="Lets you switch all projects in this group together."
          />
        </div>
        <div class="mt-3 w-52">
          <TdFormField label="Starting branch for new work">
            <template #default="{ id }">
              <TdInput :id="id" v-model="defaultBaseBranch" placeholder="main" />
            </template>
          </TdFormField>
        </div>
      </details>

      <TdAlert v-if="error" variant="error">{{ error }}</TdAlert>
    </form>

    <template #footer>
      <div class="flex w-full items-center justify-between">
        <TdButton
          v-if="isEditing"
          variant="danger"
          :loading="deleting"
          @click="remove"
        >
          <template #icon-left><Trash2 /></template>
          Delete group
        </TdButton>
        <span v-else />
        <div class="flex items-center gap-2">
          <TdButton variant="ghost" @click="store.closeGroupEditor">Cancel</TdButton>
          <TdButton
            type="submit"
            form="group-editor-form"
            variant="primary"
            :loading="submitting"
          >
            {{ isEditing ? 'Save changes' : 'Create group' }}
          </TdButton>
        </div>
      </div>
    </template>
  </TdModal>
</template>
