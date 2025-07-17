<script lang="ts">
	import type { TokenBalance } from '$lib/types/tokenBalance';
	import { Spinner } from 'flowbite-svelte';
	import { formatUnits } from 'viem';

	export let title: string;
	export let description: string;
	export let decimals: number | undefined;
	export let tokenBalance: TokenBalance;
</script>

<div class="flex max-w-xl flex-grow flex-col gap-y-4 text-left">
	<h1 class="break-words text-xl font-semibold text-gray-900 lg:text-2xl dark:text-white">
		{title}
	</h1>
	<div class="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
		<div class="text-gray-600 dark:text-gray-400">
			{description}
		</div>
		{#if tokenBalance.loading}
			<div class="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
				<Spinner class="h-4 w-4" />
				<span>Loading balance...</span>
			</div>
		{:else if tokenBalance.balance !== null && !tokenBalance.error && decimals}
			<div class="text-sm text-gray-600 dark:text-gray-400">
				Balance: {formatUnits(tokenBalance.balance, decimals)}
			</div>
		{:else if tokenBalance.error}
			<div class="text-sm text-red-600 dark:text-red-400">
				{tokenBalance.error}
			</div>
		{/if}
	</div>
</div>
