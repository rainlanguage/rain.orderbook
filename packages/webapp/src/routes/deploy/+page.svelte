<script lang="ts">
	import { DeploymentSteps } from '@rainlanguage/ui-components';
	import { Button, Dropdown, DropdownItem, Input, Spinner } from 'flowbite-svelte';
	import { CheckCircleOutline, ChevronDownOutline } from 'flowbite-svelte-icons';
	import { fade } from 'svelte/transition';

	enum StrategyLoadErrors {
		NO_STRATEGY = 'No valid strategy exists at this URL'
	}

	export let data;
	let strategyUrl = '';
	let isLoading = false;
	let error: StrategyLoadErrors | null = null;
	let errorDetails: string | null = null;
	let dotrain = '';
	let strategyName = '';
	let debounceTimer: ReturnType<typeof setTimeout>;
	let dropdownOpen = false;

	const { files } = data;

	$: if (strategyUrl) {
		isLoading = true;
		clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => {
			loadStrategyFromUrl();
		}, 1000); // 500ms delay
	}

	async function loadStrategyFromUrl() {
		error = null;
		errorDetails = null;

		try {
			const response = await fetch(strategyUrl);
			if (!response.ok) {
				throw new Error(`HTTP error - status: ${response.status}`);
			}
			dotrain = await response.text();
		} catch (e) {
			error = StrategyLoadErrors.NO_STRATEGY;
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
			console.error('Failed to load strategy:', e);
		} finally {
			isLoading = false;
		}
	}
</script>

<div class="flex flex-col justify-center gap-6">
	<div class="flex flex-row items-center gap-6">
		{#if files.length > 0}
			<div class="min-w-xl flex">
				<Button size="lg" class="mix-w-xl"
					>Select a strategy<ChevronDownOutline
						class="ms-2 flex h-3 w-3 text-white dark:text-white"
					/></Button
				>
				<Dropdown bind:open={dropdownOpen}>
					{#each files as file}
						<DropdownItem
							active={strategyUrl === file.download_url}
							on:click={() => {
								strategyUrl = file.download_url;
								strategyName = file.name;
								dropdownOpen = false;
							}}
						>
							{file.name}
						</DropdownItem>
					{/each}
				</Dropdown>
			</div>
			or
		{/if}

		<Input
			id="strategy-url"
			type="url"
			placeholder="Enter URL to .rain file"
			bind:value={strategyUrl}
			size="lg"
			class="max-w-xl"
		/>
	</div>
	{#if isLoading}
		<Spinner />
	{:else if error}
		{error}
		{errorDetails}
	{:else if dotrain}
		<div in:fade class="flex flex-col gap-4">
			<div class="mb-6 flex items-center gap-2">
				<CheckCircleOutline class="h-5 w-5 text-green-500" />
				<p>Strategy loaded: {strategyName}</p>
			</div>

			<DeploymentSteps {dotrain} />
		</div>
	{/if}
</div>
