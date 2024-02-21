<script>
  import '../app.pcss';
  import '@fontsource/dm-sans/200.css';
  import '@fontsource/dm-sans/300.css';
  import '@fontsource/dm-sans/400.css';
  import '@fontsource/dm-sans/500.css';
  import '@fontsource/dm-sans/600.css';
  import '@fontsource/dm-sans/800.css';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import { toastsList } from '$lib/stores/toasts';
  import AppToast from '$lib/components/AppToast.svelte';
  import { transactionStatusNoticesList } from '$lib/stores/transactionStatusNotice';
  import TransactionStatusNotice from '$lib/components/TransactionStatusNotice.svelte';
  import WindowDraggableArea from '$lib/components/WindowDraggableArea.svelte';
  import { redirectIfSettingsNotDefined } from '$lib/utils/redirect';

  redirectIfSettingsNotDefined();
</script>

<WindowDraggableArea />

<div class="flex min-h-screen w-full justify-start bg-white dark:bg-gray-900">
  <Sidebar />

  <main class="ml-64 h-full w-full grow overflow-x-auto p-8">
    <slot />
  </main>

  <div class="fixed right-5 top-5 z-50 w-full max-w-md">
    {#each $transactionStatusNoticesList as transactionStatusNotice}
      <TransactionStatusNotice {transactionStatusNotice} />
    {/each}
    {#each $toastsList as toast}
      <div class="flex justify-end">
        <AppToast {toast} />
      </div>
    {/each}
  </div>
</div>
