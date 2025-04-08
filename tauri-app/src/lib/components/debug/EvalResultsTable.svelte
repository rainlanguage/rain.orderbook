<script lang="ts">
  import type { RainEvalResultsTable } from '@rainlanguage/orderbook/js_api';
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

<Table
  divClass="cursor-pointer rounded-lg overflow-hidden dark:border-none border overflow-x-scroll"
>
  <TableHead>
    <TableHeadCell>Stack item</TableHeadCell>
    <TableHeadCell>Value</TableHeadCell>
    <TableHeadCell>Hex</TableHeadCell>
  </TableHead>
  <TableBody>
    {#each table.rows[0] as value, i}
      <TableBodyRow>
        <TableBodyCell data-testid="debug-stack">{table.columnNames[i]}</TableBodyCell>
        <TableBodyCell data-testid="debug-value"
          >{isHex(value) ? formatEther(hexToBigInt(value)) : ''}</TableBodyCell
        >
        <TableBodyCell data-testid="debug-value-hex">{value}</TableBodyCell>
      </TableBodyRow>
    {/each}
  </TableBody>
</Table>
