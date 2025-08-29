<script lang="ts">
	import { goto } from '$app/navigation';
	import type { RaindexVault } from '@rainlanguage/orderbook';

	export let vault: RaindexVault;

	const handleClick = (event: MouseEvent) => {
		// Stop event propagation to prevent parent table row click
		event.stopPropagation();
		// Navigate to vault details page
		goto(`/vaults/${vault.chainId}-${vault.orderbook}-${vault.id}`);
	};
</script>

<button
	type="button"
	class="flex w-full flex-col rounded-xl border border-gray-200 bg-gray-50 px-4 py-3 shadow-sm transition-colors hover:bg-gray-100 dark:border-gray-600 dark:bg-gray-700 dark:hover:bg-gray-600"
	on:click={handleClick}
	data-testid="vault-card"
>
	<span class="font-semibold text-gray-800 dark:text-gray-200">{vault.token.symbol}</span>
	<span class="truncate text-xs text-gray-500 dark:text-gray-400">{vault.formattedBalance}</span>
</button>
