<script lang="ts">
	import { fade } from 'svelte/transition';
	import { DotrainOrderGui, type NameAndDescriptionCfg } from '@rainlanguage/orderbook/js_api';
	import DeploymentsSection from './DeploymentsSection.svelte';
	import SvelteMarkdown from 'svelte-markdown';

	export let strategyName: string = '';
	export let dotrain: string = '';
	let strategyDetails: NameAndDescriptionCfg;
	let error: string;
	let errorDetails: string;
	let markdownContent: string = '';

	const isMarkdownUrl = (url: string): boolean => {
		return url.trim().toLowerCase().endsWith('.md');
	};

	const fetchMarkdownContent = async (url: string) => {
		try {
			const response = await fetch(url);
			if (!response.ok) throw new Error(`Failed to fetch markdown: ${response.statusText}`);
			return await response.text();
		} catch {
			return null;
		}
	};

	const getStrategy = async () => {
		try {
			let result = await DotrainOrderGui.getStrategyDetails(dotrain);
			if (result.error) {
				throw new Error(result.error.msg);
			}
			strategyDetails = result.value;
			if (strategyDetails.description && isMarkdownUrl(strategyDetails.description)) {
				const content = await fetchMarkdownContent(strategyDetails.description);
				if (content) {
					markdownContent = content;
				} else {
					error = 'Error fetching markdown';
					errorDetails = 'Failed to fetch markdown content';
				}
			}
		} catch (e: unknown) {
			error = 'Error getting strategy details';
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
		}
	};

	getStrategy();
</script>

{#if dotrain && strategyDetails}
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
{:else if error}
	<div>
		<p>{error}</p>
		<p>{errorDetails}</p>
	</div>
{/if}
