<script lang="ts">
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';
	import { fade } from 'svelte/transition';
	import {
		DotrainOrderGui,
		type DeploymentKeys,
		type NameAndDescription
	} from '@rainlanguage/orderbook/js_api';
	import DeploymentTile from './DeploymentTile.svelte';

	export let strategyUrl: string;
	export let strategyName: string;
	let strategyDetails: NameAndDescription;
	let availableDeployments: DeploymentKeys;
	let dotrain: string;
	let error: string;
	let errorDetails: string;

	const getStrategy = async () => {
		const response = await fetch(strategyUrl);
		const data = await response.text();
		dotrain = data;
		strategyDetails = await DotrainOrderGui.getStrategyDetails(dotrain);
		try {
			availableDeployments = await DotrainOrderGui.getDeploymentKeys(dotrain);
		} catch (e: unknown) {
			error = 'Error getting deployments';
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
		}
		return;
	};

	$: getStrategy();
</script>

{#if dotrain && strategyDetails}
	<div in:fade class="flex flex-col gap-12">
		<div class="flex max-w-2xl flex-col gap-6 text-start">
			<h1 class="mb-6 text-4xl font-semibold text-gray-900 lg:text-8xl dark:text-white">
				{strategyDetails.name}
			</h1>
			<p class="text-xl text-gray-600 dark:text-gray-400">
				{strategyDetails.description}
			</p>
		</div>
		{#if availableDeployments}
			<DeploymentSectionHeader title="Select Deployment" />

			<div class="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
				{#each availableDeployments as deployment}
					<DeploymentTile {deployment} {dotrain} {strategyName} />
				{/each}
			</div>
		{/if}
	</div>
{:else if error}
	<p>{error}</p>
	<p>{errorDetails}</p>
{/if}
