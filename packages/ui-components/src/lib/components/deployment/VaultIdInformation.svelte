<script lang="ts">
	import type { TokenBalance } from '$lib/types/tokenBalance';
	import TokenBalanceComponent from './TokenBalance.svelte';

	export let title: string;
	export let description: string;
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
		<TokenBalanceComponent {tokenBalance} />
	</div>
</div>
