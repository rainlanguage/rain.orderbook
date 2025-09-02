<script lang="ts">
	import { Input, Button, Modal } from 'flowbite-svelte';
	import {
		SearchOutline,
		CheckCircleSolid,
		ListSolid,
		ExclamationCircleSolid
	} from 'flowbite-svelte-icons';
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
	let customToken: TokenInfo | null = null;
	let customTokenError = '';
	let isValidatingCustomToken = false;

	const gui = useGui();

	async function loadTokens(search?: string) {
		isSearching = true;

		// Clear any previous custom token state
		customToken = null;
		customTokenError = '';

		const result = await gui.getAllTokens(search);
		if (result.error) {
			tokens = [];
		} else {
			tokens = result.value;
		}

		isSearching = false;

		// If the search query looks like an address, check if it's already in the results
		// If not, show it as a custom token option
		if (search && isAddress(search)) {
			const addressExists = tokens.some(
				(token) => token.address.toLowerCase() === search.toLowerCase()
			);
			if (!addressExists) {
				await validateCustomToken(search);
			}
		}
	}

	async function validateCustomToken(address: string) {
		isValidatingCustomToken = true;
		customTokenError = '';

		try {
			// Create a minimal token object with just the address
			// The parent component will handle validation and fetching real token info
			customToken = {
				address: address,
				name: address, // Display the address as the name until real info is fetched
				symbol: '', // Will be populated by parent after validation
				decimals: 18 // Default value, will be updated by parent
			};
		} catch (error) {
			customTokenError = (error as Error).message || 'Unable to validate token address';
		} finally {
			isValidatingCustomToken = false;
		}
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
		: 'Select a token...';

	$: if (modalOpen) {
		tick().then(() => {
			const input = document.querySelector('.token-search-input') as HTMLInputElement;
			if (input) {
				input.focus();
			}
		});
	}
</script>

<div class="token-dropdown">
	<div class="relative w-full">
		<Button
			color="alternative"
			class="flex w-full justify-between overflow-hidden overflow-ellipsis pl-4 pr-2 text-left hover:bg-gray-50 dark:hover:bg-gray-700"
			size="lg"
			on:click={() => (modalOpen = true)}
		>
			<div class="flex-grow overflow-hidden">
				<span class="text-gray-900 dark:text-white">{displayText}</span>
			</div>
			<div class="flex items-center gap-1 text-xs text-gray-500 dark:text-gray-400">
				<span>Browse</span>
				<ListSolid class="h-4 w-4" />
			</div>
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
					placeholder="Search tokens or enter token address (0x...)"
					bind:value={searchQuery}
					on:input={handleSearch}
					class="token-search-input pl-10"
				/>
			</div>

			<div class="token-list max-h-80 overflow-y-auto">
				{#if isSearching || isValidatingCustomToken}
					<div class="p-4 text-center text-gray-500 dark:text-gray-400">
						<p>{isValidatingCustomToken ? 'Validating token address...' : 'Searching tokens...'}</p>
					</div>
				{:else}
					<!-- Show custom token if found -->
					{#if customToken}
						<div class="custom-token-section border-b border-gray-200 dark:border-gray-600">
							<div class="bg-yellow-50 p-3 dark:bg-yellow-900/20">
								<div
									class="mb-2 flex items-center gap-2 text-sm text-yellow-800 dark:text-yellow-200"
								>
									<ExclamationCircleSolid class="h-4 w-4" />
									<span class="font-medium">Custom Token (Not in verified list)</span>
								</div>
								<p class="text-xs text-yellow-700 dark:text-yellow-300">
									Clicking will validate this address and fetch token information.
								</p>
							</div>
							<div
								class="token-item flex cursor-pointer items-center border-b border-gray-100 p-3 hover:bg-gray-50 dark:border-gray-600 dark:hover:bg-gray-700"
								class:bg-blue-50={selectedToken?.address === customToken.address}
								class:dark:bg-blue-900={selectedToken?.address === customToken.address}
								class:border-l-4={selectedToken?.address === customToken.address}
								class:border-l-blue-500={selectedToken?.address === customToken.address}
								on:click={() => customToken && handleTokenSelect(customToken)}
								on:keydown={(e) =>
									e.key === 'Enter' && customToken && handleTokenSelect(customToken)}
								role="button"
								tabindex="0"
							>
								<div class="token-info flex-grow">
									<div class="token-name font-medium text-gray-900 dark:text-white">
										{formatAddress(customToken.address)}
									</div>
									<div class="token-details flex gap-2 text-sm text-gray-500 dark:text-gray-400">
										<span class="symbol font-medium">Custom Token</span>
									</div>
								</div>
								{#if selectedToken?.address === customToken.address}
									<CheckCircleSolid class="selected-icon h-5 w-5 text-green-500" />
								{/if}
							</div>
						</div>
					{/if}

					<!-- Show custom token error if any -->
					{#if customTokenError}
						<div
							class="border-b border-gray-200 bg-red-50 p-4 dark:border-gray-600 dark:bg-red-900/20"
						>
							<div class="flex items-center gap-2 text-sm text-red-800 dark:text-red-200">
								<ExclamationCircleSolid class="h-4 w-4" />
								<span>{customTokenError}</span>
							</div>
						</div>
					{/if}

					<!-- Show tokens from list -->
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

					{#if tokens.length === 0 && !customToken && !customTokenError && searchQuery}
						<div class="no-results p-4 text-center text-gray-500 dark:text-gray-400">
							<p>No tokens found matching your search.</p>
							{#if !isAddress(searchQuery)}
								<p class="mt-1 text-xs">Try entering a token address (0x...)</p>
							{/if}
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
