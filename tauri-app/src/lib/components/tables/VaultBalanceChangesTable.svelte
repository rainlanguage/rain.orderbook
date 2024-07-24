<script lang="ts">
  import { Heading, TableHeadCell, TableBodyCell } from 'flowbite-svelte';
  import { formatTimestampSecondsAsLocal } from '$lib/utils/time';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/types/hash';
  import { formatUnits } from 'viem';
  import { createInfiniteQuery } from '@tanstack/svelte-query';
  import { vaultBalanceChangesList } from '$lib/queries/vaultBalanceChangesList';
  import { QKEY_VAULT_CHANGES } from '$lib/queries/keys';
  import { subgraphUrl } from '$lib/stores/settings';
  import { DEFAULT_PAGE_SIZE } from '$lib/queries/constants';
  import TanstackAppTable from '$lib/components/tables/TanstackAppTable.svelte';

  export let id: string;

  $: balanceChangesQuery = createInfiniteQuery({
    queryKey: [QKEY_VAULT_CHANGES + id],
    queryFn: ({ pageParam }) => {
      return vaultBalanceChangesList(id, $subgraphUrl || '', pageParam);
    },
    initialPageParam: 0,
    getNextPageParam(lastPage, _allPages, lastPageParam) {
      return lastPage.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : undefined;
    },
    refetchInterval: 10000,
    enabled: !!$subgraphUrl,
  });
</script>

<TanstackAppTable
  query={balanceChangesQuery}
  emptyMessage="No deposits or withdrawals found"
  rowHoverable={false}
>
  <svelte:fragment slot="title">
    <Heading tag="h5" class="mb-4 mt-6 font-normal">Vault Balance Changes</Heading>
  </svelte:fragment>
  <svelte:fragment slot="head">
    <TableHeadCell padding="p-4">Date</TableHeadCell>
    <TableHeadCell padding="p-0">Sender</TableHeadCell>
    <TableHeadCell padding="p-0">Transaction Hash</TableHeadCell>
    <TableHeadCell padding="p-0">Balance Change</TableHeadCell>
    <TableHeadCell padding="p-0">Balance</TableHeadCell>
    <TableHeadCell padding="p--">Type</TableHeadCell>
  </svelte:fragment>

  <svelte:fragment slot="bodyRow" let:item>
    <TableBodyCell tdClass="px-4 py-2">
      {formatTimestampSecondsAsLocal(BigInt(item.timestamp))}
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2 min-w-48">
      <Hash type={HashType.Wallet} value={item.transaction.from} />
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2 min-w-48">
      <Hash type={HashType.Transaction} value={item.transaction.id} />
    </TableBodyCell>
    <TableBodyCell tdClass="break-word p-0 text-left">
      {formatUnits(BigInt(item.amount), Number(item.vault.token.decimals ?? 0))}
      {item.vault.token.symbol}
    </TableBodyCell>
    <TableBodyCell tdClass="break-word p-0 text-left">
      {formatUnits(BigInt(item.new_vault_balance), Number(item.vault.token.decimals ?? 0))}
      {item.vault.token.symbol}
    </TableBodyCell>
    <TableBodyCell tdClass="break-word p-0 text-left">
      {item.__typename}
    </TableBodyCell>
  </svelte:fragment>
</TanstackAppTable>
