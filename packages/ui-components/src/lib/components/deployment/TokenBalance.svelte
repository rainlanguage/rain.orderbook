<script lang="ts">
	import type { TokenBalance } from '$lib/types/tokenBalance';
	import { Spinner } from 'flowbite-svelte';

	export let tokenBalance: TokenBalance | undefined;
</script>

{#if tokenBalance}
	{#if tokenBalance.loading}
		<div class="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
			<Spinner class="h-4 w-4" />
		</div>
	{:else if !tokenBalance.error}
		<div class="text-sm text-gray-600 dark:text-gray-400">
			Balance: {tokenBalance.value.formattedAmount}
		</div>
	{:else if tokenBalance.error}
		<div class="text-sm text-red-600 dark:text-red-400">
			{tokenBalance.error}
		</div>
	{/if}
{/if}
