<script lang="ts">
	import { StrategySection } from '@rainlanguage/ui-components';
	import { Button, Input, Spinner, Toggle, Textarea } from 'flowbite-svelte';
	import { registryUrl } from '$lib/stores/registry';
	import { getFileRegistry } from './getFileRegistry';
	import { onMount } from 'svelte';
	import { rawDotrain } from '$lib/stores/raw-dotrain';

	let files: { name: string; url: string }[] = [];
	let inputDotrain = '';
	let error = '';
	let errorDetails = '';
	let loading = false;
	let advancedMode = false;

	onMount(() => {
		fetchFilesFromRegistry($registryUrl);
	});

	const fetchFilesFromRegistry = async (url: string) => {
		loading = true;
		try {
			files = await getFileRegistry(url);
		} catch (e) {
			error = 'Error getting registry';
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
		}
		loading = false;
	};

	const loadStrategy = () => {
		if (inputDotrain.trim()) {
			files = [];
			$rawDotrain = inputDotrain;
			inputDotrain = '';
		}
	};
</script>

<div class="flex w-full flex-col">
	<div class="flex items-start justify-end gap-4">
		{#if advancedMode}
			<div class="mb-12 flex w-2/3 flex-col items-start gap-4">
				<div class="flex w-full items-start gap-4">
					<Input
						id="strategy-url"
						type="url"
						placeholder="Enter URL to raw strategy registry file"
						bind:value={$registryUrl}
					/>
					<Button class="text-nowrap" on:click={() => fetchFilesFromRegistry($registryUrl)}>
						Load URL
					</Button>
				</div>
				<div class="flex w-full items-start gap-4">
					<Textarea
						id="textarea-id"
						placeholder="Raw strategy"
						rows="8"
						bind:value={inputDotrain}
					/>
					<Button class="text-nowrap" on:click={loadStrategy}>Load Strategy</Button>
				</div>
			</div>
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
	{/if}
	{#if files.length > 0}
		<div class="mb-36 flex flex-col gap-8">
			{#each files as { name, url }}
				<StrategySection strategyUrl={url} strategyName={name} />
			{/each}
		</div>
	{:else if $rawDotrain}
		<StrategySection rawDotrain={$rawDotrain} strategyName={'raw'} />
	{/if}
</div>
