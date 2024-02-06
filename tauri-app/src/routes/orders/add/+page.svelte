<script lang="ts">
  import { redirectIfSettingsNotDefined } from '$lib/utils/redirect';
  import { ordersList } from '$lib/stores/ordersList';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import CodeMirrorDotrain from '$lib/components/CodeMirrorDotrain.svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { loadDotrainFile } from '$lib/utils/dotrain';
  import { toasts } from '$lib/stores/toasts';
  import { orderAdd } from '$lib/utils/orderAdd';

  let dotrain: string = '';
  let isOpening = false;
  let isSubmitting = false;

  $: isEmpty = dotrain.length === 0;

  async function openFile() {
    isOpening = true
    try {
      dotrain = await loadDotrainFile();
    } catch(e) {
      toasts.error(e as string);
    }
    isOpening = false
  }

  async function execute() {
    isSubmitting = true;
    try {
      await orderAdd(dotrain);
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }

  redirectIfSettingsNotDefined();
  ordersList.fetchPage(1);
</script>

<PageHeader title="Add Order">
  <svelte:fragment slot="actions">
    <ButtonLoading size="xs" loading={isOpening} color="blue" on:click={openFile}>Load Dotrain File</ButtonLoading>
  </svelte:fragment>
</PageHeader>

<div class="mb-4">
	<h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
		Order Strategy
	</h5>
  <CodeMirrorDotrain bind:value={dotrain} disabled={isSubmitting} />
</div>

<div class="flex justify-end">
  <ButtonLoading color="green" size="xl" loading={isSubmitting} disabled={isEmpty} on:click={execute}>Add Order</ButtonLoading>
</div>
