<script lang="ts">
  import { redirectIfSettingsNotDefined } from '$lib/utils/redirect';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import CodeMirrorDotrain from '$lib/components/CodeMirrorDotrain.svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { textFileStore } from '$lib/storesGeneric/textFileStore';
  import { orderAdd } from '$lib/services/order';

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
  <svelte:fragment slot="actions"></svelte:fragment>
</PageHeader>

<div class="flex w-full justify-center">
  <div class="mb-4 w-full max-w-screen-xl">
    <div class="flex w-full items-end justify-between">
      <div
        class="flex-3 grow-1 overflow-hidden overflow-ellipsis text-right tracking-tight text-gray-900 dark:text-white"
      >
        {#if $dotrainFile.path}
          {$dotrainFile.path}
        {:else}
          New Order
        {/if}
      </div>
      <div>
        <div class="flex justify-end gap-x-2">
          {#if $dotrainFile.path}
            <ButtonLoading
              loading={$dotrainFile.isSaving}
              color="green"
              on:click={dotrainFile.saveFile}>Save</ButtonLoading
            >
          {/if}
          <ButtonLoading
            loading={$dotrainFile.isSavingAs}
            color="green"
            on:click={dotrainFile.saveFileAs}>Save As</ButtonLoading
          >
          <ButtonLoading
            loading={$dotrainFile.isLoading}
            color="blue"
            on:click={dotrainFile.loadFile}>Load</ButtonLoading
          >
        </div>
      </div>
    </div>

    <div class="my-4 overflow-hidden rounded-lg border dark:border-none">
      <CodeMirrorDotrain
        bind:value={$dotrainFile.text}
        disabled={isSubmitting}
        styles={{ '&': { minHeight: '400px' } }}
      />
    </div>

    <div class="flex justify-end">
      <ButtonLoading
        color="green"
        loading={isSubmitting}
        disabled={$dotrainFile.isEmpty}
        on:click={execute}>Add Order</ButtonLoading
      >
    </div>
  </div>
</div>
