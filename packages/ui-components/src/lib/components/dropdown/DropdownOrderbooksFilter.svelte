<script lang="ts">
	import { Button, Dropdown, Label, Checkbox, Input } from 'flowbite-svelte';
	import { ChevronDownSolid, SearchSolid } from 'flowbite-svelte-icons';
	import { isEmpty } from 'lodash';
	import type { Address, OrderbookCfg } from '@rainlanguage/orderbook';
	import type { AppStoresInterface } from '../../types/appStores';
	import { getNetworkName } from '$lib/utils/getNetworkName';
	import { useRaindexClient } from '$lib/hooks/useRaindexClient';

	export let activeOrderbookAddresses: AppStoresInterface['activeOrderbookAddresses'];
	export let selectedOrderbookAddresses: Address[];
	export let selectedChainIds: number[];

	export let label: string = 'Filter by orderbook';
	export let allLabel: string = 'All orderbooks';
	export let emptyMessage: string = 'No orderbooks available';

	interface OrderbookItem {
		key: string;
		address: Address;
		label: string | undefined;
		chainId: number;
	}

	const raindexClient = useRaindexClient();

	let filteredOrderbooks: OrderbookItem[] = [];
	let searchTerm: string = '';
	let selectedIndex = 0;

	$: orderbooksResult = raindexClient.getAllOrderbooks();
	$: orderbooksMap = orderbooksResult?.value ?? new Map<string, OrderbookCfg>();
	$: orderbooksError = orderbooksResult?.error;

	$: availableOrderbooks = (() => {
		const items: OrderbookItem[] = [];
		orderbooksMap.forEach((cfg: OrderbookCfg, key: string) => {
			if (selectedChainIds.length === 0 || selectedChainIds.includes(cfg.network.chainId)) {
				items.push({
					key,
					address: cfg.address as Address,
					label: cfg.label,
					chainId: cfg.network.chainId
				});
			}
		});
		return items;
	})();

	$: selectedCount = selectedOrderbookAddresses.length;

	$: allSelected = selectedCount === availableOrderbooks.length && availableOrderbooks.length > 0;
	$: buttonText =
		selectedCount === 0
			? 'Select orderbooks'
			: allSelected
				? allLabel
				: `${selectedCount} orderbook${selectedCount > 1 ? 's' : ''}`;

	$: {
		if (searchTerm.trim() === '') {
			filteredOrderbooks = availableOrderbooks;
		} else {
			const term = searchTerm.toLowerCase();
			filteredOrderbooks = availableOrderbooks.filter(
				(ob) =>
					ob.label?.toLowerCase().includes(term) ||
					ob.address?.toLowerCase().includes(term) ||
					ob.key?.toLowerCase().includes(term)
			);
			selectedIndex = filteredOrderbooks.length > 0 ? 0 : -1;
		}
	}

	$: sortedFilteredOrderbooks = [...filteredOrderbooks].sort((a, b) => {
		const aSelected = selectedOrderbookAddresses.some(
			(addr) => addr.toLowerCase() === a.address.toLowerCase()
		);
		const bSelected = selectedOrderbookAddresses.some(
			(addr) => addr.toLowerCase() === b.address.toLowerCase()
		);
		if (aSelected === bSelected) return 0;
		return aSelected ? -1 : 1;
	});

	function getDisplayName(ob: OrderbookItem): string {
		const truncatedAddr = `${ob.address.slice(0, 6)}...${ob.address.slice(-4)}`;
		return ob.label ? `${ob.label} (${truncatedAddr})` : truncatedAddr;
	}

	function updateSelectedOrderbooks(newSelection: Address[]) {
		activeOrderbookAddresses.set(newSelection);
	}

	function toggleOrderbook({ address }: OrderbookItem) {
		if (!address) return;

		const normalizedAddress = address.toLowerCase() as Address;
		const isSelected = selectedOrderbookAddresses.some(
			(addr) => addr.toLowerCase() === normalizedAddress
		);
		const newSelection = isSelected
			? $activeOrderbookAddresses.filter((addr) => addr.toLowerCase() !== normalizedAddress)
			: [...$activeOrderbookAddresses, normalizedAddress];

		updateSelectedOrderbooks(newSelection);
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (!sortedFilteredOrderbooks.length) return;

		switch (event.key) {
			case 'Enter':
				event.preventDefault();
				if (sortedFilteredOrderbooks.length > 0) {
					const orderbookToToggle = sortedFilteredOrderbooks[selectedIndex];
					if (orderbookToToggle) {
						toggleOrderbook(orderbookToToggle);
					}
				}
				break;
			case 'ArrowDown':
				event.preventDefault();
				selectedIndex = Math.min(selectedIndex + 1, sortedFilteredOrderbooks.length - 1);
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
			data-testid="dropdown-orderbooks-filter-button"
			aria-label="Select orderbooks to filter"
			aria-expanded="false"
			aria-haspopup="listbox"
		>
			<div class="w-[110px] overflow-hidden text-ellipsis whitespace-nowrap">
				{buttonText}
			</div>
			<ChevronDownSolid class="mx-2 h-3 w-3 text-black dark:text-white" />
		</Button>

		<Dropdown
			class="max-h-[75vh] w-full min-w-60 overflow-y-auto py-0"
			data-testid="dropdown-orderbooks-filter"
		>
			{#if orderbooksError}
				<div class="ml-2 w-full rounded-lg p-3 text-red-500">
					Cannot load orderbooks list: {orderbooksError.readableMsg || 'Unknown error'}
				</div>
			{:else if isEmpty(availableOrderbooks)}
				<div class="ml-2 w-full rounded-lg p-3">{emptyMessage}</div>
			{:else}
				<div class="sticky top-0 bg-white p-3 dark:bg-gray-800">
					<Input
						placeholder="Search orderbooks..."
						bind:value={searchTerm}
						autofocus
						on:keydown={handleKeyDown}
						data-testid="orderbooks-filter-search"
					>
						<SearchSolid slot="left" class="h-4 w-4 text-gray-500" />
					</Input>
				</div>

				{#if isEmpty(filteredOrderbooks)}
					<div class="ml-2 w-full rounded-lg p-3">No orderbooks match your search</div>
				{:else}
					{#each sortedFilteredOrderbooks as orderbook, index (`${orderbook.address}-${orderbook.chainId}`)}
						<Checkbox
							data-testid="dropdown-orderbooks-filter-option"
							class="w-full rounded-lg p-3 hover:bg-gray-100 dark:hover:bg-gray-600 {selectedIndex ===
							index
								? 'bg-blue-100 dark:bg-blue-900'
								: ''}"
							on:click={() => toggleOrderbook(orderbook)}
							checked={!!(
								orderbook.address &&
								selectedOrderbookAddresses.some(
									(addr) => addr.toLowerCase() === orderbook.address.toLowerCase()
								)
							)}
						>
							<div class="ml-2 flex w-full">
								<div class="flex-1 text-sm font-medium">{getDisplayName(orderbook)}</div>
								<div class="text-xs text-gray-500">
									{getNetworkName(orderbook.chainId)}
								</div>
							</div>
						</Checkbox>
					{/each}
				{/if}
			{/if}
		</Dropdown>
	</div>
</div>
