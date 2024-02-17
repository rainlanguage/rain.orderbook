<script>
  import '../app.pcss';
  import '@fontsource/dm-sans/400.css';
  import '@fontsource/dm-sans/600.css';
  import '@fontsource/dm-sans/800.css';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import { toastsList } from '$lib/stores/toasts';
  import AppToast from '$lib/components/AppToast.svelte';
  import { transactionStatusNoticesList } from '$lib/stores/transactionStatusNotice';
  import TransactionStatusNotice from '$lib/components/TransactionStatusNotice.svelte';
</script>

<!-- This is where the window is draggable on mac OS without a title bar -->
<div class="absolute w-full h-24 top-0" data-tauri-drag-region></div>

<div class="flex min-h-screen w-full justify-start bg-white p-2 dark:bg-gray-600">
  <Sidebar />

  <main class="ml-52 h-full w-full grow overflow-x-auto p-8">
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
