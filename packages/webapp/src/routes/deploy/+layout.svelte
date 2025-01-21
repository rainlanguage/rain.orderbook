<script lang="ts">

	import { Button, Dropdown, DropdownItem, Input, Spinner } from 'flowbite-svelte';
	import { ChevronDownOutline } from 'flowbite-svelte-icons';

	import { goto } from '$app/navigation';
	import { page } from '$app/stores';

	const { files, strategyName, strategyUrl } = $page.data;

	let isLoading = false;
	let debounceTimer: ReturnType<typeof setTimeout>;
	let dropdownOpen = false;
	let _strategyUrl = strategyUrl;
	let _strategyName = strategyName;

	$: if (strategyUrl) {
		isLoading = true;
		clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => {
			isLoading = false;
			goto(`/deploy/${_strategyName}`);
		}, 1000); // 500ms delay
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
				<Dropdown bind:open={dropdownOpen} bind:value={_strategyUrl}>
					{#each files as file}
						<DropdownItem
							active={strategyUrl === file.download_url}
							on:click={() => {
								_strategyUrl = file.download_url;
								_strategyName = file.name;
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
			bind:value={_strategyUrl}
			size="lg"
			class="max-w-xl"
		/>
	</div>
	{#if isLoading}
		<Spinner />
	{:else}
		<slot />
	{/if}
</div>
