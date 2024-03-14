/** eslint-disable no-console */
<script lang="ts">
  import PageHeader from '$lib/components/PageHeader.svelte';
  import CodeMirrorDotrain from '$lib/components/CodeMirrorDotrain.svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { orderAdd } from '$lib/services/order';
  import FileTextarea from '$lib/components/FileTextarea.svelte';
  import { Helper, Label, Button, Spinner } from 'flowbite-svelte';
  import InputBlockNumber from '$lib/components/InputBlockNumber.svelte';
  import { forkBlockNumber } from '$lib/stores/forkBlockNumber';
  import DropdownRadio from '$lib/components/DropdownRadio.svelte';
  import SkeletonRow from '$lib/components/SkeletonRow.svelte';
  import { deployments, activeDeploymentRef, dotrainFile } from '$lib/stores/settings';
  import { RawRainlangExtension, type RawLanguageServicesCallbacks } from 'codemirror-rainlang';
  import { completionCallback, hoverCallback, problemsCallback } from '$lib/services/langServices';
  import { makeChartData } from '$lib/services/chart';
  import { settingsText } from '$lib/stores/settings';
  import type { ChartData } from '$lib/typeshare/fuzz';
  import Charts from '$lib/components/Charts.svelte';

  let isSubmitting = false;
  let isCharting = false;
  let chartData: ChartData[];

  const callbacks: RawLanguageServicesCallbacks = {
		hover: hoverCallback,
		completion: completionCallback,
		diagnostics: problemsCallback
	}
  let ext = new RawRainlangExtension(callbacks);

  async function execute() {
    isSubmitting = true;
    try {
      await orderAdd($dotrainFile.text);
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }

  async function chart() {
    isCharting = true;
    chartData = await makeChartData($dotrainFile.text, $settingsText);
    isCharting = false;
  }

  $: console.log(chartData)
</script>

<PageHeader title="Add Order" />

<FileTextarea textFile={dotrainFile} title="New Order">
    <svelte:fragment slot="textarea">
      <CodeMirrorDotrain
          bind:value={$dotrainFile.text}
          disabled={isSubmitting}
          styles={{ '&': { minHeight: '400px' } }}
          rainlangExtension={ext}
        />
    </svelte:fragment>

    <svelte:fragment slot="submit">
      <ButtonLoading
        color="green"
        loading={isSubmitting}
        disabled={$dotrainFile.isEmpty}
        on:click={execute}>Add Order</ButtonLoading
      >
    </svelte:fragment>

    <svelte:fragment slot="deployment">
      <Label>Deployment</Label>
      {#if $deployments === undefined || Object.keys($deployments).length === 0}
        <SkeletonRow />
      {:else}
        <DropdownRadio options={$deployments} bind:value={$activeDeploymentRef}>
          <svelte:fragment slot="content" let:selected>
            {selected}
          </svelte:fragment>

          <svelte:fragment slot="option" let:option>
            <div class="w-full text-xs overflow-hidden overflow-ellipsis break-all">
              {option}
            </div>
          </svelte:fragment>
        </DropdownRadio>
      {/if}
    </svelte:fragment>
</FileTextarea>

<div class="my-8">
  <Label class="mb-2">Parse at Block Number</Label>
  <InputBlockNumber bind:value={$forkBlockNumber.value} isFetching={$forkBlockNumber.isFetching} on:clickGetLatest={forkBlockNumber.fetch} required={false} />
  <Helper class="mt-2 text-sm">
    The block number at which to parse the rain while drafting. Resets to
    the latest block on app launch.
  </Helper>
</div>

<Button disabled={isCharting} on:click={chart}><span class="mr-2">Make charts</span>{#if isCharting}<Spinner size="5" />{/if}</Button>
<Charts {chartData} />