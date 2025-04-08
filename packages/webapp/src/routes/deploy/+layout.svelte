<script lang="ts">
	import CustomRegistryWarning from '$lib/components/CustomRegistryWarning.svelte';
	import { InputRegistryUrl, PageHeader } from '@rainlanguage/ui-components';
	import { Toggle } from 'flowbite-svelte';
	import { page } from '$app/stores';
	import RegistryManager from '$lib/services/RegistryManager';

	let advancedMode = localStorage.getItem('registry') ? true : false;
	let registryFromStorage = RegistryManager.getFromStorage();
	$: customRegistry = RegistryManager.isCustomRegistry(registryFromStorage);

	function loadRegistryUrl(url: string) {
		RegistryManager.setToStorage(url);
		RegistryManager.updateUrlParam(url);
		window.location.reload();
	}
</script>

<PageHeader title={$page.data.pageName || 'Deploy'} pathname={$page.url.pathname}>
	<svelte:fragment slot="actions">
		<div class="flex flex-col gap-2">
			{#if $page.url.pathname === '/deploy'}
				<Toggle checked={advancedMode} on:change={() => (advancedMode = !advancedMode)}>
					<span class="whitespace-nowrap">Advanced mode</span>
				</Toggle>
			{/if}
		</div>
	</svelte:fragment>
	<svelte:fragment slot="warning">
		{#if customRegistry}
			<CustomRegistryWarning />
		{/if}
	</svelte:fragment>
</PageHeader>
<div class="flex flex-col items-end gap-4">
	{#if advancedMode && $page.url.pathname === '/deploy'}
		<div class="flex w-full flex-col items-start gap-4 lg:w-2/3">
			<InputRegistryUrl {loadRegistryUrl} />
		</div>
	{/if}
</div>
<slot></slot>
