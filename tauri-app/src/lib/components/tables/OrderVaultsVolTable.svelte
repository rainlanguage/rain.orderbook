<script lang="ts">
  import { createInfiniteQuery } from '@tanstack/svelte-query';
  import { TanstackAppTable } from '@rainlanguage/ui-components';
  import { QKEY_VAULTS_VOL_LIST } from '@rainlanguage/ui-components';
  import { orderVaultsVolume } from '$lib/queries/orderTradesList';
  import { subgraphUrl } from '$lib/stores/settings';
  import { TableBodyCell, TableHeadCell } from 'flowbite-svelte';
  import { Hash, HashType } from '@rainlanguage/ui-components';

  import { formatUnits } from 'viem';
  import { TableTimeFilters } from '@rainlanguage/ui-components';
  import { bigintStringToHex } from '@rainlanguage/ui-components';

  export let id: string;

  let startTimestamp: number | undefined;
  let endTimestamp: number | undefined;

  $: vaultsVol = createInfiniteQuery({
    queryKey: [id, QKEY_VAULTS_VOL_LIST + id],
    queryFn: () => orderVaultsVolume(id, $subgraphUrl || '', startTimestamp, endTimestamp),
    initialPageParam: 0,
    getNextPageParam: () => undefined,
    enabled: !!$subgraphUrl,
  });
</script>

<TanstackAppTable query={vaultsVol} emptyMessage="No trades found" rowHoverable={false}>
  <svelte:fragment slot="timeFilter">
    <TableTimeFilters bind:startTimestamp bind:endTimestamp />
  </svelte:fragment>
  <svelte:fragment slot="head">
    <TableHeadCell padding="p-4">Vault</TableHeadCell>
    <TableHeadCell padding="p-0">Token</TableHeadCell>
    <TableHeadCell padding="p-0">In Volume</TableHeadCell>
    <TableHeadCell padding="p-0">Out Volume</TableHeadCell>
    <TableHeadCell padding="p-0">Net Volume</TableHeadCell>
    <TableHeadCell padding="p-0">Total Volume</TableHeadCell>
  </svelte:fragment>

  <svelte:fragment slot="bodyRow" let:item>
    <TableBodyCell tdClass="px-4 py-2">
      <Hash type={HashType.Identifier} shorten value={bigintStringToHex(item.id)} />
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2 min-w-32">
      <div class="flex gap-x-3">
        <Hash type={HashType.Address} shorten value={item.token.address} />
        {item.token.symbol}
      </div>
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2 min-w-32" data-testid="total-in">
      {formatUnits(BigInt(item.volDetails.totalIn), Number(item.token.decimals ?? 0))}
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2" data-testid="total-out">
      {formatUnits(BigInt(item.volDetails.totalOut), Number(item.token.decimals ?? 0))}
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2" data-testid="net-vol">
      {(BigInt(item.volDetails.totalIn) >= BigInt(item.volDetails.totalOut) ? '' : '-') +
        formatUnits(BigInt(item.volDetails.netVol), Number(item.token.decimals ?? 0))}
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2" data-testid="total-vol">
      {formatUnits(BigInt(item.volDetails.totalVol), Number(item.token.decimals ?? 0))}
    </TableBodyCell>
  </svelte:fragment>
</TanstackAppTable>
