<script lang="ts">
	import { DeploymentSteps } from '@rainlanguage/ui-components';
	import StrategyTile from './StrategyTile.svelte';
	import { page } from '$app/stores';
	const { strategies } = $page.data;
</script>

<div class="flex flex-col justify-center pb-12">
	<div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
		{#each strategies as strategy}
			<StrategyTile strategyUrl={strategy} />
		{/each}
	</div>
	<DeploymentSteps />
</div>
