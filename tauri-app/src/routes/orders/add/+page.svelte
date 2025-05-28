<script lang="ts">
  import { PageHeader, CodeMirrorDotrain, ButtonLoading } from '@rainlanguage/ui-components';
  import FileTextarea from '$lib/components/FileTextarea.svelte';
  import { Label, Button, Spinner, Tabs, TabItem } from 'flowbite-svelte';
  import { makeChartData } from '$lib/services/chart';
  import type { ChartData } from '@rainlanguage/orderbook';
  import { settingsText, activeNetworkRef } from '$lib/stores/settings';
  import Charts from '$lib/components/Charts.svelte';
  import { globalDotrainFile } from '$lib/storesGeneric/textFileStore';
  import { isEmpty, isNil } from 'lodash';
  import type { Config } from '@rainlanguage/orderbook';
  import { DropdownRadio } from '@rainlanguage/ui-components';
  import { toasts } from '$lib/stores/toasts';
  import type { ConfigSource } from '@rainlanguage/orderbook';
  import ModalExecute from '$lib/components/ModalExecute.svelte';
  import { orderAdd, orderAddCalldata, validateRaindexVersion } from '$lib/services/order';
  import { ethersExecute } from '$lib/services/ethersTx';
  import { formatEthersTransactionError } from '$lib/utils/transaction';
  import { CodeMirrorRainlang } from '@rainlanguage/ui-components';
  import { SentrySeverityLevel, reportErrorToSentry } from '$lib/services/sentry';
  import { pickScenarios } from '$lib/services/pickConfig';
  import {
    convertConfigstringToConfig,
    mergeDotrainConfigWithSettings,
  } from '$lib/services/config';
  import ScenarioDebugTable from '$lib/components/ScenarioDebugTable.svelte';
  import { useDebouncedFn } from '$lib/utils/asyncDebounce';
  import Words from '$lib/components/Words.svelte';
  import { getAuthoringMetaV2ForScenarios } from '$lib/services/authoringMeta';
  import RaindexVersionValidator from '$lib/components/RaindexVersionValidator.svelte';
  import { page } from '$app/stores';
  import { codeMirrorTheme } from '$lib/stores/darkMode';
  import { generateRainlangStrings } from '$lib/services/generateRainlangStrings';
  import { getDeploymentsNetworks } from '$lib/utils/getDeploymentNetworks';
  import { createRainlangExtension } from '$lib/services/handleRainlangExtension';

  let isSubmitting = false;
  let isCharting = false;
  let chartData: ChartData;
  let deploymentRef: string | undefined = undefined;
  let scenarioRef: string | undefined = undefined;
  let mergedConfigSource: ConfigSource | undefined = undefined;
  let mergedConfig: Config | undefined = undefined;
  let openAddOrderModal = false;

  $: deployments = mergedConfig?.deployments;
  $: deployment = deploymentRef ? deployments?.[deploymentRef] : undefined;

  // Resetting the selected deployment to undefined if it is not in the current
  // strats deployment list anymore
  $: if (deploymentRef && deployments && !Object.keys(deployments).includes(deploymentRef)) {
    deploymentRef = undefined;
  }

  $: bindings = deployment ? deployment.scenario.bindings : {};
  $: if ($globalDotrainFile.text) updateMergedConfig();

  $: scenarios = pickScenarios(mergedConfig, $activeNetworkRef);

  let openTab: Record<string, boolean> = {};

  const {
    debouncedFn: debounceGetAuthoringMetas,
    result: authoringMetasResult,
    error: authoringMetasError,
  } = useDebouncedFn(getAuthoringMetaV2ForScenarios, 500);

  $: debounceGetAuthoringMetas($globalDotrainFile.text, [$settingsText]);

  const {
    debouncedFn: debouncedGenerateRainlangStrings,
    result: generatedRainlang,
    error,
  } = useDebouncedFn(generateRainlangStrings, 500);

  $: debouncedGenerateRainlangStrings(
    $globalDotrainFile.text,
    [$settingsText],
    mergedConfig?.scenarios,
  );

  $: rainlangExtension = createRainlangExtension(bindings, deployment?.scenario);

  $: {
    if (isNil(scenarioRef) && !isEmpty(scenarios)) {
      scenarioRef = Object.keys(scenarios)[0];
    }
  }

  async function updateMergedConfig() {
    try {
      mergedConfigSource = await mergeDotrainConfigWithSettings($globalDotrainFile.text);
      mergedConfig = await convertConfigstringToConfig(mergedConfigSource);
    } catch (e) {
      reportErrorToSentry(e, SentrySeverityLevel.Info);
    }
  }

  async function chart() {
    isCharting = true;
    try {
      chartData = await makeChartData($globalDotrainFile.text, $settingsText);
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
      if (isEmpty(deployment.order?.orderbook) || isEmpty(deployment.order.orderbook?.address))
        throw Error('No orderbook associated with scenario');

      await orderAdd($globalDotrainFile.text, deployment);
    } catch (e) {
      reportErrorToSentry(e);
    }
    isSubmitting = false;
  }

  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      if (!deployment) throw Error('Select a deployment to add order');
      if (isEmpty(deployment.order?.orderbook) || isEmpty(deployment.order.orderbook?.address))
        throw Error('No orderbook associated with scenario');

      const calldata = (await orderAddCalldata($globalDotrainFile.text, deployment)) as Uint8Array;
      const tx = await ethersExecute(calldata, deployment.order.orderbook.address);
      toasts.success('Transaction sent successfully!');
      await tx.wait(1);
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(formatEthersTransactionError(e));
    }
    isSubmitting = false;
  }

  const { debouncedFn: debounceValidateRaindexVersion, error: raindexVersionError } =
    useDebouncedFn(validateRaindexVersion, 500);

  $: debounceValidateRaindexVersion($globalDotrainFile.text, [$settingsText]);

  $: deploymentNetworks = getDeploymentsNetworks(deployments);
</script>

<PageHeader title="Add Order" pathname={$page.url.pathname} />

<FileTextarea textFile={globalDotrainFile}>
  <svelte:fragment slot="alert">
    <RaindexVersionValidator error={$raindexVersionError} />
  </svelte:fragment>

  <svelte:fragment slot="textarea">
    <CodeMirrorDotrain
      codeMirrorTheme={$codeMirrorTheme}
      rainlangText={$globalDotrainFile.text}
      disabled={isSubmitting}
      styles={{ '&': { minHeight: '400px' } }}
      {rainlangExtension}
      onTextChange={(text) => globalDotrainFile.set({ ...$globalDotrainFile, text })}
    />
  </svelte:fragment>

  <svelte:fragment slot="additionalFields">
    <div class="flex items-center justify-end gap-x-4">
      {#if isEmpty(deployments)}
        <span class="text-gray-500 dark:text-gray-400">No valid deployments found</span>
      {:else}
        <Label class="whitespace-nowrap">Select deployment</Label>
        <DropdownRadio options={deployments} bind:value={deploymentRef}>
          <svelte:fragment slot="content" let:selectedRef>
            <span>{!isNil(selectedRef) ? selectedRef : 'Select a deployment'}</span>
          </svelte:fragment>

          <svelte:fragment slot="option" let:ref>
            <div class="w-full overflow-hidden overflow-ellipsis">
              <div class="text-md break-word mb-2">{ref}</div>
            </div>
          </svelte:fragment>
        </DropdownRadio>
      {/if}
      <ButtonLoading
        class="min-w-fit"
        color="green"
        loading={isSubmitting}
        disabled={$globalDotrainFile.isEmpty || isNil(deploymentRef) || !!$raindexVersionError}
        on:click={() => (openAddOrderModal = true)}>Add Order</ButtonLoading
      >
    </div>
  </svelte:fragment>
</FileTextarea>

<Button disabled={isCharting} on:click={chart} size="sm" class="self-center"
  ><span class="mr-2">Generate charts</span>{#if isCharting}<Spinner size="5" />{/if}</Button
>

<Tabs
  style="underline"
  contentClass="mt-4"
  defaultClass="flex flex-wrap space-x-2 rtl:space-x-reverse mt-4"
>
  <TabItem open title="Rainlang">
    {#if $generatedRainlang && !$error}
      <Tabs
        style="underline"
        contentClass="mt-4"
        defaultClass="flex flex-wrap space-x-2 rtl:space-x-reverse mt-4"
      >
        {#each Array.from($generatedRainlang.entries()) as [scenario, rainlangText]}
          <TabItem bind:open={openTab[scenario.key]} title={scenario.key}>
            <CodeMirrorRainlang
              {rainlangText}
              codeMirrorDisabled={true}
              codeMirrorTheme={$codeMirrorTheme}
            />
          </TabItem>
        {/each}
      </Tabs>
    {/if}
  </TabItem>
  <TabItem title="Debug"><ScenarioDebugTable bind:networks={deploymentNetworks} /></TabItem>
  <TabItem title="Charts">
    <Charts {chartData} />
  </TabItem>
  <TabItem title="Words">
    <Words authoringMetas={$authoringMetasResult} error={$authoringMetasError} />
  </TabItem>
</Tabs>

<ModalExecute
  bind:open={openAddOrderModal}
  overrideNetwork={deployment?.order.network}
  title="Add Order"
  execButtonLabel="Add Order"
  {executeLedger}
  {executeWalletconnect}
  bind:isSubmitting
/>
