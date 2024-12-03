<script lang="ts">
  import { createInfiniteQuery } from '@tanstack/svelte-query';
  import { TanstackAppTable } from '@rainlanguage/ui-components';
  import { QKEY_ORDER_TRADES_LIST } from '@rainlanguage/ui-components';
  import { getOrderTradesList, getOrderTradesCount } from '@rainlanguage/orderbook/js_api';
  import { rpcUrl, subgraphUrl } from '$lib/stores/settings';
  import { DEFAULT_PAGE_SIZE } from '@rainlanguage/ui-components';
  import { TableBodyCell, TableHeadCell } from 'flowbite-svelte';
  import { formatTimestampSecondsAsLocal } from '@rainlanguage/ui-components';
  import { Hash, HashType } from '@rainlanguage/ui-components';

  import { formatUnits } from 'viem';
  import { handleDebugTradeModal } from '$lib/services/modal';
  import { BugOutline } from 'flowbite-svelte-icons';
  import type { Trade } from '$lib/typeshare/subgraphTypes';
  import TableTimeFilters from '../charts/TableTimeFilters.svelte';

  export let id: string;

  let startTimestamp: number | undefined;
  let endTimestamp: number | undefined;
  let tradesCount: number | undefined;

  $: orderTradesQuery = createInfiniteQuery({
    queryKey: [id, QKEY_ORDER_TRADES_LIST + id],
    queryFn: async ({ pageParam }: { pageParam: number }) => {
      tradesCount = undefined;

      const [count, trades] = await Promise.all([
        getOrderTradesCount(
          $subgraphUrl || '',
          id,
          startTimestamp ? BigInt(startTimestamp) : undefined,
          endTimestamp ? BigInt(endTimestamp) : undefined,
        ),
        getOrderTradesList(
          $subgraphUrl || '',
          id,
          { page: pageParam + 1, pageSize: DEFAULT_PAGE_SIZE },
          startTimestamp ? BigInt(startTimestamp) : undefined,
          endTimestamp ? BigInt(endTimestamp) : undefined,
        ),
      ]);

      if (typeof count === 'number') {
        tradesCount = count;
      }

      return trades;
    },
    initialPageParam: 0,
    getNextPageParam: (lastPage: Trade[], _allPages: Trade[][], lastPageParam: number) => {
      return lastPage.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : undefined;
    },
    enabled: !!$subgraphUrl,
  });
</script>

<TanstackAppTable query={orderTradesQuery} emptyMessage="No trades found" rowHoverable={false}>
  <svelte:fragment slot="info">
    {#if tradesCount !== undefined}
      <div class="px-2">{tradesCount} Trades</div>
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
        BigInt(item.outputVaultBalanceChange.amount) * BigInt(-1),
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
