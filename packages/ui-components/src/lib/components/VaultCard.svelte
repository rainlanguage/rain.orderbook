<script lang="ts">
	import { goto } from '$app/navigation';
	import { Badge } from 'flowbite-svelte';
	import { WalletOutline } from 'flowbite-svelte-icons';
	import type { RaindexVault } from '@rainlanguage/orderbook';

	export let vault: RaindexVault;

	const handleClick = (event: MouseEvent) => {
		event.stopPropagation();
		event.preventDefault();
		if (vault.vaultless) return;
		goto(`/vaults/${vault.chainId}-${vault.orderbook}-${vault.id}`);
	};
</script>

{#if vault.vaultless}
	<div
		class="flex flex-col rounded-xl border border-blue-200 bg-blue-900/10 px-4 py-3"
		data-testid="vault-card-vaultless"
	>
		<div class="flex items-center gap-2">
			<span class="font-semibold text-gray-800 dark:text-gray-200">{vault.token.symbol}</span>
			<Badge color="blue" class="text-xs">
				<WalletOutline size="xs" class="mr-1" />Vaultless
			</Badge>
		</div>
	</div>
{:else}
	<button
		type="button"
		class="flex flex-col rounded-xl border border-gray-200 bg-gray-50 px-4 py-3 text-left shadow-sm transition-colors hover:bg-gray-100 dark:border-gray-600 dark:bg-gray-700 dark:hover:bg-gray-600"
		on:click={handleClick}
		data-testid="vault-card"
	>
		<span class="font-semibold text-gray-800 dark:text-gray-200">{vault.token.symbol}</span>
		<span class="truncate text-xs text-gray-500 dark:text-gray-400">{vault.formattedBalance}</span>
	</button>
{/if}
