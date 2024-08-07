<script lang="ts">
  import { createInfiniteQuery } from '@tanstack/svelte-query';
  import TanstackAppTable from './TanstackAppTable.svelte';
  import { QKEY_ORDER_TRADES_LIST } from '$lib/queries/keys';
  import { orderTradesList } from '$lib/queries/orderTradesList';
  import { subgraphUrl } from '$lib/stores/settings';
  import { DEFAULT_PAGE_SIZE } from '$lib/queries/constants';
  import { TableBodyCell, TableHeadCell } from 'flowbite-svelte';
  import { formatTimestampSecondsAsLocal } from '$lib/utils/time';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/types/hash';
  import { formatUnits } from 'viem';

  export let id: string;

  $: orderTradesQuery = createInfiniteQuery({
    queryKey: [QKEY_ORDER_TRADES_LIST + id],
    queryFn: ({ pageParam }) => {
      return orderTradesList(id, $subgraphUrl || '', pageParam);
    },
    initialPageParam: 0,
    getNextPageParam(lastPage, _allPages, lastPageParam) {
      return lastPage.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : undefined;
    },
    refetchInterval: 10000,
    enabled: !!$subgraphUrl,
  });
</script>

<TanstackAppTable query={orderTradesQuery} emptyMessage="No trades found" rowHoverable={false}>
  <svelte:fragment slot="head">
    <TableHeadCell padding="p-4">Date</TableHeadCell>
    <TableHeadCell padding="p-0">Sender</TableHeadCell>
    <TableHeadCell padding="p-0">Transaction Hash</TableHeadCell>
    <TableHeadCell padding="p-0">Input</TableHeadCell>
    <TableHeadCell padding="p-0">Output</TableHeadCell>
    <TableHeadCell padding="p-0">IO Ratio</TableHeadCell>
  </svelte:fragment>

  <svelte:fragment slot="bodyRow" let:item>
    <TableBodyCell tdClass="px-4 py-2">
      {formatTimestampSecondsAsLocal(BigInt(item.timestamp))}
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2 min-w-32">
      <Hash type={HashType.Wallet} value={item.trade_event.transaction.from} />
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2 min-w-32">
      <Hash type={HashType.Transaction} value={item.trade_event.transaction.id} />
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2">
      {formatUnits(
        BigInt(item.input_vault_balance_change.amount),
        Number(item.input_vault_balance_change.vault.token.decimals ?? 0),
      )}
      {item.input_vault_balance_change.vault.token.symbol}
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2">
      {formatUnits(
        BigInt(item.output_vault_balance_change.amount),
        Number(item.output_vault_balance_change.vault.token.decimals ?? 0),
      )}
      {item.output_vault_balance_change.vault.token.symbol}
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2" data-testid="io-ratio">
      {Math.abs(
        Number(
          formatUnits(
            BigInt(item.input_vault_balance_change.amount),
            Number(item.input_vault_balance_change.vault.token.decimals ?? 0),
          ),
        ) /
          Number(
            formatUnits(
              BigInt(item.output_vault_balance_change.amount),
              Number(item.output_vault_balance_change.vault.token.decimals ?? 0),
            ),
          ),
      )}
      {item.input_vault_balance_change.vault.token.symbol}/{item.output_vault_balance_change.vault
        .token.symbol}
    </TableBodyCell>
  </svelte:fragment>
</TanstackAppTable>
