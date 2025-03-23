<script lang="ts">
	import { DotrainOrderGui } from '@rainlanguage/orderbook';
	import DeploymentTile from './DeploymentTile.svelte';
	export let dotrain: string;
	export let strategyName: string;
</script>

{#await DotrainOrderGui.getDeploymentDetails(dotrain) then deploymentsWithDetails}
	<div
		class="mr-auto grid w-full grid-cols-1 justify-items-start gap-4 md:grid-cols-2 lg:w-auto lg:grid-cols-3"
	>
		{#each deploymentsWithDetails as [key, { name, description }]}
			<DeploymentTile {name} {description} {key} {strategyName} />
		{/each}
	</div>
{:catch error}
	<p class="text-red-500">Error loading deployments:</p>
	<p class="text-gray-500">
		{error instanceof Error ? error.message : 'Unknown error'}
	</p>
{/await}
