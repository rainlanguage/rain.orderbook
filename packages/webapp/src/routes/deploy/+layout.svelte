<script lang="ts">
	import CustomRegistryWarning from '$lib/components/CustomRegistryWarning.svelte';
	import { InputRegistryUrl, PageHeader } from '@rainlanguage/ui-components';
	import { Toggle } from 'flowbite-svelte';
	import { page } from '$app/stores';

	let advancedMode = localStorage.getItem('registry') ? true : false;
	$: customRegistry = $page.url.searchParams.get('registry');
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
		<div class="flex w-2/3 flex-col items-start gap-4">
			<InputRegistryUrl />
		</div>
	{/if}
</div>
<slot></slot>
