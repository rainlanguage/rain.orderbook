<script lang="ts">
  import { redirectIfSettingsNotDefined } from '$lib/utils/redirect';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import CodeMirrorDotrain from '$lib/components/CodeMirrorDotrain.svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { textFileStore } from '$lib/storesGeneric/textFileStore';
  import { orderAdd } from '$lib/utils/orderAdd';
  import { Card } from 'flowbite-svelte';

  let isSubmitting = false;

  const dotrainFile = textFileStore('Rain', 'rain');

  async function execute() {
    isSubmitting = true;
    try {
      await orderAdd($dotrainFile.text);
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }

  redirectIfSettingsNotDefined();
</script>

<PageHeader title="Add Order">
  <svelte:fragment slot="actions">
    {#if $dotrainFile.path}
      <ButtonLoading size="xs" loading={$dotrainFile.isSaving} color="green" on:click={dotrainFile.saveFile}>Save</ButtonLoading>
    {/if}
    <ButtonLoading size="xs" loading={$dotrainFile.isSavingAs} color="green" on:click={dotrainFile.saveFileAs}>Save As</ButtonLoading>
    <ButtonLoading size="xs" loading={$dotrainFile.isLoading} color="blue" on:click={dotrainFile.loadFile}>Load File</ButtonLoading>
  </svelte:fragment>
</PageHeader>


<div class="flex justify-center w-full">
  <div class="w-full max-w-screen-xl mb-4">
    <div class="flex justify-between items-end w-full pr-2">
      <h5 class="text-xl font-bold tracking-tight text-gray-900 dark:text-white min-w-48 grow-0">
        Order Rain
      </h5>
      {#if $dotrainFile.path}
        <div class="text-sm tracking-tight text-gray-900 dark:text-white text-right overflow-hidden overflow-ellipsis flex-3 grow-1">{$dotrainFile.path}</div>
      {/if}
    </div>

    <Card size="xl" class="w-full mb-4 mt-0">
      <CodeMirrorDotrain bind:value={$dotrainFile.text} disabled={isSubmitting} styles={{"&": {minHeight: "400px"}}} />
    </Card>


    <div class="flex justify-end">
      <ButtonLoading color="green" size="xl" loading={isSubmitting} disabled={$dotrainFile.isEmpty} on:click={execute}>Add Order</ButtonLoading>
    </div>
  </div>
</div>
