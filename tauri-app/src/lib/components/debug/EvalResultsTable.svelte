<script lang="ts">
  import type { RainEvalResultsTable } from '$lib/typeshare/config';
  import {
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
  } from 'flowbite-svelte';
  import { formatEther, hexToBigInt, isHex } from 'viem';

  export let table: RainEvalResultsTable;
</script>

<Table divClass="cursor-pointer rounded-lg overflow-hidden dark:border-none border">
  <TableHead>
    <TableHeadCell>Stack item</TableHeadCell>
    <TableHeadCell>Value</TableHeadCell>
    <TableHeadCell>Hex</TableHeadCell>
  </TableHead>
  <TableBody>
    {#each table.rows[0] as value, i}
      <TableBodyRow>
        <TableBodyCell data-testid="modal-quote-debug-stack">{table.column_names[i]}</TableBodyCell>
        <TableBodyCell data-testid="modal-quote-debug-value"
          >{isHex(value) ? formatEther(hexToBigInt(value)) : ''}</TableBodyCell
        >
        <TableBodyCell data-testid="modal-quote-debug-value-hex">{value}</TableBodyCell>
      </TableBodyRow>
    {/each}
  </TableBody>
</Table>
