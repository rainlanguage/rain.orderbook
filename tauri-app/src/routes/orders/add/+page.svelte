<script lang="ts">
  import PageHeader from '$lib/components/PageHeader.svelte';
  import CodeMirrorDotrain from '$lib/components/CodeMirrorDotrain.svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { textFileStore } from '$lib/storesGeneric/textFileStore';
  import { orderAdd } from '$lib/services/order';
  import FileTextarea from '$lib/components/FileTextarea.svelte';

  let isSubmitting = false;

  const dotrainFile = textFileStore('Rain', ['rain']);

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

<div class="flex w-full justify-center">
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
  </FileTextarea>
</div>
