<script lang="ts">
  import { createInfiniteQuery } from '@tanstack/svelte-query';
  import TanstackAppTable from './TanstackAppTable.svelte';
  import { QKEY_VAULTS_VOL_LIST } from '$lib/queries/keys';
  import { orderVaultsVolume } from '$lib/queries/orderTradesList';
  import { subgraphUrl } from '$lib/stores/settings';
  import { TableBodyCell, TableHeadCell } from 'flowbite-svelte';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/types/hash';
  import { formatUnits } from 'viem';
  import TableTimeFilters from '../charts/TableTimeFilters.svelte';
  import { bigintStringToHex } from '$lib/utils/hex';

  export let id: string;

  const now = Math.floor(new Date().getTime() / 1000);
  let startTimestamp: number | undefined = now - 60 * 60 * 24;
  let endTimestamp: number | undefined = now;

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
      {formatUnits(BigInt(item.totalIn), Number(item.token.decimals ?? 0))}
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2" data-testid="total-out">
      {formatUnits(BigInt(item.totalOut), Number(item.token.decimals ?? 0))}
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2" data-testid="total-vol">
      {formatUnits(BigInt(item.totalVol), Number(item.token.decimals ?? 0))}
    </TableBodyCell>
  </svelte:fragment>
</TanstackAppTable>
