<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { Button, Dropdown, Label, Checkbox, Input } from 'flowbite-svelte';
	import { ChevronDownSolid, SearchSolid } from 'flowbite-svelte-icons';
	import { isEmpty } from 'lodash';
	import type { SgErc20WithSubgraphName } from '@rainlanguage/orderbook';
	import type { AppStoresInterface } from '../../types/appStores';
	import type { Readable } from 'svelte/store';
	import type { QueryObserverResult } from '@tanstack/svelte-query';

	const dispatch = createEventDispatcher();

	export let tokensQuery: Readable<QueryObserverResult<SgErc20WithSubgraphName[], Error>>;
	export let activeTokens: AppStoresInterface['activeTokens'];

	export let label: string = 'Filter by tokens';
	export let allLabel: string = 'All tokens';
	export let emptyMessage: string = 'No tokens available';
	export let loadingMessage: string = 'Loading tokens...';

	let filteredTokens: SgErc20WithSubgraphName[] = [];
	let searchTerm: string = '';
	let selectedIndex = 0; // Selected item index

	$: availableTokens = ($tokensQuery?.data as SgErc20WithSubgraphName[]) || [];

	$: selectedCount = $activeTokens.length;
	$: allSelected = selectedCount === filteredTokens.length && filteredTokens.length > 0;
	$: buttonText =
		selectedCount === 0
			? 'Select tokens'
			: allSelected
				? allLabel
				: `${selectedCount} token${selectedCount > 1 ? 's' : ''}`;

	// Filter tokens based on search term
	$: {
		if (searchTerm.trim() === '') {
			filteredTokens = availableTokens;
		} else {
			const term = searchTerm.toLowerCase();
			filteredTokens = availableTokens.filter(
				(token) =>
					token.token.symbol?.toLowerCase().includes(term) ||
					token.token.name?.toLowerCase().includes(term) ||
					token.token.address?.toLowerCase().includes(term)
			);
			// Select first element in the list automatically if there are any results
			selectedIndex = filteredTokens.length > 0 ? 0 : -1;
		}
	}

	function updateSelectedTokens(newSelection: SgErc20WithSubgraphName['token']['address'][]) {
		activeTokens.set(newSelection);
		dispatch('change', activeTokens);
	}

	function toggleToken({ token }: SgErc20WithSubgraphName) {
		if (!token.address) return;

		const idx = $activeTokens.indexOf(token.address);
		const newSelection =
			idx !== -1
				? [...$activeTokens.slice(0, idx), ...$activeTokens.slice(idx + 1)]
				: [...$activeTokens, token.address];

		updateSelectedTokens(newSelection);
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (!filteredTokens.length) return;

		switch (event.key) {
			case 'Enter':
				event.preventDefault();
				if (filteredTokens.length > 0) {
					const targetIndex = selectedIndex >= 0 ? selectedIndex : 0;
					const tokenToToggle = filteredTokens[targetIndex];
					if (tokenToToggle) {
						toggleToken(tokenToToggle);
					}
				}
				break;
			case 'ArrowDown':
				event.preventDefault();
				selectedIndex = Math.min(selectedIndex + 1, filteredTokens.length - 1);
				break;
			case 'ArrowUp':
				event.preventDefault();
				selectedIndex = Math.max(selectedIndex - 1, 0);
				break;
			case 'Escape':
				searchTerm = '';
				selectedIndex = -1;
				break;
		}
	}
</script>

<div class="flex flex-col gap-x-2">
	<Label>{label}</Label>
	<div>
		<Button
			color="alternative"
			class="flex w-full justify-between overflow-hidden pl-2 pr-0 text-left"
			data-testid="dropdown-tokens-filter-button"
		>
			<div class="w-[90px] overflow-hidden text-ellipsis whitespace-nowrap">
				{buttonText}
			</div>
			<ChevronDownSolid class="mx-2 h-3 w-3 text-black dark:text-white" />
		</Button>

		<Dropdown
			class="max-h-[75vh] w-full min-w-60 overflow-y-auto py-0"
			data-testid="dropdown-tokens-filter"
		>
			{#if $tokensQuery.isLoading}
				<div class="ml-2 w-full rounded-lg p-3">{loadingMessage}</div>
			{:else if $tokensQuery.isError}
				<div class="ml-2 w-full rounded-lg p-3 text-red-500">
					Cannot load tokens list: {$tokensQuery.error?.message || 'Unknown error'}
				</div>
			{:else if isEmpty(availableTokens)}
				<div class="ml-2 w-full rounded-lg p-3">{emptyMessage}</div>
			{:else}
				<!-- Search input -->
				<div class="sticky top-0 bg-white p-3 dark:bg-gray-800">
					<Input
						placeholder="Search tokens..."
						bind:value={searchTerm}
						autofocus
						on:keydown={handleKeyDown}
						data-testid="tokens-filter-search"
					>
						<SearchSolid slot="left" class="h-4 w-4 text-gray-500" />
					</Input>
				</div>

				{#if isEmpty(filteredTokens)}
					<div class="ml-2 w-full rounded-lg p-3">No tokens match your search</div>
				{:else}
					{#each filteredTokens as token, index}
						<Checkbox
							data-testid="dropdown-tokens-filter-option"
							class="w-full rounded-lg p-3 hover:bg-gray-100 dark:hover:bg-gray-600 {selectedIndex ===
							index
								? 'bg-blue-100 dark:bg-blue-900'
								: ''}"
							on:click={() => toggleToken(token)}
							checked={!!(token.token.address && $activeTokens.indexOf(token.token.address) > -1)}
						>
							<div class="ml-2 flex w-full">
								<div class="flex-1 text-sm font-medium">{token.token.symbol || 'Unknown'}</div>
								<div class="text-xs text-gray-500">
									{token.subgraphName}
								</div>
							</div>
						</Checkbox>
					{/each}
				{/if}
			{/if}
		</Dropdown>
	</div>
</div>
