<script lang="ts">
  import PageHeader from '$lib/components/PageHeader.svelte';
  import CodeMirrorDotrain from '$lib/components/CodeMirrorDotrain.svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import FileTextarea from '$lib/components/FileTextarea.svelte';
  import { Helper, Label, Button, Spinner, Tabs, TabItem} from 'flowbite-svelte';
  import InputBlockNumber from '$lib/components/InputBlockNumber.svelte';
  import { forkBlockNumber } from '$lib/stores/forkBlockNumber';
  import { RawRainlangExtension, type Problem } from 'codemirror-rainlang';
  import { problemsCallback } from '$lib/services/langServices';
  import { makeChartData } from '$lib/services/chart';
  import type { ChartData } from '$lib/typeshare/config';
  import { settingsText, activeNetworkRef, orderbookAddress } from '$lib/stores/settings';
  import Charts from '$lib/components/Charts.svelte';
  import { textFileStore } from '$lib/storesGeneric/textFileStore';
  import { pickBy, isEmpty, isNil } from 'lodash';
  import { convertConfigstringToConfig, mergeDotrainConfigWithSettings, mergeDotrainConfigWithSettingsProblems } from '$lib/services/config';
  import type { Config } from '$lib/typeshare/config';
  import DropdownRadio from '$lib/components/DropdownRadio.svelte';
  import { toasts } from '$lib/stores/toasts';
  import type { ConfigSource } from '$lib/typeshare/config';
  import DropdownProperty from '$lib/components/DropdownProperty.svelte';
  import ModalExecute from '$lib/components/ModalExecute.svelte';
  import { orderAdd, orderAddCalldata, orderAddComposeRainlang } from '$lib/services/order';
  import { ethersExecute } from '$lib/services/ethersTx';
  import { formatEthersTransactionError } from '$lib/utils/transaction';
  import CodeMirrorRainlang from '$lib/components/CodeMirrorRainlang.svelte';
  import { promiseTimeout } from '$lib/utils/time';
  import { SentrySeverityLevel, reportErrorToSentry } from '$lib/services/sentry';

  let isSubmitting = false;
  let isCharting = false;
  let chartData: ChartData;
  let dotrainFile = textFileStore('Rain', ['rain']);
  let deploymentRef: string | undefined = undefined;
  let scenarioRef: string | undefined = undefined;
  let mergedConfigSource: ConfigSource | undefined = undefined;
  let mergedConfig: Config | undefined = undefined;
  let openAddOrderModal = false;
  let rainlangText = "";
  let resetRainlang = true;

  $: deployments = (!isNil(mergedConfigSource) && !isNil(mergedConfigSource?.deployments) && !isNil(mergedConfigSource?.orders)) ?
    pickBy(mergedConfigSource.deployments, (d) => mergedConfig?.scenarios?.[d.scenario]?.deployer?.network?.name === $activeNetworkRef) : {};
  $: deployment = (!isNil(deploymentRef) && !isNil(mergedConfig)) ? mergedConfig.deployments[deploymentRef] : undefined;
  $: bindings = deployment ? deployment.scenario.bindings : {};
  $: $dotrainFile.text, updateMergedConfig();

  $: scenarios = (!isNil(mergedConfigSource) && !isNil(mergedConfigSource?.scenarios)) ?
    pickBy(mergedConfigSource.scenarios, (d) => !d.deployer || mergedConfig?.deployers?.[d.deployer]?.network?.name === $activeNetworkRef) : {};
  $: scenario = (!isNil(scenarioRef) && !isNil(mergedConfig)) ? mergedConfig.scenarios[scenarioRef] : undefined;

  const rainlangExtension = new RawRainlangExtension({
    diagnostics: async (text) => {
      let configProblems = [];
      let problems = []
      try {
        // get problems with merging settings config with frontmatter
        configProblems = await mergeDotrainConfigWithSettingsProblems(text.text);
      } catch(e) {
        configProblems = [
          {
            msg: e as string,
            position: [0, 0],
            code: 9
          }
        ];
      }
      try {
        // get problems with dotrain
        problems = await promiseTimeout(
          problemsCallback(text, bindings, deployment?.scenario.deployer.address),
          5000,
          "failed to parse on native parser"
        );
      } catch(e) {
        problems = [{
          msg: e as string,
          position: [0, 0],
          code: 9
        }];
      }
      return [...configProblems, ...problems] as Problem[];
    },
  });

  $: {
    if(isNil(deploymentRef) && !isEmpty(deployments)) {
      deploymentRef = Object.keys(deployments)[0];
    }
  }
  $: {
    if(isNil(scenarioRef) && !isEmpty(scenarios)) {
      scenarioRef = Object.keys(scenarios)[0];
    }
  }

  $: $dotrainFile.text, scenario, resetRainlang = true;

  async function updateMergedConfig() {
    try {
      mergedConfigSource = await mergeDotrainConfigWithSettings($dotrainFile.text);
      mergedConfig = await convertConfigstringToConfig(mergedConfigSource);
    } catch (e) {
      reportErrorToSentry(e, SentrySeverityLevel.Info);
    }
  }

  async function chart() {
    isCharting = true;
    try {
      chartData = await makeChartData($dotrainFile.text, $settingsText);
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(e as string);
    }
    isCharting = false;
  }

  async function executeLedger() {
    isSubmitting = true;
    try {
      if (!deployment) throw Error('Select a deployment to add order');

      await orderAdd($dotrainFile.text, deployment);
    } catch (e) {
      reportErrorToSentry(e);
    }
    isSubmitting = false;
  }
  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      if (!deployment) throw Error('Select a deployment to add order');
      if (!$orderbookAddress) throw Error('Select an orderbook to add order');

      const calldata = (await orderAddCalldata($dotrainFile.text, deployment)) as Uint8Array;
      const tx = await ethersExecute(calldata, $orderbookAddress);
      toasts.success('Transaction sent successfully!');
      await tx.wait(1);
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(formatEthersTransactionError(e));
    }
    isSubmitting = false;
  }

  async function generateRainlangString() {
    try {
      if(!scenario) throw Error("Select a scenario to generate rainlang");
      rainlangText = await orderAddComposeRainlang($dotrainFile.text, scenario);
      resetRainlang = false;
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error("Please resolve issues first!")
      rainlangText = "";
      resetRainlang = true;
    }
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
    <div class="flex flex-col gap-y-2">
      <Label>Select Deployment</Label>
      {#if isEmpty(deployments)}
        <span class="text-gray-500 dark:text-gray-400">No deployments found for the selected network</span>
      {:else}
        <div class="flex justify-end gap-x-2">
          <div class="w-full">
            <DropdownRadio options={deployments} bind:value={deploymentRef}>
              <svelte:fragment slot="content"  let:selectedRef>
                <span>{!isNil(selectedRef) ? selectedRef : 'Select a deployment'}</span>
              </svelte:fragment>

              <svelte:fragment slot="option" let:ref let:option>
                <div class="w-full overflow-hidden overflow-ellipsis">
                  <div class="text-md mb-2 break-word">{ref}</div>
                  <DropdownProperty key="Scenario" value={option.scenario} />
                  <DropdownProperty key="Order" value={option.order} />
                </div>
              </svelte:fragment>
            </DropdownRadio>
          </div>
          <ButtonLoading
            class="min-w-fit"
            color="green"
            loading={isSubmitting}
            disabled={$dotrainFile.isEmpty}
            on:click={() => openAddOrderModal = true}>Add Order</ButtonLoading
          >
        </div>
      {/if}
    </div>
  </svelte:fragment>
</FileTextarea>

<div class="my-8">
  <Label class="mb-2">Parse at Block Number</Label>
  <InputBlockNumber
    bind:value={$forkBlockNumber.value}
    isFetching={$forkBlockNumber.isFetching}
    on:clickGetLatest={forkBlockNumber.fetch}
    required={false}
  />
  <Helper class="mt-2 text-sm">
    The block number at which to parse the rain while drafting. Resets to the latest block on app
    launch.
  </Helper>
</div>

<Tabs
  style="underline"
  contentClass="mt-4"
  defaultClass="flex flex-wrap space-x-2 rtl:space-x-reverse mt-4"
>
  <TabItem open title="Rainlang">
    <div class="flex flex-col gap-y-2">
      <Label>Select Scenario</Label>
      {#if isEmpty(scenarios)}
        <span class="text-gray-500 dark:text-gray-400">No scenarios found for the selected network</span>
      {:else}
        <div class="flex justify-end gap-x-2">
          <DropdownRadio options={scenarios} bind:value={scenarioRef}>
            <svelte:fragment slot="content"  let:selectedRef>
              <span>{!isNil(selectedRef) ? selectedRef : 'Select a scenario'}</span>
            </svelte:fragment>

            <svelte:fragment slot="option" let:ref let:option>
              <div class="w-full overflow-hidden overflow-ellipsis">
                <div class="text-md mb-2 break-word">{ref}</div>
                <DropdownProperty key="Scenario" value={option.deployer ?? ""} />
              </div>
            </svelte:fragment>
          </DropdownRadio>
          <Button on:click={generateRainlangString} class="min-w-fit">Generate Rainlang</Button>
        </div>
      {/if}
      {#if !resetRainlang}
        <CodeMirrorRainlang bind:value={rainlangText} disabled={true}/>
      {/if}
    </div>
  </TabItem>
  <TabItem title="Charts">
    {#if chartData}
      <Charts {chartData} />
    {/if}
    <Button disabled={isCharting} on:click={chart} class="w-full"><span class="mr-2">Make charts</span>{#if isCharting}<Spinner size="5" />{/if}</Button>
  </TabItem>
</Tabs>

<ModalExecute
  bind:open={openAddOrderModal}
  title="Add Order"
  execButtonLabel="Add Order"
  {executeLedger}
  {executeWalletconnect}
  bind:isSubmitting
/>
