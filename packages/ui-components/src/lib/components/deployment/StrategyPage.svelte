<script lang="ts">
	import { fade } from 'svelte/transition';
	import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
	import DeploymentsSection from './DeploymentsSection.svelte';
	import SvelteMarkdown from 'svelte-markdown';

	export let strategyName: string = '';
	export let dotrain: string = '';
	let markdownContent: string = '';

	const isMarkdownUrl = (url: string): boolean => {
		return url.trim().toLowerCase().endsWith('.md');
	};

	const fetchMarkdownContent = async (url: string) => {
		try {
			const response = await fetch(url);
			if (!response.ok) throw new Error(`Failed to fetch markdown: ${response.statusText}`);
			return await response.text();
		} catch (error) {
			throw new Error(
				`Failed to fetch markdown: ${error instanceof Error ? error.message : 'Unknown error'}`
			);
		}
	};

	const getStrategyWithMarkdown = async () => {
		const strategyDetails = await DotrainOrderGui.getStrategyDetails(dotrain);
		if (strategyDetails.description && isMarkdownUrl(strategyDetails.description)) {
			try {
				markdownContent = await fetchMarkdownContent(strategyDetails.description);
			} catch (error: unknown) {
				error = error instanceof Error ? error.message : 'Unknown error';
			}
		}
		return strategyDetails;
	};
</script>

{#await getStrategyWithMarkdown() then strategyDetails}
	<div>
		<div in:fade class="flex flex-col gap-8">
			<div class="flex max-w-2xl flex-col gap-3 text-start lg:gap-6">
				<h1 class="text-4xl font-semibold text-gray-900 lg:text-6xl dark:text-white">
					{strategyDetails.name}
				</h1>
				{#if isMarkdownUrl(strategyDetails.description) && markdownContent}
					<div data-testId="markdown-content" class="prose dark:prose-invert">
						<SvelteMarkdown source={markdownContent} />
					</div>
				{:else}
					<p
						data-testId="plain-description"
						class="text-base text-gray-600 lg:text-lg dark:text-gray-400"
					>
						<span class="text-red-500">Could not load strategy description from: </span>

						{strategyDetails.description}
					</p>
				{/if}
			</div>
			<div class="u flex flex-col gap-4">
				<h2 class="text-3xl font-semibold text-gray-900 dark:text-white">Deployments</h2>
				<DeploymentsSection {dotrain} {strategyName} />
			</div>
		</div>
	</div>
{:catch error}
	<div class="p-4 text-red-500">
		<p class="text-xl font-semibold">Error getting strategy details</p>
		<p class="text-gray-600 dark:text-gray-400">
			{error instanceof Error ? error.message : 'Unknown error'}
		</p>
	</div>
{/await}
