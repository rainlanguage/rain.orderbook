<script lang="ts">
	import { Input } from 'flowbite-svelte';
	import type { GuiSelectTokensCfg, TokenInfo } from '@rainlanguage/orderbook';
	import { CheckCircleSolid, CloseCircleSolid } from 'flowbite-svelte-icons';
	import { Spinner } from 'flowbite-svelte';
	import { onMount } from 'svelte';
	import { useGui } from '$lib/hooks/useGui';
	import ButtonSelectOption from './ButtonSelectOption.svelte';
	import TokenDropdown from './TokenDropdown.svelte';

	export let token: GuiSelectTokensCfg;
	export let onSelectTokenSelect: () => void;
	export let availableTokens: TokenInfo[] = [];
	export let loading: boolean = false;

	let inputValue: string | null = null;
	let tokenInfo: TokenInfo | null = null;
	let error = '';
	let checking = false;
	let selectionMode: 'dropdown' | 'custom' = 'dropdown';
	let searchQuery: string = '';
	let selectedToken: TokenInfo | null = null;

	const gui = useGui();

	onMount(async () => {
		let result = await gui.getTokenInfo(token.key);
		if (result.error) {
			throw new Error(result.error.msg);
		}
		tokenInfo = result.value;
		if (result.value.address) {
			inputValue = result.value.address;
		}
	});

	$: if (tokenInfo?.address && availableTokens.length > 0) {
		const foundToken = availableTokens.find(
			(t) => t.address.toLowerCase() === tokenInfo?.address.toLowerCase()
		);
		selectedToken = foundToken || null;
	}

	$: if (availableTokens.length > 0 && tokenInfo?.address && selectionMode === 'dropdown') {
		const foundToken = availableTokens.find(
			(t) => t.address.toLowerCase() === tokenInfo?.address.toLowerCase()
		);
		if (!foundToken) {
			selectionMode = 'custom';
		}
	}

	$: if (tokenInfo?.address && inputValue === null) {
		inputValue = tokenInfo.address;
	}

	function setMode(mode: 'dropdown' | 'custom') {
		selectionMode = mode;
		error = '';

		if (mode === 'dropdown') {
			searchQuery = '';
			if (inputValue && tokenInfo) {
				const foundToken = availableTokens.find(
					(t) => t.address.toLowerCase() === inputValue?.toLowerCase()
				);
				if (foundToken) {
					selectedToken = foundToken;
				} else {
					inputValue = null;
					tokenInfo = null;
					selectedToken = null;
					clearTokenSelection();
				}
			} else {
				inputValue = null;
				tokenInfo = null;
				selectedToken = null;
			}
		} else if (mode === 'custom') {
			selectedToken = null;
			tokenInfo = null;
			inputValue = '';
			error = '';
			clearTokenSelection();
		}
	}

	function handleTokenSelect(token: TokenInfo) {
		selectedToken = token;
		inputValue = token.address;
		saveTokenSelection(token.address);
	}

	function handleSearch(query: string) {
		searchQuery = query;
	}

	async function saveTokenSelection(address: string) {
		checking = true;
		error = '';
		try {
			await gui.saveSelectToken(token.key, address);
			await getInfoForSelectedToken();
		} catch (e) {
			const errorMessage = (e as Error).message || 'Invalid token address.';
			error = errorMessage;
		} finally {
			checking = false;
			onSelectTokenSelect();
		}
	}

	function clearTokenSelection() {
		gui.removeSelectToken(token.key);
		onSelectTokenSelect();
	}

	async function getInfoForSelectedToken() {
		error = '';
		try {
			let result = await gui.getTokenInfo(token.key);
			if (result.error) {
				throw new Error(result.error.msg);
			}
			tokenInfo = result.value;
			error = '';
		} catch {
			return (error = 'No token exists at this address.');
		}
	}

	async function handleInput(event: Event) {
		const currentTarget = event.currentTarget;
		if (currentTarget instanceof HTMLInputElement) {
			inputValue = currentTarget.value;

			if (tokenInfo && tokenInfo.address.toLowerCase() !== inputValue.toLowerCase()) {
				tokenInfo = null;
				selectedToken = null;
			}

			if (!inputValue) {
				error = '';
				tokenInfo = null;
				selectedToken = null;
				return;
			}

			saveTokenSelection(inputValue);
		}
	}
</script>

<div class="token-selection-container flex w-full flex-col gap-4">
	<div class="token-header">
		{#if token.name || token.description}
			<div class="flex flex-col">
				{#if token.name}
					<h1 class="break-words text-xl font-semibold text-gray-900 lg:text-xl dark:text-white">
						{token.name}
					</h1>
				{/if}
				{#if token.description}
					<p class="text-sm font-light text-gray-600 lg:text-base dark:text-gray-400">
						{token.description}
					</p>
				{/if}
			</div>
		{/if}
	</div>

	{#if availableTokens.length > 0 && !loading}
		<div class="selection-mode flex gap-2">
			<ButtonSelectOption
				active={selectionMode === 'dropdown'}
				buttonText="Select from list"
				clickHandler={() => setMode('dropdown')}
				dataTestId="dropdown-mode-button"
			/>
			<ButtonSelectOption
				active={selectionMode === 'custom'}
				buttonText="Custom address"
				clickHandler={() => setMode('custom')}
				dataTestId="custom-mode-button"
			/>
		</div>
	{/if}

	{#if selectionMode === 'dropdown' && availableTokens.length > 0}
		<TokenDropdown
			tokens={availableTokens}
			{selectedToken}
			onSelect={handleTokenSelect}
			searchValue={searchQuery}
			onSearch={handleSearch}
		/>
	{/if}

	{#if selectionMode === 'custom' || availableTokens.length === 0}
		<div class="custom-input">
			<Input
				type="text"
				size="lg"
				placeholder="Enter token address (0x...)"
				bind:value={inputValue}
				on:input={handleInput}
			/>
		</div>
	{/if}

	<div class="token-status">
		{#if loading}
			<div class="flex h-5 flex-row items-center gap-2">
				<Spinner class="h-5 w-5" />
				<span>Loading tokens...</span>
			</div>
		{:else if checking}
			<div class="flex h-5 flex-row items-center gap-2">
				<Spinner class="h-5 w-5" />
				<span>Checking...</span>
			</div>
		{:else if tokenInfo}
			<div
				class="flex h-5 flex-row items-center gap-2"
				data-testid={`select-token-success-${token.key}`}
			>
				<CheckCircleSolid class="h-5 w-5" color="green" />
				<span>{tokenInfo.name}</span>
			</div>
		{:else if error}
			<div class="flex h-5 flex-row items-center gap-2" data-testid="error">
				<CloseCircleSolid class="h-5 w-5" color="red" />
				<span>{error}</span>
			</div>
		{/if}
	</div>
</div>
