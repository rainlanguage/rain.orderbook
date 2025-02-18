<script lang="ts">
  import type { ChartData } from '@rainlanguage/orderbook/js_api';
  import { transformData } from '$lib/utils/chartData';
  import {
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
  } from 'flowbite-svelte';

  export let chartData: ChartData;
</script>

{#if chartData}
  <div class="flex w-full gap-x-4 overflow-x-scroll">
    {#each Object.values(chartData.scenariosData) as scenario}
      {@const data = transformData(scenario)}
      <div class="flex flex-col gap-y-2">
        <span>{scenario.scenario}</span>
        <Table divClass="cursor-pointer rounded-lg overflow-hidden dark:border-none border">
          <TableHead>
            <TableHeadCell>Stack item</TableHeadCell>
            <TableHeadCell>Value</TableHeadCell>
            <TableHeadCell>Hex</TableHeadCell>
          </TableHead>
          <TableBody>
            {#each Object.entries(data[0]) as [key, value]}
              <TableBodyRow>
                <TableBodyCell>{key}</TableBodyCell>
                <TableBodyCell>{value[0]}</TableBodyCell>
                <TableBodyCell>{value[1]}</TableBodyCell>
              </TableBodyRow>
            {/each}
          </TableBody>
        </Table>
      </div>
    {/each}
  </div>
{:else}
  No scenario data
{/if}
