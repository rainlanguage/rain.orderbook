<script lang="ts">
	import type { StrategyDetail } from '$lib/types/strategy';
	import StrategyShortTile from './StrategyShortTile.svelte';
	export let validStrategies: StrategyDetail[];
</script>

{#if validStrategies.length > 0}
	{#key validStrategies}
		<div
			class="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3"
			data-testid="valid-strategies"
		>
			{#each validStrategies as strategyDetail}
				<StrategyShortTile
					strategyDetails={strategyDetail.details}
					registryName={strategyDetail.name}
				/>
			{/each}
		</div>
	{/key}
{/if}
