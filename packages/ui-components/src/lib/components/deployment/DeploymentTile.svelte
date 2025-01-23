<script lang="ts">
	import { DotrainOrderGui, type NameAndDescription } from '@rainlanguage/orderbook/js_api';
	import { goto } from '$app/navigation';
	export let strategyName: string;
	export let deployment: string;
	export let dotrain: string;

	let deploymentDetails: NameAndDescription;
	let error: string;
	let errorDetails: string;

	const getDetails = async () => {
		try {
			deploymentDetails = await DotrainOrderGui.getDeploymentDetails(dotrain, deployment);
			console.log(deploymentDetails);
		} catch (e: unknown) {
			error = 'Error getting deployment details';
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
		}
	};

	$: getDetails();
</script>

{#if deploymentDetails}
	<button
		on:click={() => goto(`/deploy/${strategyName}/${deployment}`)}
		class="block max-w-sm cursor-pointer rounded-lg border border-gray-200 bg-white p-4 text-left shadow hover:bg-gray-100 dark:border-gray-700 dark:bg-gray-800 dark:hover:bg-gray-700"
	>
		<h1 class="text-2xl font-semibold text-gray-900 dark:text-white">{deploymentDetails.name}</h1>
		<p class="text-gray-600 dark:text-gray-400">{deploymentDetails.description}</p>
	</button>
{:else if error}
	<p>{error}</p>
	<p>{errorDetails}</p>
{/if}
