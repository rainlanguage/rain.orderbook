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
  let debugError: string | undefined;

  const fetchData = async (dotrain: string, settings: string, blockNumber?: number) => {
    if (dotrain === '' || settings === '') {
      return;
    }

    try {
      debugError = undefined;
      loading = true;

      const res = await makeDeploymentDebugData(
        dotrain,
        settings,
        enabled ? undefined : blockNumber,
      );

      blockNumber = parseInt(res.block_number);

      loading = false;
      return res;
    } catch (error) {
      debugError = error as string;
    }
  };
  const { debouncedFn: debounceMakeDeploymentDebugData, result: data } = useDebouncedFn(
    fetchData,
    500,
  );
  $: debounceMakeDeploymentDebugData($globalDotrainFile.text, $settingsText, blockNumber);

  const handleRefresh = () => {
    fetchData($globalDotrainFile.text, $settingsText, blockNumber);
  };
</script>

<div class="flex items-center justify-end">
  <div class="flex items-center gap-x-1">
    {#if debugError}
      <div class="text-red-500">{debugError}</div>
    {/if}
    {#if $data && isHex($data.block_number)}
      <EditableSpan
        displayValue={blockNumber?.toString() || hexToNumber($data.block_number).toString()}
        on:focus={() => {
          enabled = false;
        }}
        on:blur={({ detail: { value } }) => {
          blockNumber = parseInt(value);
          handleRefresh();
        }}
      />
    {/if}
    <span></span>
    {#if data}
      <Refresh
        data-testid="refreshButton"
        class="h-8 w-5 cursor-pointer text-gray-400 dark:text-gray-400"
        on:click={handleRefresh}
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
          handleRefresh();
        }}
        class={`ml-2 h-8 w-3 cursor-pointer text-gray-400 dark:text-gray-400 ${enabled ? 'hidden' : ''}`}
      />
    {/if}
  </div>
</div>

{#each Object.entries($data?.result ?? {}) as [deploymentName, results]}
  <h2 class="text-md my-4">Deployment: <strong>{deploymentName}</strong></h2>
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
        <TableBodyRow>
          <TableBodyCell>{item.order}</TableBodyCell>
          <TableBodyCell>{item.scenario}</TableBodyCell>
          <TableBodyCell>{item.pair}</TableBodyCell>
          {#if item.result}
            {@const fuzzResult = item.result}
            {@const data = transformData(fuzzResult)[0]}
            {@const dataEntries = Object.entries(data)}
            {#if dataEntries.length === 1}
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
