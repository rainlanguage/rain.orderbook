<script lang="ts">
  import { queryClient } from '$lib/queries/queryClient';
  import { tradeDebug } from '$lib/queries/tradeDebug';
  import { createQuery } from '@tanstack/svelte-query';
  import {
    Alert,
    Modal,
    Spinner,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
  } from 'flowbite-svelte';
  import { formatEther, hexToBigInt } from 'viem';

  export let open: boolean;
  export let txHash: string;
  export let rpcUrl: string;

  $: debugQuery = createQuery(
    {
      queryKey: [txHash + rpcUrl],
      queryFn: () => {
        return tradeDebug(txHash, rpcUrl);
      },
      retry: 0,
    },
    queryClient,
  );
</script>

<Modal title="Debug trade" bind:open outsideclose size="lg">
  <div class="flex flex-col gap-y-2 text-sm">
    <span data-testid="modal-trade-debug-tx-hash">Trade transaction: {txHash}</span>
    <span data-testid="modal-trade-debug-rpc-url">RPC: {rpcUrl}</span>
  </div>
  {#if $debugQuery.isLoading}
    <div data-testid="modal-trade-debug-loading-message" class="flex items-center gap-x-2">
      <Spinner size="4" />
      <span>Replaying trade... this can take a while.</span>
    </div>
  {/if}
  {#if $debugQuery.isError}
    <Alert data-testid="modal-trade-debug-error" color="red">{$debugQuery.error}</Alert>
  {/if}
  {#if $debugQuery.data}
    <Table divClass="cursor-pointer rounded-lg overflow-hidden dark:border-none border">
      <TableHead>
        <TableHeadCell>Stack item</TableHeadCell>
        <TableHeadCell>Value</TableHeadCell>
        <TableHeadCell>Hex</TableHeadCell>
      </TableHead>
      <TableBody>
        {#each $debugQuery.data as value, i}
          <TableBodyRow>
            <TableBodyCell data-testid="modal-trade-debug-stack">{i}</TableBodyCell>
            <TableBodyCell data-testid="modal-trade-debug-value"
              >{formatEther(hexToBigInt(value))}</TableBodyCell
            >
            <TableBodyCell data-testid="modal-trade-debug-value-hex">{value}</TableBodyCell>
          </TableBodyRow>
        {/each}
      </TableBody>
    </Table>
  {/if}
</Modal>
