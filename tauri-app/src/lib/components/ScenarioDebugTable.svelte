<script lang="ts">
  import { transformData } from '$lib/utils/chartData';
  import { formatUnits, hexToNumber, isHex } from 'viem';
  import {
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
  } from 'flowbite-svelte';
  import { BugOutline, PauseSolid, PlaySolid } from 'flowbite-svelte-icons';
  import { handleScenarioDebugModal } from '$lib/services/modal';
  import Refresh from './icon/Refresh.svelte';
  import EditableSpan from './EditableSpan.svelte';
  import { useDebouncedFn } from '$lib/utils/asyncDebounce';
  import { makeDeploymentDebugData } from '$lib/services/chart';
  import { globalDotrainFile } from '$lib/storesGeneric/textFileStore';
  import { settingsText } from '$lib/stores/settings';

  let enabled = true;
  let loading = false;
  let blockNumber: number | undefined;

  const fetchData = async () => {
    loading = true;
    const res = await makeDeploymentDebugData(
      $globalDotrainFile.text,
      $settingsText,
      enabled ? undefined : blockNumber,
    );

    blockNumber = parseInt(res.block_number);

    loading = false;
    return res;
  };
  const { debouncedFn: debounceMakeDeploymentDebugData, result: data } = useDebouncedFn(
    fetchData,
    500,
  );
  $: debounceMakeDeploymentDebugData();
</script>

<div class="mt-4">
  <div class="flex items-center justify-end">
    <div class="flex items-center gap-x-1">
      {#if $data && isHex($data.block_number)}
        <EditableSpan
          displayValue={blockNumber?.toString() || hexToNumber($data.block_number).toString()}
          on:focus={() => {
            enabled = false;
          }}
          on:blur={({ detail: { value } }) => {
            blockNumber = parseInt(value);
            fetchData();
          }}
        />
      {/if}
      <span></span>
      {#if data}
        <Refresh
          data-testid="refreshButton"
          class="h-8 w-5 cursor-pointer text-gray-400 dark:text-gray-400"
          on:click={fetchData}
          spin={loading}
        />
        <PauseSolid
          class={`ml-2 h-8 w-3 cursor-pointer text-gray-400 dark:text-gray-400 ${!enabled ? 'hidden' : ''}`}
          on:click={() => {
            enabled = false;
          }}
        />
        <PlaySolid
          on:click={() => {
            enabled = true;
            blockNumber = undefined;
            fetchData();
          }}
          class={`ml-2 h-8 w-3 cursor-pointer text-gray-400 dark:text-gray-400 ${enabled ? 'hidden' : ''}`}
        />
      {/if}
    </div>
  </div>

  {#each Object.entries($data?.result ?? {}) as [deploymentName, results]}
    <h2 class="my-4 text-lg">Deployment: <strong>{deploymentName}</strong></h2>
    <Table divClass="rounded-lg overflow-hidden dark:border-none border">
      <TableHead>
        <TableHeadCell>Order</TableHeadCell>
        <TableHeadCell>Scenario</TableHeadCell>
        <TableHeadCell>Pair</TableHeadCell>
        <TableHeadCell>Maximum Output</TableHeadCell>
        <TableHeadCell>Ratio</TableHeadCell>
        <TableHeadCell class="w-[50px]" />
      </TableHead>

      <TableBody>
        {#each results as item}
          {@const data = transformData(item.fuzz_result)[0]}
          {@const ioRatio = Object.entries(data)[Object.entries(data).length - 1]}
          {@const maxOutput = Object.entries(data)[Object.entries(data).length - 2]}
          <TableBodyRow>
            <TableBodyCell>{item.order_name}</TableBodyCell>
            <TableBodyCell>{item.fuzz_result.scenario}</TableBodyCell>
            <TableBodyCell>{item.pair}</TableBodyCell>
            <TableBodyCell>
              {maxOutput[1][0]}
            </TableBodyCell>
            <TableBodyCell>
              {ioRatio[1][0]}
              <span class="text-gray-400">
                ({formatUnits(10n ** 36n / BigInt(ioRatio[1][1]), 18)})
              </span>
            </TableBodyCell>
            <TableBodyCell>
              <button on:click={() => handleScenarioDebugModal(item.pair, item.fuzz_result.data)}>
                <BugOutline size="sm" color="grey" />
              </button>
            </TableBodyCell>
          </TableBodyRow>
        {/each}
      </TableBody>
    </Table>
  {/each}
</div>
