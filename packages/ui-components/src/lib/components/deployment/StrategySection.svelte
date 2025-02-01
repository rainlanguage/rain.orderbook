<script lang="ts">
	import { fade } from 'svelte/transition';
	import { DotrainOrderGui, type NameAndDescription } from '@rainlanguage/orderbook/js_api';
	import DeploymentsSection from './DeploymentsSection.svelte';

	export let strategyUrl: string;
	export let strategyName: string;
	let strategyDetails: NameAndDescription;
	let dotrain: string;
	let error: string;
	let errorDetails: string;

	const getStrategy = async () => {
		try {
			const response = await fetch(strategyUrl);
			const data = await response.text();
			dotrain = data;
			try {
				strategyDetails = await DotrainOrderGui.getStrategyDetails(dotrain);
			} catch (e: unknown) {
				error = 'Error getting strategy details';
				errorDetails = e instanceof Error ? e.message : 'Unknown error';
			}
		} catch (e: unknown) {
			error = 'Error fetching strategy';
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
		}
	};

	$: getStrategy();
</script>

{#if dotrain && strategyDetails}
	<div>
		<div in:fade class="flex flex-col gap-12">
			<div class="flex max-w-2xl flex-col gap-6 text-start">
				<h1 class="mb-6 text-4xl font-semibold text-gray-900 lg:text-8xl dark:text-white">
					{strategyDetails.name}
				</h1>
				<p class="text-xl text-gray-600 dark:text-gray-400">
					{strategyDetails.description}
				</p>
			</div>
			<DeploymentsSection {dotrain} {strategyName} />
		</div>
	</div>
{:else if error}
	<div>
		<p>{error}</p>
		<p>{errorDetails}</p>
	</div>
{/if}
