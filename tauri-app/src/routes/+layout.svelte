<script>
  import '../app.pcss';
  import '@fontsource/dm-sans/400.css';
  import '@fontsource/dm-sans/600.css';
  import '@fontsource/dm-sans/800.css';
  import Sidebar from '$lib/Sidebar.svelte';
  import { toastsList } from '$lib/stores/toasts';
  import AppToast from '$lib/AppToast.svelte';
  import { transactionStatusNoticesList } from '$lib/stores/transactionStatusNotice';
  import TransactionStatusNotice from '$lib/TransactionStatusNotice.svelte';
</script>

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
      <AppToast {toast} />
    {/each}
  </div>
</div>
