<script lang="ts">
	import type { StrategyDetail } from '$lib/types/strategy';
	export let strategiesWithErrors: StrategyDetail[];
</script>

<div
	class="flex flex-col gap-4 rounded-xl bg-red-100 p-6 dark:bg-red-900"
	data-testid="invalid-strategies"
>
	<h2 class="text-xl font-semibold text-gray-900 dark:text-white">
		Invalid Strategies in registry
	</h2>
	<div class="flex flex-col gap-2">
		{#each strategiesWithErrors as strategy}
			<div class="flex flex-col gap-1">
				<span class="font-medium">{strategy.name}</span>
				<span class="text-red-600 dark:text-red-400">{strategy.error}</span>
			</div>
		{/each}
	</div>
</div>
