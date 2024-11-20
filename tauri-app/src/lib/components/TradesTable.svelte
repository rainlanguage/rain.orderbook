<script lang="ts">
  import { TableBodyCell, TableHeadCell } from 'flowbite-svelte';
  import { formatTimestampSecondsAsLocal } from '@rainlanguage/ui-components';
  import { Hash, HashType } from '@rainlanguage/ui-components';

  import AppTable from '$lib/components/AppTable.svelte';
  import type { ListStore } from '$lib/storesGeneric/listStore';
  import type { Trade } from '$lib/typeshare/subgraphTypes';
  import { formatUnits } from 'viem';

  export let orderTradesList: ListStore<Trade>;
</script>

<AppTable listStore={orderTradesList} emptyMessage="No trades found" rowHoverable={false}>
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
      {item.inputVaultBalanceChange.vault.token.symbol}/{item.outputVaultBalanceChange.vault.token
        .symbol}
    </TableBodyCell>
  </svelte:fragment>
</AppTable>
