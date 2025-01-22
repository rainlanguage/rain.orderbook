<script lang="ts">
	import { Button, Dropdown, Input, Spinner, Radio } from 'flowbite-svelte';
	import { ChevronDownOutline } from 'flowbite-svelte-icons';

	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { fade } from 'svelte/transition';
	const { files, strategyName, strategyUrl, deployment } = $page.data;

	let isLoading = false;
	let debounceTimer: ReturnType<typeof setTimeout>;
	let dropdownOpen = false;
	let selectedStrategy = strategyUrl;
	let _strategyUrl = strategyUrl;
	let _strategyName = strategyName;

	$: if (selectedStrategy) {
		isLoading = true;
		clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => {
			isLoading = false;
			_strategyUrl = selectedStrategy;
			if (deployment) {
				goto(`/deploy/${_strategyName}/${deployment}`);
			} else {
				goto(`/deploy/${_strategyName}`);
			}
			return () => clearTimeout(debounceTimer);
		}, 500);
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
				<Dropdown bind:open={dropdownOpen} class="w-44 space-y-3 p-3 text-sm">
					{#each files as file}
						<li>
							<Radio
								name="strategy-select"
								bind:group={selectedStrategy}
								value={file.download_url}
								on:change={() => {
									_strategyName = file.name;
									dropdownOpen = false;
								}}
							>
								{file.name}
							</Radio>
						</li>
					{/each}
				</Dropdown>
			</div>
			or
		{/if}

		<Input
			id="strategy-url"
			type="url"
			placeholder="Enter URL to .rain file"
			bind:value={_strategyUrl}
			size="lg"
			class="max-w-xl"
		/>
	</div>
	{#if isLoading}
		<Spinner />
	{:else}<div in:fade>
			<slot />
		</div>
	{/if}
</div>
