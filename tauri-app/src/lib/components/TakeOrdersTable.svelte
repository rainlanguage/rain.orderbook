<script lang="ts">
  import { TableBodyCell, TableHeadCell } from 'flowbite-svelte';
  import { formatTimestampSecondsAsLocal } from '$lib/utils/time';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/types/hash';
  import AppTable from '$lib/components/AppTable.svelte';
  import type { ListStore } from '$lib/storesGeneric/listStore';
  import type { Trade } from '$lib/typeshare/orderTakesList';
  import { formatUnits } from 'viem';

  export let orderTakesList: ListStore<Trade>;
</script>

<AppTable listStore={orderTakesList} emptyMessage="No trades found" rowHoverable={false}>
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
</AppTable>
