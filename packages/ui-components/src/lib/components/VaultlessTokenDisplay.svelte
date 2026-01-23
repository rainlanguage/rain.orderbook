<script lang="ts">
	import { createQuery } from '@tanstack/svelte-query';
	import { Badge, Spinner } from 'flowbite-svelte';
	import { WalletOutline, InfoCircleOutline } from 'flowbite-svelte-icons';
	import Tooltip from './Tooltip.svelte';
	import type { RaindexVault } from '@rainlanguage/orderbook';

	export let tokenVault: RaindexVault;

	$: balanceQuery = createQuery({
		queryKey: ['vaultless-balance', tokenVault.token.address, tokenVault.owner],
		queryFn: async () => {
			const [balRes, allowRes] = await Promise.all([
				tokenVault.getOwnerBalance(),
				tokenVault.getAllowance()
			]);
			if (balRes.error) throw new Error(balRes.error.readableMsg);
			if (allowRes.error) throw new Error(allowRes.error.readableMsg);
			return {
				balance: balRes.value.formattedAmount,
				allowance: allowRes.value.formattedAmount
			};
		},
		refetchInterval: 10000
	});
</script>

<div
	class="flex items-center justify-between space-y-2 rounded-lg border border-blue-200 bg-blue-900/10 p-2"
	data-testid="vaultless-token-display"
>
	<div class="flex flex-col items-start gap-y-2">
		<div class="flex items-center gap-2">
			<span id={`vaultless-info-${tokenVault.token.address}`}>
				{tokenVault.token.name} ({tokenVault.token.symbol})
			</span>
			<Badge color="blue" class="text-xs">
				<WalletOutline size="xs" class="mr-1" />Vaultless
			</Badge>
			<InfoCircleOutline
				class="h-4 w-4 text-blue-500"
				id={`vaultless-tooltip-${tokenVault.token.address}`}
			/>
			<Tooltip triggeredBy={`#vaultless-tooltip-${tokenVault.token.address}`}>
				Tokens transfer directly from wallet
			</Tooltip>
		</div>
		{#if $balanceQuery.isLoading}
			<div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400">
				<Spinner size="4" />
				<span>Loading wallet data...</span>
			</div>
		{:else if $balanceQuery.error}
			<span class="text-sm text-red-500 dark:text-red-400">
				Error: {$balanceQuery.error.message}
			</span>
		{:else if $balanceQuery.data}
			<div class="flex flex-col gap-1 text-sm text-gray-500 dark:text-gray-400">
				<span>Wallet Balance: {$balanceQuery.data.balance}</span>
				<span>Approved: {$balanceQuery.data.allowance}</span>
			</div>
		{/if}
	</div>
</div>
