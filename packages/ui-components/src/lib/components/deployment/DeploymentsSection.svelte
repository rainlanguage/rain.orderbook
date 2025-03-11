<script lang="ts">
	import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
	import DeploymentTile from './DeploymentTile.svelte';

	export let dotrain: string;
	export let strategyName: string;
	let deployments: { key: string; name: string; description: string }[] = [];
	let error: string = '';
	let errorDetails: string = '';

	const getDeployments = async () => {
		try {
			const result = await DotrainOrderGui.getDeploymentDetails(dotrain);
			if (result.error) {
				throw new Error(result.error.msg);
			}
			const deploymentsWithDetails = result.value;
			deployments = Array.from(deploymentsWithDetails, ([key, details]) => ({
				key,
				...details
			}));
		} catch (e: unknown) {
			error = 'Error getting deployments.';
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
		}
	};

	$: if (dotrain) {
		getDeployments();
	}
</script>

{#if deployments.length > 0}
	<div
		class="mr-auto grid w-full grid-cols-1 justify-items-start gap-4 md:grid-cols-2 lg:w-auto lg:grid-cols-3"
	>
		{#each deployments as { key, name, description }}
			<DeploymentTile {name} {description} {key} {strategyName} />
		{/each}
	</div>
{:else if error}
	<p class="text-red-500">Error loading deployments: {error}</p>
{:else}
	<p class="text-gray-500">{errorDetails}</p>
{/if}
