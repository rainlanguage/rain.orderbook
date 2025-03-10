<script lang="ts">
	import { fade } from 'svelte/transition';
	import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
	import DeploymentsSection from './DeploymentsSection.svelte';
	import SvelteMarkdown from 'svelte-markdown';

	export let strategyName: string = '';
	export let dotrain: string = '';
	let markdownContent: string = '';
	let error: string | undefined;

	const isMarkdownUrl = (url: string): boolean => {
		return url.trim().toLowerCase().endsWith('.md');
	};

	const fetchMarkdownContent = async (url: string) => {
		try {
			const response = await fetch(url);
			if (response.ok) {
				markdownContent = await response.text();
			}
		} catch {
			error = `Failed to fetch markdown`;
		}
	};

	const getStrategyWithMarkdown = async () => {
		try {
			const strategyDetails = await DotrainOrderGui.getStrategyDetails(dotrain);
			if (strategyDetails.description && isMarkdownUrl(strategyDetails.description)) {
				await fetchMarkdownContent(strategyDetails.description);
			}
			return strategyDetails;
		} catch (e) {
			throw new Error('Failed to get strategy details');
		}
	};
</script>

{#await getStrategyWithMarkdown() then strategyDetails}
	<div>
		<div in:fade class="flex flex-col gap-8">
			<div class="flex max-w-2xl flex-col gap-3 text-start lg:gap-6">
				<h1 class="text-4xl font-semibold text-gray-900 lg:text-6xl dark:text-white">
					{strategyDetails.name}
				</h1>
				{#if markdownContent}
					<div data-testid="markdown-content" class="prose dark:prose-invert">
						<SvelteMarkdown source={markdownContent} />
					</div>
				{:else}
					<div class="flex flex-col gap-2">
						{#if error}
							<p data-testid="markdown-error" class="text-red-500">{error}</p>
						{/if}
						<p
							data-testid="plain-description"
							class="text-base text-gray-600 lg:text-lg dark:text-gray-400"
						>
							{strategyDetails.description}
						</p>
					</div>
				{/if}
			</div>
			<div class="u flex flex-col gap-4">
				<h2 class="text-3xl font-semibold text-gray-900 dark:text-white">Deployments</h2>
				<DeploymentsSection {dotrain} {strategyName} />
			</div>
		</div>
	</div>
{:catch error}
	<div>
		<p class="text-red-500">{error}</p>
	</div>
{/await}
