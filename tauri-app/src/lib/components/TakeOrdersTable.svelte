<script lang="ts">
  import { TableBodyCell, TableHeadCell } from 'flowbite-svelte';
  import { formatTimestampSecondsAsLocal } from '$lib/utils/time';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/types/hash';
  import AppTable from '$lib/components/AppTable.svelte';
  import type { ListStore } from '$lib/storesGeneric/listStore';
  import type { TakeOrderEntity } from '$lib/typeshare/orderTakesList';

  export let orderTakesList: ListStore<TakeOrderEntity>;
</script>

<AppTable listStore={orderTakesList} emptyMessage="No trades found" rowHoverable={false}>
  <svelte:fragment slot="head">
    <TableHeadCell padding="p-4">Date</TableHeadCell>
    <TableHeadCell padding="p-0">Sender</TableHeadCell>
    <TableHeadCell padding="p-0">Transaction Hash</TableHeadCell>
    <TableHeadCell padding="p-0">Output</TableHeadCell>
    <TableHeadCell padding="p-0">Input</TableHeadCell>
    <TableHeadCell padding="p-0">IO Ratio</TableHeadCell>
  </svelte:fragment>

  <svelte:fragment slot="bodyRow" let:item>
    <TableBodyCell tdClass="px-4 py-2">
      {formatTimestampSecondsAsLocal(BigInt(item.timestamp))}
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2 min-w-32">
      <Hash type={HashType.Wallet} value={item.sender.id} />
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2 min-w-32">
      <Hash type={HashType.Transaction} value={item.transaction.id} />
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2">
      {item.input_display}
      {item.input_token.symbol}
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2">
      {item.output_display}
      {item.output_token.symbol}
    </TableBodyCell>
    <TableBodyCell tdClass="break-all py-2" data-testid="io-ratio">
      {Number(item.output_display) / Number(item.input_display)}
      {item.output_token.symbol}/{item.input_token.symbol}
    </TableBodyCell>
  </svelte:fragment>
</AppTable>
