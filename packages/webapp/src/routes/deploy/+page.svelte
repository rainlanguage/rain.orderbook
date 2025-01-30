<script lang="ts">
	import { StrategySection } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { Button, Input, Spinner, Toggle, Textarea } from 'flowbite-svelte';
	const { files } = $page.data;

	let _registryUrl = '';
	let _files = files;
	let _dotrain = '';

	let error = '';
	let errorDetails = '';
	let loading = false;
	let advancedMode = false;

	const getFileRegistry = async (url: string) => {
		loading = true;
		try {
			const response = await fetch(url);
			const files = await response.text();
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

<div class="flex w-full flex-col">
	<div class="flex items-start justify-end gap-4">
		{#if advancedMode}
			<div class="mb-12 flex w-2/3 items-start gap-4">
				<div class="flex w-full flex-col gap-4">
					<Input
						id="strategy-url"
						type="url"
						placeholder="Enter URL to raw strategy registry file"
						bind:value={_registryUrl}
					/>
					<Textarea id="textarea-id" placeholder="Raw strategy" rows="8" bind:value={_dotrain} />
				</div>
				<Button on:click={() => getFileRegistry(_registryUrl)}>Load</Button>
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
	{:else if _files.length > 0}
		<div class="flex flex-col gap-36">
			{#each _files as { name, url }}
				<StrategySection strategyUrl={url} strategyName={name} />
			{/each}
		</div>
	{/if}
</div>
