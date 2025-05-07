<script lang="ts">
  import { formatUnits } from 'viem';
  import type { DeploymentsDebugDataMap } from '@rainlanguage/orderbook';
  import { transformData } from '$lib/utils/chartData';
  import { BugOutline, EditOutline, PauseSolid, PlaySolid } from 'flowbite-svelte-icons';
  import { handleScenarioDebugModal } from '$lib/services/modal';
  import { DEFAULT_REFRESH_INTERVAL, Refresh } from '@rainlanguage/ui-components';
  import { makeDeploymentsDebugDataMap } from '$lib/services/chart';
  import { globalDotrainFile } from '$lib/storesGeneric/textFileStore';
  import { settingsText } from '$lib/stores/settings';
  import { createQuery } from '@tanstack/svelte-query';
  import { useDebouncedFn } from '$lib/utils/asyncDebounce';
  import { writable } from 'svelte/store';
  import ModalDebugContext from './modal/ModalDebugContext.svelte';
  import {
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
  } from 'flowbite-svelte';

  let enabled = false;
  let openDebugBlockNumberModal = false;
  let blockNumbers: Record<number, number> = {};
  export let networks: Record<number, string> | undefined;

  $: queryKey = writable([$globalDotrainFile.text, $settingsText]);
  let displayData: DeploymentsDebugDataMap['dataMap'] | undefined = undefined;

  const fetchData = async () => {
    const res = await makeDeploymentsDebugDataMap(
      $queryKey[0],
      $settingsText,
      enabled ? undefined : blockNumbers,
    );
    // build a map of chain ids against block numbers for unified debug on same
    // block number per chain id, this is because we dont want to run the debug
    // on different blocks for different deployments if those deployments happen
    // to be on the same network but for example with different rpc, so we keep
    // a map of chain ids against block number rather than map of deployment keys
    // against block numbers
    for (const deploymentKey in res.dataMap) {
      blockNumbers[res.dataMap[deploymentKey].chainId] =
        res.dataMap[deploymentKey].blockNumber || 0;
    }
    return res;
  };

  const fileUpdate = async (dotrain: string, settings: string): Promise<void> => {
    queryKey.set([dotrain, settings]);
  };
  const { debouncedFn: debounceFileUpdate } = useDebouncedFn(fileUpdate, 500);
  $: debounceFileUpdate($globalDotrainFile.text, $settingsText);

  $: scenarioDebugQuery = createQuery<DeploymentsDebugDataMap>({
    queryKey: $queryKey,
    queryFn: fetchData,
    refetchOnWindowFocus: false,
    enabled: $globalDotrainFile.text !== '' && $settingsText !== '',
    refetchInterval: enabled ? DEFAULT_REFRESH_INTERVAL : false,
  });

  $: {
    if (!$scenarioDebugQuery.isError && $scenarioDebugQuery.data) {
      displayData = $scenarioDebugQuery.data.dataMap;
    } else if ($globalDotrainFile.text === '' || $settingsText === '') {
      displayData = undefined;
    }
  }

  const handleRefresh = () => {
    $scenarioDebugQuery.refetch();
  };

  const togglePlayback = () => {
    enabled = !enabled;
    if (enabled) {
      blockNumbers = {};
      handleRefresh();
    }
  };
</script>

<div class="flex items-center justify-end">
  <div class="flex items-center gap-x-1">
    {#if $scenarioDebugQuery.isError}
      <div class="text-red-500">{$scenarioDebugQuery.error}</div>
    {/if}
    <span></span>
    <button
      type="button"
      class="mr-2 flex items-center text-sm text-gray-500 hover:text-gray-700 focus:outline-none"
      on:click={() => (openDebugBlockNumberModal = true)}
    >
      <span class="mr-1">Change block heights</span>
      <EditOutline />
    </button>
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

{#if !$scenarioDebugQuery.error && displayData}
  {#each Object.entries(displayData).sort( (a, b) => (a[1].chainId > b[1].chainId ? 1 : -1), ) as [deploymentName, results]}
    <h2 class="text-md my-4">Deployment: <strong>{deploymentName}</strong></h2>
    <Table divClass="rounded-lg overflow-hidden dark:border-none border overflow-x-scroll">
      <TableHead>
        <TableHeadCell>Order</TableHeadCell>
        <TableHeadCell>Scenario</TableHeadCell>
        <TableHeadCell>Pair</TableHeadCell>
        <TableHeadCell>Maximum Output</TableHeadCell>
        <TableHeadCell>Ratio</TableHeadCell>
        <TableHeadCell>Block Height</TableHeadCell>
        <TableHeadCell class="w-[50px]" />
      </TableHead>

      <TableBody>
        {#each results.pairsData as item}
          <TableBodyRow>
            <TableBodyCell>{item.order}</TableBodyCell>
            <TableBodyCell>{item.scenario}</TableBodyCell>
            <TableBodyCell>{item.pair}</TableBodyCell>
            {#if item.result}
              {@const fuzzResult = item.result}
              {@const data = transformData(fuzzResult)[0]}
              {@const dataEntries = Object.entries(data)}
              {@const keyRegex = /^\d+\.\d+$/}
              {@const mainEntries = dataEntries.filter(([key]) => keyRegex.test(key))}

              {#if mainEntries.length < 2}
                <TableBodyCell colspan="2" class="text-red-500"
                  >Missing stack data for max output and ratio</TableBodyCell
                >
              {:else}
                {@const maxOutput = mainEntries[mainEntries.length - 2]}
                {@const ioRatio = mainEntries[mainEntries.length - 1]}
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
              <TableBodyCell>{results.blockNumber}</TableBodyCell>
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

<ModalDebugContext bind:open={openDebugBlockNumberModal} bind:blockNumbers bind:networks />
