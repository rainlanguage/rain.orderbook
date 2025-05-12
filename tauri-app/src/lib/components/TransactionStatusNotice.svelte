<script lang="ts">
  import { Button, Spinner, Toast } from 'flowbite-svelte';
  import type { TransactionStatusNotice } from '$lib/types/tauriBindings';
  import {
    CheckCircleSolid,
    CloseCircleSolid,
    ExclamationCircleSolid,
  } from 'flowbite-svelte-icons';
  import { formatBlockExplorerTransactionUrl } from '$lib/utils/transaction';
  import { activeChainHasBlockExplorer } from '$lib/stores/settings';

  export let transactionStatusNotice: TransactionStatusNotice;
</script>

<Toast class="mt-2 w-full !max-w-none" dismissable={false}>
  <div data-testid="notice-label" class="mb-4 text-lg font-bold text-gray-900 dark:text-white">
    {transactionStatusNotice.label}
  </div>
  <div class="flex w-full items-center justify-start space-x-4 px-4">
    {#if transactionStatusNotice.status.type === 'Initialized' || transactionStatusNotice.status.type === 'PendingPrepare'}
      <Spinner data-testid="status-pending-prepare" />
      <div class="mb-2 text-xl">Preparing Transaction</div>
    {:else if transactionStatusNotice.status.type === 'PendingSign'}
      <ExclamationCircleSolid class="h-10 w-10" color="yellow" />
      <div data-testid="status-pending-sign">
        <div class="mb-2 text-xl">Awaiting Signature</div>
        <div>Please review and sign the transaction on your Ledger device</div>
      </div>
    {:else if transactionStatusNotice.status.type === 'PendingSend'}
      <Spinner data-testid="status-pending-send" />
      <div>
        <div class="mb-2 text-xl">Submitting Transaction</div>
        <div>Sending and awaiting confirmations...</div>
      </div>
    {:else if transactionStatusNotice.status.type === 'Confirmed'}
      <CheckCircleSolid data-testid="status-confirmed" class="h-10 w-10" color="green" />
      <div>
        <div class="mb-2 text-xl">Transaction Confirmed</div>
        <div data-testid="confirmed-payload" class="mb-4 break-all">
          Hash: {transactionStatusNotice.status.payload}
        </div>
        {#if $activeChainHasBlockExplorer}
          <Button
            data-testid="block-explorer-link"
            size="xs"
            color="light"
            href={formatBlockExplorerTransactionUrl(transactionStatusNotice.status.payload)}
            target="_blank"
          >
            View on Block Explorer
          </Button>
        {/if}
      </div>
    {:else if transactionStatusNotice.status.type === 'Failed'}
      <CloseCircleSolid data-testid="status-failed" class="h-10 w-10" color="red" />
      <div>
        <div class="mb-2 text-xl">Transaction Failed</div>
        <div data-testid="failed-payload">
          {transactionStatusNotice.status.payload}
        </div>
      </div>
    {/if}
  </div>
</Toast>
