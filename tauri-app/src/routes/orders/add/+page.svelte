<script lang="ts">
  import PageHeader from '$lib/components/PageHeader.svelte';
  import CodeMirrorDotrain from '$lib/components/CodeMirrorDotrain.svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { textFileStore } from '$lib/storesGeneric/textFileStore';
  import { orderAdd } from '$lib/services/order';
  import FileTextarea from '$lib/components/FileTextarea.svelte';
  import { Helper, Label } from 'flowbite-svelte';
  import InputBlockNumber from '$lib/components/InputBlockNumber.svelte';
  import { forkBlockNumber } from '$lib/stores/forkBlockNumber';
  import { invoke } from '@tauri-apps/api';

  import { activeChainSettingsIndex } from '$lib/stores/settings';
  import DropdownRadio from '$lib/components/DropdownRadio.svelte';
  import SkeletonRow from '$lib/components/SkeletonRow.svelte';

  import {
    settings,
    settingsText,
  } from '$lib/stores/settings';

  let isSubmitting = false;

  $: dotrainFile = textFileStore('Rain', ['rain']);
  $: topConfig = $settings.chains;
  let config = topConfig;
  $: {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    invoke("merge_config", {dotrain: $dotrainFile.text, topConfig}).then(v => config = v as any);
  };

  $: order = config[0];
  $: scenario = config[1];


  async function execute() {
    isSubmitting = true;
    try {
      await orderAdd($dotrainFile.text);
      // eslint-disable-next-line no-empty
    } catch (e) {}
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

    <Label>Order</Label>
    {#if $settings === undefined || $settings.chains.length === 0}
      <SkeletonRow />
    {:else}
      <DropdownRadio options={$settings.chains || []} bind:value={$activeChainSettingsIndex}>
        <svelte:fragment slot="content" let:selected>
          {selected.label ? selected.label : selected.rpc_url}
        </svelte:fragment>

        <svelte:fragment slot="option" let:option>
          {#if option.label}
            <div class="w-full overflow-hidden overflow-ellipsis">
              <div class="text-md mb-2 break-word">{option.label}</div>
              <Helper class="text-xs overflow-hidden overflow-ellipsis break-all">{option.rpc_url}</Helper>
            </div>
          {:else}
            <div class="w-full text-xs overflow-hidden overflow-ellipsis break-all">
              {option.rpc_url}
            </div>
          {/if}
        </svelte:fragment>
      </DropdownRadio>
    {/if}
</FileTextarea>

<div class="my-8">
  <Label class="mb-2">Parse at Block Number</Label>
  <InputBlockNumber bind:value={$forkBlockNumber.value} isFetching={$forkBlockNumber.isFetching} on:clickGetLatest={forkBlockNumber.fetch} required={false} />
  <Helper class="mt-2 text-sm">
    The block number at which to parse the rain while drafting. Resets to
    the latest block on app launch.
  </Helper>
</div>