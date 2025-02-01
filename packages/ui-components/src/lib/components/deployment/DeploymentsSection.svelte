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
			const deploymentsWithDetails = await DotrainOrderGui.getDeploymentDetails(dotrain);
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
	<section class="flex flex-col gap-6">
		<div class="container grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
			{#each deployments as { key, name, description }}
				<DeploymentTile {name} {description} {key} {strategyName} />
			{/each}
		</div>
	</section>
{:else if error}
	<p class="text-red-500">Error loading deployments: {error}</p>
{:else}
	<p class="text-gray-500">{errorDetails}</p>
{/if}
