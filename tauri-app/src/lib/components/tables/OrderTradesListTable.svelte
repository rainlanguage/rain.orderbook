<script lang="ts">
  import { createInfiniteQuery } from '@tanstack/svelte-query';
  import TanstackAppTable from './TanstackAppTable.svelte';
  import { QKEY_ORDER_TRADES_LIST } from '$lib/queries/keys';
  import { orderTradesCount, orderTradesList } from '$lib/queries/orderTradesList';
  import { rpcUrl, subgraphUrl } from '$lib/stores/settings';
  import { DEFAULT_PAGE_SIZE } from '$lib/queries/constants';
  import { TableBodyCell, TableHeadCell } from 'flowbite-svelte';
  import { formatTimestampSecondsAsLocal } from '$lib/utils/time';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/types/hash';
  import { formatUnits } from 'viem';
  import { handleDebugTradeModal } from '$lib/services/modal';
  import { BugOutline } from 'flowbite-svelte-icons';
  import type { Trade } from '$lib/typeshare/subgraphTypes';
  import TableTimeFilters from '../charts/TableTimeFilters.svelte';

  export let id: string;

  const now = Math.floor(new Date().getTime() / 1000);
  let startTimestamp: number | undefined = now - 60 * 60 * 24;
  let endTimestamp: number | undefined = now;
  let tradesCount: number | undefined;

  $: orderTradesQuery = createInfiniteQuery({
    queryKey: [id, QKEY_ORDER_TRADES_LIST + id],
    queryFn: ({ pageParam }: { pageParam: number }) => {
      return orderTradesList(
        id,
        $subgraphUrl || '',
        pageParam,
        undefined,
        startTimestamp,
        endTimestamp,
      );
    },
    initialPageParam: 0,
    getNextPageParam: (lastPage: Trade[], _allPages: Trade[][], lastPageParam: number) => {
      return lastPage.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : undefined;
    },
    enabled: !!$subgraphUrl,
  });

  $: $orderTradesQuery.isFetching || $orderTradesQuery.isLoading,
    orderTradesCount(id, $subgraphUrl || '', startTimestamp, endTimestamp)
      .then((v) => (typeof v === 'number' ? (tradesCount = v) : (tradesCount = undefined)))
      .catch(() => (tradesCount = undefined));
</script>

<TanstackAppTable query={orderTradesQuery} emptyMessage="No trades found" rowHoverable={false}>
  <svelte:fragment slot="info">
    {#if tradesCount !== undefined}
      <div class="px-2">Total Count: {tradesCount}</div>
    {/if}
  </svelte:fragment>
  <svelte:fragment slot="timeFilter">
    <TableTimeFilters bind:startTimestamp bind:endTimestamp />
  </svelte:fragment>
  <svelte:fragment slot="head">
    <TableHeadCell padding="p-4">Date</TableHeadCell>
    <TableHeadCell padding="p-0">Sender</TableHeadCell>
    <TableHeadCell padding="p-0">Transaction Hash</TableHeadCell>
    <TableHeadCell padding="p-0">Input</TableHeadCell>
    <TableHeadCell padding="p-0">Output</TableHeadCell>
    <TableHeadCell padding="p-0">IO Ratio</TableHeadCell>
    <TableHeadCell padding="p-0"></TableHeadCell>
  </svelte:fragment>

  <svelte:fragment slot="bodyRow" let:item>
    <TableBodyCell tdClass="px-4 py-2">
      {formatTimestampSecondsAsLocal(BigInt(item.timestamp))}
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2 min-w-32">
      <Hash type={HashType.Wallet} value={item.tradeEvent.transaction.from} />
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2 min-w-32">
      <Hash type={HashType.Transaction} value={item.tradeEvent.transaction.id} />
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2">
      {formatUnits(
        BigInt(item.inputVaultBalanceChange.amount),
        Number(item.inputVaultBalanceChange.vault.token.decimals ?? 0),
      )}
      {item.inputVaultBalanceChange.vault.token.symbol}
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2">
      {formatUnits(
        BigInt(item.outputVaultBalanceChange.amount),
        Number(item.outputVaultBalanceChange.vault.token.decimals ?? 0),
      )}
      {item.outputVaultBalanceChange.vault.token.symbol}
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2" data-testid="io-ratio">
      {Math.abs(
        Number(
          formatUnits(
            BigInt(item.inputVaultBalanceChange.amount),
            Number(item.inputVaultBalanceChange.vault.token.decimals ?? 0),
          ),
        ) /
          Number(
            formatUnits(
              BigInt(item.outputVaultBalanceChange.amount),
              Number(item.outputVaultBalanceChange.vault.token.decimals ?? 0),
            ),
          ),
      )}
      <span class="text-gray-400">
        ({Math.abs(
          Number(
            formatUnits(
              BigInt(item.outputVaultBalanceChange.amount),
              Number(item.outputVaultBalanceChange.vault.token.decimals ?? 0),
            ),
          ) /
            Number(
              formatUnits(
                BigInt(item.inputVaultBalanceChange.amount),
                Number(item.inputVaultBalanceChange.vault.token.decimals ?? 0),
              ),
            ),
        )})
      </span>
    </TableBodyCell>
    <TableBodyCell tdClass="py-2">
      <button
        data-testid="debug-trade-button"
        class="text-gray-500 hover:text-gray-700"
        on:click={() => {
          if ($rpcUrl) handleDebugTradeModal(item.tradeEvent.transaction.id, $rpcUrl);
        }}
      >
        <BugOutline size="xs" />
      </button>
    </TableBodyCell>
  </svelte:fragment>
</TanstackAppTable>
