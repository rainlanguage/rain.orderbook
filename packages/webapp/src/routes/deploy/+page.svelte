<script lang="ts">
	import { StrategySection } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { Button, Input, Spinner, Toggle } from 'flowbite-svelte';
	const { files } = $page.data;

	let _registryUrl = '';
	let _files = files;

	let error = '';
	let errorDetails = '';
	let loading = false;
	let advancedMode = false;

	const getFileRegistry = async (url: string) => {
		loading = true;
		try {
			const response = await fetch(url);
			const files = await response.text();
			console.log(files);
			// Parse the response text into array of {name, url} objects
			_files = files
				.split('\n')
				.filter((line) => line.trim())
				.map((line) => {
					const [name, url] = line.split(' ');
					return { name, url };
				});
		} catch (e) {
			error = 'Error getting registry';
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
		}
		loading = false;
		return;
	};
</script>

<div class="flex flex-col gap-24">
	<div class="flex h-12 w-full items-center justify-end gap-4">
		{#if advancedMode}
			<Input
				id="strategy-url"
				type="url"
				placeholder="Enter URL to raw strategy registry file"
				bind:value={_registryUrl}
				class="max-w-lg"
			/>
			<Button on:click={() => getFileRegistry(_registryUrl)}>Load</Button>
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
	{:else if _files.length > 0}
		{#each _files as { name, url }}
			<StrategySection strategyUrl={url} strategyName={name} />
		{/each}
	{/if}
</div>
