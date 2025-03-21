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
  import { QueryClientProvider } from '@tanstack/svelte-query';
  import { queryClient } from '$lib/queries/queryClient';
  import { WalletProvider } from '@rainlanguage/ui-components';
  import { derived } from 'svelte/store';
  import { walletconnectAccount } from '$lib/stores/walletconnect';
  import { ledgerWalletAddress } from '$lib/stores/wallets';

  const account = derived(
    [ledgerWalletAddress, walletconnectAccount],
    ([$ledgerWalletAddress, $walletconnectAccount]) => {
      return $ledgerWalletAddress || $walletconnectAccount || null;
    },
  );
</script>

<WindowDraggableArea />

<WalletProvider {account}>
  <QueryClientProvider client={queryClient}>
    <div
      class="mb-10 flex h-[calc(100vh-2.5rem)] w-full justify-start bg-white dark:bg-gray-900 dark:text-gray-400"
    >
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
      <div class="bg-primary-400 fixed bottom-0 left-64 right-0 h-10 p-2 text-center text-white">
        The Raindex app is still early alpha - have fun but use at your own risk!
      </div>
    </div>
  </QueryClientProvider>
</WalletProvider>
