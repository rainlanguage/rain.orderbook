<script lang="ts">
	import { fade } from 'svelte/transition';
	import { DotrainOrderGui, type NameAndDescriptionCfg } from '@rainlanguage/orderbook/js_api';
	import DeploymentsSection from './DeploymentsSection.svelte';

	export let strategyUrl: string = '';
	export let strategyName: string = '';
	export let rawDotrain: string = '';
	let strategyDetails: NameAndDescriptionCfg;
	let dotrain: string;
	let error: string;
	let errorDetails: string;

	const getStrategy = async () => {
		try {
			if (rawDotrain) {
				dotrain = rawDotrain;
			} else {
				const response = await fetch(strategyUrl);
				dotrain = await response.text();
			}
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
	<div in:fade class="flex flex-col gap-8">
		<div class="flex max-w-prose flex-col gap-6 text-start">
			<h1 class="text-3xl font-semibold text-gray-900 lg:text-5xl dark:text-white">
				{strategyDetails.name}
			</h1>
			<p class="text-base text-gray-600 lg:text-lg dark:text-gray-400">
				{strategyDetails.description}
			</p>
		</div>
		<DeploymentsSection {dotrain} {strategyName} />
	</div>
{:else if error}
	<div>
		<p>{error}</p>
		<p>{errorDetails}</p>
	</div>
{/if}
