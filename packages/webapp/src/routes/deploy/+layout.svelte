<script lang="ts">
	import CustomRegistryWarning from '$lib/components/CustomRegistryWarning.svelte';
	import {
		InputRegistryUrl,
		PageHeader,
		RegistryProvider,
		RegistryManager,
		type RegistryStore
	} from '@rainlanguage/ui-components';
	import { Toggle } from 'flowbite-svelte';
	import { page } from '$app/stores';

	import { onMount } from 'svelte';
	import { writable } from 'svelte/store';
	import { REGISTRY_URL } from '$lib/constants';
	import { slide } from 'svelte/transition';
	const { registryFromUrl } = $page.data;
	let advancedMode = false;

	let registryError = '';
	let customRegistry = false;

	const registryManager = new RegistryManager(registryFromUrl || REGISTRY_URL);
	const registryManagerStore = writable(registryManager);
	onMount(() => {
		try {
			advancedMode = registryManager.isCustomRegistry();
			customRegistry = advancedMode;
		} catch (e) {
			registryError = e instanceof Error ? e.message : 'Failed to initialize registry manager';
		}
	});

	$: console.log('registryManagerStore', $registryManagerStore);
</script>

<RegistryProvider {registryManagerStore}>
	{#if $registryManagerStore}
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
				<div in:slide class="w-full">
					<InputRegistryUrl />
				</div>
			{/if}
			<div class="h-4">
				{#if registryError}
					<p data-testid="registry-error" class="text-red-500">{registryError}</p>
				{/if}
			</div>
		</div>
		<slot></slot>
	{/if}
</RegistryProvider>
