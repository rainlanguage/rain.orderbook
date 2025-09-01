<script lang="ts">
	import type { GuiSelectTokensCfg, TokenInfo } from '@rainlanguage/orderbook';
	import { CheckCircleSolid, CloseCircleSolid } from 'flowbite-svelte-icons';
	import { Spinner } from 'flowbite-svelte';
	import { onMount } from 'svelte';
	import { useGui } from '$lib/hooks/useGui';
	import TokenSelectionModal from './TokenSelectionModal.svelte';
	import TokenBalanceComponent from './TokenBalance.svelte';
	import type { TokenBalance } from '$lib/types/tokenBalance';

	export let token: GuiSelectTokensCfg;
	export let onSelectTokenSelect: (key: string) => void;
	export let tokenBalances: Map<string, TokenBalance> = new Map();

	let tokenInfo: TokenInfo | null = null;
	let error = '';
	let checking = false;
	let selectedToken: TokenInfo | null = null;

	const gui = useGui();

	onMount(async () => {
		try {
			let result = await gui.getTokenInfo(token.key);
			if (result.error) {
				throw new Error(result.error.msg);
			}
			tokenInfo = result.value;
			if (result.value.address) {
				selectedToken = result.value;
				onSelectTokenSelect(token.key);
			}
		} catch {
			// do nothing
		}
	});

	function handleTokenSelect(token: TokenInfo) {
		selectedToken = token;
		saveTokenSelection(token.address);
	}

	async function saveTokenSelection(address: string) {
		checking = true;
		error = '';
		try {
			await gui.setSelectToken(token.key, address);
			await getInfoForSelectedToken();
		} catch (e) {
			const errorMessage = (e as Error).message || 'Invalid token address.';
			error = errorMessage;
		} finally {
			checking = false;
			onSelectTokenSelect(token.key);
		}
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

	<TokenSelectionModal {selectedToken} onSelect={handleTokenSelect} />

	<div class="token-status">
		{#if checking}
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
				<TokenBalanceComponent tokenBalance={tokenBalances.get(token.key)} />
			</div>
		{:else if error}
			<div class="flex h-5 flex-row items-center gap-2" data-testid="error">
				<CloseCircleSolid class="h-5 w-5" color="red" />
				<span>{error}</span>
			</div>
		{/if}
	</div>
</div>
