<script lang="ts">
  import type { DeploymentDebugData } from '@rainlanguage/orderbook';
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
  import { Refresh } from '@rainlanguage/ui-components';
  import { EditableSpan } from '@rainlanguage/ui-components';
  import { makeDeploymentDebugData } from '$lib/services/chart';
  import { globalDotrainFile } from '$lib/storesGeneric/textFileStore';
  import { settingsText } from '$lib/stores/settings';
  import { createQuery } from '@tanstack/svelte-query';
  import { useDebouncedFn } from '$lib/utils/asyncDebounce';
  import { writable } from 'svelte/store';

  let enabled = false;
  let blockNumber: number | undefined;

  $: queryKey = writable([$globalDotrainFile.text, $settingsText]);

  const fetchData = async () => {
    const res = await makeDeploymentDebugData(
      $queryKey[0],
      $settingsText,
      enabled ? undefined : blockNumber,
    );
    blockNumber = parseInt(res.blockNumber) || 0;
    return res;
  };

  const fileUpdate = async (dotrain: string, settings: string): Promise<void> => {
    queryKey.set([dotrain, settings]);
  };
  const { debouncedFn: debounceFileUpdate } = useDebouncedFn(fileUpdate, 500);
  $: debounceFileUpdate($globalDotrainFile.text, $settingsText);

  $: scenarioDebugQuery = createQuery<DeploymentDebugData>({
    queryKey: $queryKey,
    queryFn: fetchData,
    refetchOnWindowFocus: false,
    enabled: $globalDotrainFile.text !== '' && $settingsText !== '',
  });

  const handleRefresh = () => {
    $scenarioDebugQuery.refetch();
  };

  const togglePlayback = () => {
    enabled = !enabled;
    if (enabled) {
      blockNumber = undefined;
    }
    handleRefresh();
  };
</script>

<div class="flex items-center justify-end">
  <div class="flex items-center gap-x-1">
    {#if $scenarioDebugQuery.isError}
      <div class="text-red-500">{$scenarioDebugQuery.error}</div>
    {/if}
    {#if $scenarioDebugQuery.data && isHex($scenarioDebugQuery.data.blockNumber)}
      <EditableSpan
        displayValue={blockNumber?.toString() ||
          hexToNumber($scenarioDebugQuery.data.blockNumber).toString()}
        on:focus={() => {
          enabled = false;
        }}
        on:blur={({ detail: { value } }) => {
          blockNumber = parseInt(value) || 0;
          handleRefresh();
        }}
      />
    {/if}
    <span></span>
    <Refresh
      data-testid="refreshButton"
      class="h-8 w-5 cursor-pointer text-gray-400 dark:text-gray-400"
      on:click={handleRefresh}
      spin={$scenarioDebugQuery.isFetching}
    />
    <button
      class="ml-2 h-8 w-3 cursor-pointer text-gray-400 dark:text-gray-400"
      on:click={togglePlayback}
    >
      {#if enabled}
        <PauseSolid />
      {:else}
        <PlaySolid />
      {/if}
    </button>
  </div>
</div>

{#if !$scenarioDebugQuery.error}
  {#each Object.entries($scenarioDebugQuery.data?.result ?? {}) as [deploymentName, results]}
    <h2 class="text-md my-4">Deployment: <strong>{deploymentName}</strong></h2>
    <Table divClass="rounded-lg overflow-hidden dark:border-none border overflow-x-scroll">
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
          <TableBodyRow>
            <TableBodyCell>{item.order}</TableBodyCell>
            <TableBodyCell>{item.scenario}</TableBodyCell>
            <TableBodyCell>{item.pair}</TableBodyCell>
            {#if item.result}
              {@const fuzzResult = item.result}
              {@const data = transformData(fuzzResult)[0]}
              {@const dataEntries = Object.entries(data)}
              {#if dataEntries.length < 2}
                <TableBodyCell colspan="2" class="text-red-500"
                  >Missing stack data for max output and ratio</TableBodyCell
                >
              {:else}
                {@const maxOutput = dataEntries[dataEntries.length - 2]}
                {@const ioRatio = dataEntries[dataEntries.length - 1]}
                <TableBodyCell>
                  {maxOutput[1][0]}
                </TableBodyCell>
                <TableBodyCell>
                  {ioRatio[1][0]}
                  <span class="text-gray-400">
                    ({BigInt(ioRatio[1][1]) === 0n
                      ? '0'
                      : formatUnits(10n ** 36n / BigInt(ioRatio[1][1]), 18)})
                  </span>
                </TableBodyCell>
              {/if}
              <TableBodyCell>
                <button on:click={() => handleScenarioDebugModal(item.pair, fuzzResult.data)}>
                  <BugOutline size="sm" color="grey" />
                </button>
              </TableBodyCell>
            {:else}
              <TableBodyCell colspan="5" class="text-red-500">{item.error}</TableBodyCell>
            {/if}
          </TableBodyRow>
        {/each}
      </TableBody>
    </Table>
  {/each}
{/if}
