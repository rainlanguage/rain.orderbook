<script lang="ts">
	import { writable } from 'svelte/store';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import CodeMirrorDotrain from '$lib/components/CodeMirrorDotrain.svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { textFileStore } from '$lib/storesGeneric/textFileStore';
  import { orderAdd } from '$lib/services/order';
  import FileTextarea from '$lib/components/FileTextarea.svelte';
  import { Helper, Label, Button } from 'flowbite-svelte';
  import InputBlockNumber from '$lib/components/InputBlockNumber.svelte';
  import { forkBlockNumber } from '$lib/stores/forkBlockNumber';
  import ObservableChart from '$lib/components/ObservableChart.svelte';
  import { fuzz } from '$lib/services/fuzz';

  let isSubmitting = false;
  let result = writable();

  const dotrainFile = textFileStore('Rain', ['rain']);

  async function execute() {
    isSubmitting = true;
    try {
      await orderAdd($dotrainFile.text);
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }

  async function fuzzIt() {
    $result = await fuzz("mumbai", $dotrainFile.text, "");
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
</FileTextarea>

<Button on:click={fuzzIt}>Fuzz it baby</Button>

<ObservableChart rawData={$result} />

<div class="my-8">
  <Label class="mb-2">Parse at Block Number</Label>
  <InputBlockNumber bind:value={$forkBlockNumber.value} isFetching={$forkBlockNumber.isFetching} on:clickGetLatest={forkBlockNumber.fetch} required={false} />
  <Helper class="mt-2 text-sm">
    The block number at which to parse the rain while drafting. Resets to
    the latest block on app launch.
  </Helper>
</div>