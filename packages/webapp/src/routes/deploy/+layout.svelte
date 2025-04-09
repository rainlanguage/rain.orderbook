<script lang="ts">
	import CustomRegistryWarning from '$lib/components/CustomRegistryWarning.svelte';
	import { InputRegistryUrl, PageHeader } from '@rainlanguage/ui-components';
	import { Toggle } from 'flowbite-svelte';
	import { page } from '$app/stores';
	import RegistryManager from '$lib/services/RegistryManager';
	import { onMount } from 'svelte';
	import { loadRegistryUrl } from '$lib/services/loadRegistryUrl';

	let advancedMode = false;
	let registryFromStorage: string | null = null;
	let registryError = '';
	let customRegistry = false;

	onMount(() => {
		try {
			registryFromStorage = RegistryManager.getFromStorage();
			advancedMode = registryFromStorage ? true : false;
		} catch (e) {
			registryError = e instanceof Error ? e.message : 'Failed to access registry settings';
		}
	});

	$: {
		try {
			customRegistry = RegistryManager.isCustomRegistry(registryFromStorage);
		} catch (e) {
			registryError = e instanceof Error ? e.message : 'Failed to check registry status';
		}
	}

	async function handleLoadRegistryUrl(url: string) {
		try {
			registryError = '';
			await loadRegistryUrl(url);
		} catch (e) {
			registryError = e instanceof Error ? e.message : 'Failed to update registry URL';
		}
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
<div class="flex flex-col items-end">
	{#if advancedMode && $page.url.pathname === '/deploy'}
		<InputRegistryUrl loadRegistryUrl={handleLoadRegistryUrl} />
	{/if}
	<div class="h-4">
		{#if registryError}
			<p data-testid="registry-error" class="text-red-500">{registryError}</p>
		{/if}
	</div>
</div>
<slot></slot>
