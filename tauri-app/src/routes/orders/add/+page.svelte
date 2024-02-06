<script lang="ts">
  import { redirectIfSettingsNotDefined } from '$lib/utils/redirect';
  import { ordersList } from '$lib/stores/ordersList';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import CodeMirrorDotrain from '$lib/components/CodeMirrorDotrain.svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { loadDotrainFile } from '$lib/utils/dotrain';
  import { toasts} from '$lib/stores/toasts';

  let isLoading = false;
  let dotrain: string;

  async function openFile() {
    isLoading = true
    try {
      dotrain = await loadDotrainFile();
    } catch(e) {
      toasts.error(e as string);
    }
    isLoading = false
  }

  redirectIfSettingsNotDefined();
  ordersList.fetchPage(1);
</script>

<PageHeader title="Add Order">
  <svelte:fragment slot="actions">
    <ButtonLoading size="sm" loading={isLoading} color="blue" on:click={openFile}>Open a Dotrain File</ButtonLoading>
  </svelte:fragment>
</PageHeader>

<div class="mb-4">
	<h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
		Order Strategy
	</h5>
  <CodeMirrorDotrain bind:value={dotrain} />
</div>

<div class="flex justify-end">
  <ButtonLoading color="green" size="xl">Publish Order</ButtonLoading>
</div>
