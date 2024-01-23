<script lang="ts">
  import { Spinner, Toast } from 'flowbite-svelte';
  import type { TransactionStatusNotice } from './typeshare/transactionStatus';
  import {
    CheckCircleSolid,
    CloseCircleSolid,
    ExclamationCircleSolid,
  } from 'flowbite-svelte-icons';

  export let transactionStatusNotice: TransactionStatusNotice;
</script>

<Toast class="mt-2 w-full !max-w-none">
  {#if transactionStatusNotice.series_position}
    <div class="text-lg font-bold">
      Transaction {transactionStatusNotice.series_position.position} of
      {transactionStatusNotice.series_position.total}
    </div>
  {/if}
  <div class="mb-4 text-lg font-bold text-white">{transactionStatusNotice.label}</div>
  <div class="flex w-full items-center justify-start space-x-4 px-4">
    {#if transactionStatusNotice.status.type === 'Initialized' || transactionStatusNotice.status.type === 'PendingPrepare'}
      <Spinner />
      <div class="mb-2 text-xl">Preparing Transaction</div>
    {:else if transactionStatusNotice.status.type === 'PendingSign'}
      <ExclamationCircleSolid class="h-10 w-10" color="yellow" />
      <div>
        <div class="mb-2 text-xl">Awaiting Signature</div>
        <div>Please review and sign the transaction on your Ledger device</div>
      </div>
    {:else if transactionStatusNotice.status.type === 'PendingSend'}
      <Spinner />
      <div>
        <div class="mb-2 text-xl">Submitting Transaction</div>
        <div>Sending and awaiting confirmations...</div>
      </div>
    {:else if transactionStatusNotice.status.type === 'Confirmed'}
      <CheckCircleSolid class="h-10 w-10" color="green" />
      <div>
        <div class="mb-2 text-xl">Transaction Confirmed</div>
        <div class="break-all">Hash: {transactionStatusNotice.status.payload}</div>
      </div>
    {:else if transactionStatusNotice.status.type === 'Failed'}
      <CloseCircleSolid class="h-10 w-10" color="red" />
      <div>
        <div class="mb-2 text-xl">Transaction Failed</div>
        <div>
          {transactionStatusNotice.status.payload}
        </div>
      </div>
    {/if}
  </div>
</Toast>
