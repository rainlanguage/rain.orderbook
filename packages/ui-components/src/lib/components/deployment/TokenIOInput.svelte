<script lang="ts">
	import { Input, Toggle } from 'flowbite-svelte';
	import { type OrderIOCfg, type TokenInfo, type VaultType } from '@rainlanguage/orderbook';
	import { onMount } from 'svelte';
	import { useGui } from '$lib/hooks/useGui';
	import type { TokenBalance } from '$lib/types/tokenBalance';
	import VaultIdInformation from './VaultIdInformation.svelte';

	const gui = useGui();

	export let label: 'Input' | 'Output';
	export let vault: OrderIOCfg;
	export let tokenBalances: Map<string, TokenBalance> = new Map();
	export let onApprovalAmountChange: ((tokenKey: string, amount: string) => void) | undefined =
		undefined;

	let tokenInfo: TokenInfo | null = null;
	let inputValue: string = '';
	let error: string = '';
	let isVaultless: boolean = vault.vaultless === true;
	let approvalAmount: string = '';

	onMount(() => {
		if (!vault.token?.key) return;

		if (!isVaultless) {
			const result = gui.getVaultIds();
			if (result.error) {
				error = result.error.msg;
				return;
			}
			const vaultIds = result.value;
			const vaultMap = vaultIds.get(label.toLowerCase());
			if (vaultMap) {
				const vaultId = vaultMap.get(vault.token.key);
				inputValue = vaultId || '';
			}
		}
	});

	const handleGetTokenInfo = async () => {
		if (!vault.token?.key) return;
		try {
			let result = await gui.getTokenInfo(vault.token?.key);
			if (result.error) {
				error = result.error.msg;
				return;
			}
			tokenInfo = result.value;
		} catch (e) {
			const errorMessage = (e as Error).message
				? (e as Error).message
				: 'Error getting token info.';
			error = errorMessage;
		}
	};

	const handleInput = async () => {
		if (!vault.token) {
			error = 'Vault token is not set.';
			return;
		}
		error = '';
		try {
			gui.setVaultId(label.toLowerCase() as VaultType, vault.token.key, inputValue);
		} catch (e) {
			const errorMessage = (e as Error).message ? (e as Error).message : 'Error setting vault ID.';
			error = errorMessage;
		}
	};

	const handleVaultlessToggle = () => {
		if (!vault.token?.key) return;
		error = '';
		const result = gui.setVaultless(label.toLowerCase() as VaultType, vault.token.key, isVaultless);
		if (result.error) {
			error = result.error.msg;
			return;
		}
		if (isVaultless) {
			inputValue = '';
		}
	};

	const handleApprovalAmountInput = () => {
		if (vault.token?.key && onApprovalAmountChange) {
			onApprovalAmountChange(vault.token.key, approvalAmount);
		}
	};

	$: if (vault.token?.key) {
		handleGetTokenInfo();
	}
</script>

<div class="flex w-full flex-col gap-6">
	<div class="flex w-full flex-col gap-2">
		<div class="flex items-center gap-2">
			<VaultIdInformation
				title={`${label} ${tokenInfo?.symbol ? `(${tokenInfo.symbol})` : ''}`}
				description={`${tokenInfo?.symbol || 'Token'} vault ID`}
				tokenBalance={tokenBalances.get(vault.token?.key || '')}
			/>
		</div>
	</div>

	<Toggle bind:checked={isVaultless} on:change={handleVaultlessToggle}
		>Vaultless mode (direct wallet transfer)</Toggle
	>

	{#if isVaultless}
		<div class="rounded-lg bg-blue-900/20 p-3">
			<p class="text-sm text-blue-300">Token transfers directly without vault storage.</p>
		</div>

		{#if label === 'Output'}
			<div class="flex flex-col gap-2">
				<label for="approval-amount" class="text-sm text-gray-400"
					>Approval Amount (defaults to unlimited)</label
				>
				<Input
					id="approval-amount"
					data-testid="approval-amount-input"
					size="lg"
					type="text"
					placeholder="Leave empty for unlimited"
					bind:value={approvalAmount}
					on:input={handleApprovalAmountInput}
				/>
			</div>
		{/if}
	{:else}
		<div class="flex flex-col gap-2">
			<Input
				data-testid="vault-id-input"
				size="lg"
				type="text"
				placeholder="Enter vault ID"
				bind:value={inputValue}
				on:input={handleInput}
			/>
		</div>
	{/if}

	{#if error}
		<p class="text-red-500">{error}</p>
	{/if}
</div>
