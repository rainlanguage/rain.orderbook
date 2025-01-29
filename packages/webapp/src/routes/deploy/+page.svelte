<script lang="ts">
	import { StrategySection } from '@rainlanguage/ui-components';
	import { Button, Input, Spinner, Toggle } from 'flowbite-svelte';
	import { registryUrl } from '$lib/stores/registry';
	import { onMount } from 'svelte';

	let files: { name: string; url: string }[] = [];

	let error = '';
	let errorDetails = '';
	let loading = false;
	let advancedMode = false;

	onMount(() => {
		getFileRegistry($registryUrl);
	});

	const getFileRegistry = async (url: string) => {
		loading = true;
		console.log('getting file registry', url);

		try {
			const response = await fetch(url);
			if (!response.ok) {
				throw new Error('Failed to fetch registry');
			}
			const filesList = await response.text();
			files = filesList
				.split('\n')
				.filter((line) => line.trim())
				.map((line) => {
					const [name, url] = line.split(' ');
					return { name, url };
				});
		} catch (e) {
			files = [];
			error = 'Error getting registry';
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
		}
		return (loading = false);
	};
</script>

<div class="flex flex-col">
	<div class="flex h-12 w-full items-center justify-end gap-4">
		{#if advancedMode}
			<Input
				id="strategy-url"
				type="url"
				placeholder="Enter URL to raw strategy registry file"
				bind:value={$registryUrl}
				class="max-w-lg"
			/>
			<Button on:click={() => getFileRegistry($registryUrl)}>Load</Button>
		{/if}
		<Toggle on:change={() => (advancedMode = !advancedMode)}>
			{'Advanced Mode'}
		</Toggle>
	</div>

	{#if loading}
		<Spinner />
	{:else if error}
		<p>{error}</p>
		<p>{errorDetails}</p>
	{:else if files.length > 0}
		<div class="flex flex-col gap-36">
			{#each files as { name, url }}
				<StrategySection strategyUrl={url} strategyName={name} />
			{/each}
		</div>
	{/if}
</div>
