<script lang="ts">
	import { Input, Button, Modal } from 'flowbite-svelte';
	import { SearchOutline, CheckCircleSolid, ChevronDownSolid, ExclamationTriangleOutline } from 'flowbite-svelte-icons';
	import type { TokenInfo } from '@rainlanguage/orderbook';
	import { useGui } from '$lib/hooks/useGui';
	import { onMount, tick } from 'svelte';
	import { isAddress } from 'viem';

	export let selectedToken: TokenInfo | null = null;
	export let onSelect: (token: TokenInfo) => void;

	let modalOpen = false;
	let searchQuery = '';
	let tokens: TokenInfo[] = [];
	let isSearching = false;
	let customTokenCandidate: TokenInfo | null = null;

	const gui = useGui();

	async function loadTokens(search?: string) {
		isSearching = true;
		customTokenCandidate = null;

		const result = await gui.getAllTokens(search);
		if (result.error) {
			tokens = [];
		} else {
			tokens = result.value;
		}

		// Check if search looks like an address and no tokens found in predefined list
		if (search && isAddress(search) && tokens.length === 0) {
			// Show the address as a custom token candidate
			customTokenCandidate = {
				key: 'custom-token',
				address: search,
				decimals: 18, // Will be updated when validated
				name: 'Unknown Token',
				symbol: 'UNKNOWN'
			};
		}

		isSearching = false;
	}

	function handleSearch(event: Event) {
		const target = event.target as HTMLInputElement;
		searchQuery = target.value;
		loadTokens(searchQuery || undefined);
	}

	onMount(() => loadTokens());

	function formatAddress(address: string): string {
		return `${address.slice(0, 6)}...${address.slice(-4)}`;
	}

	function handleTokenSelect(token: TokenInfo) {
		onSelect(token);
		modalOpen = false;
	}

	$: displayText = selectedToken
		? `${selectedToken.name} (${selectedToken.symbol})`
		: 'Select token...';

	$: if (modalOpen) {
		tick().then(() => {
			const input = document.querySelector('.token-search-input') as HTMLInputElement;
			if (input) {
				input.focus();
			}
		});
	}

	// Reset state when modal closes
	$: if (!modalOpen) {
		searchQuery = '';
		customTokenCandidate = null;
		loadTokens(); // Reload default tokens
	}
</script>

<div class="token-dropdown">
	<div class="relative w-full">
		<Button
			color="alternative"
			class="flex w-full justify-between overflow-hidden overflow-ellipsis pl-4 pr-2 text-left"
			size="lg"
			on:click={() => (modalOpen = true)}
		>
			<div class="flex-grow overflow-hidden">
				<span class="text-gray-900 dark:text-white">{displayText}</span>
			</div>
			<ChevronDownSolid class="ml-2 h-4 w-4 text-black dark:text-white" />
		</Button>

		<Modal bind:open={modalOpen} size="md" class="w-full max-w-lg">
			<div slot="header" class="flex w-full items-center justify-between">
				<h3 class="text-xl font-medium text-gray-900 dark:text-white">Select a token</h3>
			</div>
			<div class="relative w-full border-b border-gray-200 p-2 dark:border-gray-600">
				<div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-5">
					<SearchOutline class="h-4 w-4 text-gray-500 dark:text-gray-400" />
				</div>
				<Input
					type="text"
					placeholder="Search tokens or enter address (0x...)"
					bind:value={searchQuery}
					on:input={handleSearch}
					class="token-search-input pl-10"
				/>
			</div>

			<div class="token-list max-h-80 overflow-y-auto">
				{#if isSearching}
					<div class="p-4 text-center text-gray-500 dark:text-gray-400">
						<p>Searching tokens...</p>
					</div>
				{:else}
					<!-- Show custom token candidate first if exists -->
					{#if customTokenCandidate}
						<div class="custom-token-section border-b border-gray-200 dark:border-gray-600">
							<div
								class="token-item flex cursor-pointer items-center border-b border-gray-100 p-3 last:border-b-0 hover:bg-gray-50 dark:border-gray-600 dark:hover:bg-gray-700"
								class:bg-blue-50={selectedToken?.address === customTokenCandidate.address}
								class:dark:bg-blue-900={selectedToken?.address === customTokenCandidate.address}
								class:border-l-4={selectedToken?.address === customTokenCandidate.address}
								class:border-l-blue-500={selectedToken?.address === customTokenCandidate.address}
								on:click={() => handleTokenSelect(customTokenCandidate)}
								on:keydown={(e) => e.key === 'Enter' && handleTokenSelect(customTokenCandidate)}
								role="button"
								tabindex="0"
							>
								<div class="token-info flex-grow">
									<div class="token-name font-medium text-gray-900 dark:text-white">
										Custom Token
									</div>
									<div class="token-details flex gap-2 text-sm text-gray-500 dark:text-gray-400">
										<span class="address">{formatAddress(customTokenCandidate.address)}</span>
									</div>
									<div class="mt-1 flex items-center gap-1 text-xs text-amber-600 dark:text-amber-400">
										<ExclamationTriangleOutline class="h-3 w-3" />
										<span>Custom token - will be validated when selected</span>
									</div>
								</div>
								{#if selectedToken?.address === customTokenCandidate.address}
									<CheckCircleSolid class="selected-icon h-5 w-5 text-green-500" />
								{/if}
							</div>
						</div>
					{/if}

					<!-- Show regular tokens -->
					{#each tokens as token (token.address)}
						<div
							class="token-item flex cursor-pointer items-center border-b border-gray-100 p-3 last:border-b-0 hover:bg-gray-50 dark:border-gray-600 dark:hover:bg-gray-700"
							class:bg-blue-50={selectedToken?.address === token.address}
							class:dark:bg-blue-900={selectedToken?.address === token.address}
							class:border-l-4={selectedToken?.address === token.address}
							class:border-l-blue-500={selectedToken?.address === token.address}
							on:click={() => handleTokenSelect(token)}
							on:keydown={(e) => e.key === 'Enter' && handleTokenSelect(token)}
							role="button"
							tabindex="0"
						>
							<div class="token-info flex-grow">
								<div class="token-name font-medium text-gray-900 dark:text-white">
									{token.name}
								</div>
								<div class="token-details flex gap-2 text-sm text-gray-500 dark:text-gray-400">
									<span class="symbol font-medium">{token.symbol}</span>
									<span class="address">{formatAddress(token.address)}</span>
								</div>
							</div>
							{#if selectedToken?.address === token.address}
								<CheckCircleSolid class="selected-icon h-5 w-5 text-green-500" />
							{/if}
						</div>
					{/each}

					{#if tokens.length === 0 && !customTokenCandidate && searchQuery && !isAddress(searchQuery)}
						<div class="no-results p-4 text-center text-gray-500 dark:text-gray-400">
							<p>No tokens found matching your search.</p>
							<button
								class="mt-2 text-blue-600 underline hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300"
								on:click={() => {
									searchQuery = '';
									loadTokens();
								}}
							>
								Clear search
							</button>
						</div>
					{/if}
				{/if}
			</div>
		</Modal>
	</div>
</div>
