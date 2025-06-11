<script lang="ts">
  import { PageHeader, CodeMirrorDotrain, ButtonLoading } from '@rainlanguage/ui-components';
  import FileTextarea from '$lib/components/FileTextarea.svelte';
  import { Label, Button, Spinner, Tabs, TabItem } from 'flowbite-svelte';
  import { RawRainlangExtension, type Problem } from 'codemirror-rainlang';
  import { problemsCallback } from '$lib/services/langServices';
  import { makeChartData } from '$lib/services/chart';
  import type { ChartData } from '@rainlanguage/orderbook';
  import { settingsText, activeNetworkRef } from '$lib/stores/settings';
  import Charts from '$lib/components/Charts.svelte';
  import { globalDotrainFile } from '$lib/storesGeneric/textFileStore';
  import { isEmpty, isNil } from 'lodash';
  import type { Config } from '@rainlanguage/orderbook';
  import { DropdownRadio } from '@rainlanguage/ui-components';
  import { toasts } from '$lib/stores/toasts';
  import ModalExecute from '$lib/components/ModalExecute.svelte';
  import { orderAdd, orderAddCalldata, validateSpecVersion } from '$lib/services/order';
  import { ethersExecute } from '$lib/services/ethersTx';
  import { formatEthersTransactionError } from '$lib/utils/transaction';
  import { promiseTimeout, CodeMirrorRainlang } from '@rainlanguage/ui-components';
  import { SentrySeverityLevel, reportErrorToSentry } from '$lib/services/sentry';
  import { pickScenarios } from '$lib/services/pickConfig';
  import { parseDotrainAndSettingsProblems } from '$lib/services/configCodemirrorProblems';
  import ScenarioDebugTable from '$lib/components/ScenarioDebugTable.svelte';
  import { useDebouncedFn } from '$lib/utils/asyncDebounce';
  import Words from '$lib/components/Words.svelte';
  import { getAuthoringMetaV2ForScenarios } from '$lib/services/authoringMeta';
  import SpecVersionValidator from '$lib/components/SpecVersionValidator.svelte';
  import { page } from '$app/stores';
  import { codeMirrorTheme } from '$lib/stores/darkMode';
  import { executeWalletConnectOrder } from '$lib/services/executeWalletConnectOrder';
  import { executeLedgerOrder } from '$lib/services/executeLedgerOrder';
  import { generateRainlangStrings } from '$lib/services/generateRainlangStrings';
  import { getDeploymentsNetworks } from '$lib/utils/getDeploymentNetworks';
  import { parseYaml } from '$lib/services/config';

  let isSubmitting = false;
  let isCharting = false;
  let chartData: ChartData;
  let deploymentRef: string | undefined = undefined;
  let scenarioRef: string | undefined = undefined;
  let mergedConfig: Config | undefined = undefined;
  let openAddOrderModal = false;

  $: deployments = mergedConfig?.dotrainOrder.deployments;
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
    mergedConfig?.dotrainOrder.scenarios,
  );

  $: rainlangExtension = new RawRainlangExtension({
    diagnostics: async (text) => {
      let configProblems = [];
      let problems = [];
      try {
        // get problems with merging settings config with frontmatter
        configProblems = await parseDotrainAndSettingsProblems(text.text, $settingsText);
      } catch (e) {
        configProblems = [
          {
            msg: e as string,
            position: [0, 0],
            code: 9,
          },
        ];
      }
      try {
        // get problems with dotrain
        problems = await promiseTimeout(
          problemsCallback(text, bindings, deployment?.scenario.deployer.address),
          5000,
          'failed to parse on native parser',
        );
      } catch (e) {
        problems = [
          {
            msg: e as string,
            position: [0, 0],
            code: 9,
          },
        ];
      }
      return [...configProblems, ...problems] as Problem[];
    },
  });

  $: {
    if (isNil(scenarioRef) && !isEmpty(scenarios)) {
      scenarioRef = Object.keys(scenarios)[0];
    }
  }

  async function updateMergedConfig() {
    try {
      mergedConfig = await parseYaml([$globalDotrainFile.text]);
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

  async function handleExecuteLedger() {
    isSubmitting = true;
    try {
      if (!deployment) throw Error('Select a deployment to add order');
      await executeLedgerOrder($globalDotrainFile.text, deployment, orderAdd, reportErrorToSentry);
    } catch (e: unknown) {
      toasts.error((e as Error).message || 'Ledger execution failed');
    }
    isSubmitting = false;
  }

  async function handleExecuteWalletConnect() {
    isSubmitting = true;
    try {
      if (!deployment) throw Error('Select a deployment to add order');
      await executeWalletConnectOrder($globalDotrainFile.text, deployment, {
        orderAddCalldataFn: async (dotrain, deploy) =>
          (await orderAddCalldata(dotrain, deploy)) as Uint8Array,
        ethersExecuteFn: ethersExecute,
        reportErrorToSentryFn: reportErrorToSentry,
        formatEthersTransactionErrorFn: formatEthersTransactionError,
        successToastFn: toasts.success,
        errorToastFn: toasts.error,
      });
    } catch {
      // error already reported by service or toast shown
    }
    isSubmitting = false;
  }

  const { debouncedFn: debounceValidateSpecVersion, error: specVersionError } = useDebouncedFn(
    validateSpecVersion,
    500,
  );

  $: debounceValidateSpecVersion($globalDotrainFile.text, [$settingsText]);

  $: deploymentNetworks = getDeploymentsNetworks(deployments);
</script>

<PageHeader title="Add Order" pathname={$page.url.pathname} />

<FileTextarea textFile={globalDotrainFile}>
  <svelte:fragment slot="alert">
    <SpecVersionValidator error={$specVersionError} />
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
        disabled={$globalDotrainFile.isEmpty || isNil(deploymentRef) || !!$specVersionError}
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
  executeWalletconnect={handleExecuteWalletConnect}
  executeLedger={handleExecuteLedger}
  bind:isSubmitting
/>
