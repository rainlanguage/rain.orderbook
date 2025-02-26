<script lang="ts">
	import type { StrategyDetail } from '$lib/types/strategy';
	import ValidStrategiesSection from './ValidStrategiesSection.svelte';
	import InvalidStrategiesSection from './InvalidStrategiesSection.svelte';
	export let strategyDetails: StrategyDetail[];

	const validStrategies = strategyDetails.filter((strategy) => !strategy.error);
	const invalidStrategies = strategyDetails.filter((strategy) => strategy.error);
</script>

<ValidStrategiesSection {validStrategies} />
<InvalidStrategiesSection {invalidStrategies} />
