<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { subscribeToRepoChanges } from './lib/backend';
import { useRepoStore } from './stores/repoStore';
import AddScanPathModal from './components/AddScanPathModal.vue';
import GroupEditorModal from './components/GroupEditorModal.vue';
import RepoTable from './components/RepoTable.vue';
import Sidebar from './components/Sidebar.vue';
import ToastStack from './components/ToastStack.vue';
import Toolbar from './components/Toolbar.vue';

const store = useRepoStore();
let unsubscribe: (() => void) | null = null;

onMounted(async () => {
  await store.loadConfig();
  await store.scanRepositories();

  try {
    unsubscribe = await subscribeToRepoChanges((path) => {
      void store.refreshSingleRepo(path);
    });
  } catch (error) {
    store.showToast('error', 'Live file watching unavailable', String(error));
  }
});

onUnmounted(() => {
  unsubscribe?.();
});
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden bg-slate-900 font-sans text-slate-100">
    <Sidebar />
    <div class="flex h-full min-w-0 flex-1 flex-col overflow-hidden">
      <Toolbar />
      <RepoTable />
    </div>

    <GroupEditorModal />
    <AddScanPathModal />
    <ToastStack />
  </div>
</template>
