<script lang="ts">
  import PageHeader from '$lib/components/PageHeader.svelte';
  import CodeMirrorDotrain from '$lib/components/CodeMirrorDotrain.svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import FileTextarea from '$lib/components/FileTextarea.svelte';
  import { Helper, Label, Button, Spinner} from 'flowbite-svelte';
  import InputBlockNumber from '$lib/components/InputBlockNumber.svelte';
  import { forkBlockNumber } from '$lib/stores/forkBlockNumber';
  import { RawRainlangExtension, type Problem } from 'codemirror-rainlang';
  import { completionCallback, hoverCallback, problemsCallback } from '$lib/services/langServices';
  import { makeChartData } from '$lib/services/chart';
  import { settingsText, activeNetworkRef, orderbookAddress } from '$lib/stores/settings';
  import type { ChartData } from '$lib/typeshare/fuzz';
  import Charts from '$lib/components/Charts.svelte';
  import { textFileStore } from '$lib/storesGeneric/textFileStore';
  import { pickBy } from 'lodash';
  import { convertConfigstringToConfig, mergeDotrainConfigWithSettings, mergeDotrainConfigWithSettingsProblems } from '$lib/services/config';
  import type { Config } from '$lib/typeshare/config';
  import DropdownRadio from '$lib/components/DropdownRadio.svelte';
  import { toasts } from '$lib/stores/toasts';
  import type { ConfigSource } from '$lib/typeshare/configString';
  import DropdownProperty from '$lib/components/DropdownProperty.svelte';
  import ModalExecute from '$lib/components/ModalExecute.svelte';
  import { orderAdd, orderAddCalldata } from '$lib/services/order';
  import { ethersExecute } from '$lib/services/ethersTx';
  import { formatEthersTransactionError } from '$lib/utils/transaction';

  let isSubmitting = false;
  let isCharting = false;
  let chartData: ChartData[];
  let dotrainFile = textFileStore('Rain', ['rain']);
  let deploymentRef: string | undefined = undefined;
  let mergedConfigSource: ConfigSource | undefined = undefined;
  let mergedConfig: Config | undefined = undefined;
  let openAddOrderModal = false;

  $: deployments = (mergedConfigSource !== undefined && mergedConfigSource?.deployments !== undefined && mergedConfigSource?.orders !== undefined) ?
    pickBy(mergedConfigSource.deployments, (d) => mergedConfig?.scenarios?.[d.scenario]?.deployer?.network?.name === $activeNetworkRef) : {};
  $: deployment = (deploymentRef !== undefined && mergedConfig !== undefined) ? mergedConfig.deployments[deploymentRef] : undefined;
  $: bindings = deployment ? deployment.scenario.bindings : {};
  $: $dotrainFile.text, updateMergedConfig();

  $: rainlangExtension = new RawRainlangExtension({
    hover: (text, position) => hoverCallback.apply(null, [text, position, bindings]),
    diagnostics: async (text) => {
      // get problems with merging settings config with frontmatter
      const configProblems = await mergeDotrainConfigWithSettingsProblems(text.text);

      // get problems with dotrain
      const problems = await problemsCallback.apply(null, [text, bindings, deployment?.scenario.deployer.address]);

      return [...configProblems, ...problems] as Problem[];
    },
    completion: (text, position) => completionCallback.apply(null, [text, position, bindings]),
  });

  $: {
    if(deploymentRef === undefined && deployments !== undefined && Object.keys(deployments).length > 0) {
      deploymentRef = Object.keys(deployments)[0];
    }
  }

  async function updateMergedConfig() {
    try {
      mergedConfigSource = await mergeDotrainConfigWithSettings($dotrainFile.text);
      mergedConfig = await convertConfigstringToConfig(mergedConfigSource);
      // eslint-disable-next-line no-empty
    } catch(e) {}
  }

  async function chart() {
    isCharting = true;
    try {
      chartData = await makeChartData($dotrainFile.text, $settingsText);
    } catch(e) {
      toasts.error(e as string);
    }
    isCharting = false;
  }

  async function executeLedger() {
    isSubmitting = true;
    try {
      if(!deployment) throw Error("Select a deployment to add order");

      await orderAdd($dotrainFile.text, deployment);
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }
  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      if(!deployment) throw Error("Select a deployment to add order");
      if (!$orderbookAddress) throw Error("Select an orderbook to add order");

      const calldata = await orderAddCalldata($dotrainFile.text, deployment) as Uint8Array;
      const tx = await ethersExecute(calldata, $orderbookAddress);
      toasts.success("Transaction sent successfully!");
      await tx.wait(1);
    } catch (e) {
      toasts.error(formatEthersTransactionError(e));
    }
    isSubmitting = false;
  }
</script>

<PageHeader title="Add Order" />

<FileTextarea textFile={dotrainFile} title="New Order">
    <svelte:fragment slot="textarea">
      <CodeMirrorDotrain
          bind:value={$dotrainFile.text}
          disabled={isSubmitting}
          styles={{ '&': { minHeight: '400px' } }}
          {rainlangExtension}
        />
    </svelte:fragment>

    <svelte:fragment slot="additionalFields">
      <div class="flex justify-end w-full">
        <div class="w-72">
          <Label>Deployment</Label>
          {#if deployments === undefined || Object.keys(deployments).length === 0}
            <span class="text-gray-500 dark:text-gray-400">No deployments found for the selected network</span>
          {:else}
            <DropdownRadio options={deployments} bind:value={deploymentRef}>
              <svelte:fragment slot="content"  let:selectedRef>
                <span>{selectedRef !== undefined ? selectedRef : 'Select a deployment'}</span>
              </svelte:fragment>

              <svelte:fragment slot="option" let:ref let:option>
                <div class="w-full overflow-hidden overflow-ellipsis">
                  <div class="text-md mb-2 break-word">{ref}</div>
                  <DropdownProperty key="Scenario" value={option.scenario} />
                  <DropdownProperty key="Order" value={option.order} />
                </div>
              </svelte:fragment>
            </DropdownRadio>
            {/if}
          </div>
        </div>
    </svelte:fragment>

    <svelte:fragment slot="submit">
      <ButtonLoading
        color="green"
        loading={isSubmitting}
        disabled={$dotrainFile.isEmpty}
        on:click={() => openAddOrderModal = true}>Add Order</ButtonLoading
      >
      <Button disabled={isCharting} on:click={chart}><span class="mr-2">View Generated Rainlang</span>{#if isCharting}<Spinner size="5" />{/if}</Button>
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
<ModalExecute
  bind:open={openAddOrderModal}
  title="Add Order"
  execButtonLabel="Add Order"
  {executeLedger}
  {executeWalletconnect}
  bind:isSubmitting={isSubmitting}
/>