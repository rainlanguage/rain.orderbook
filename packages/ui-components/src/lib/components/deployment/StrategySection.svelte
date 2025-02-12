<script lang="ts">
	import { fade } from 'svelte/transition';
	import { DotrainOrderGui, type NameAndDescription } from '@rainlanguage/orderbook/js_api';
	import DeploymentsSection from './DeploymentsSection.svelte';
	import SvelteMarkdown from 'svelte-markdown';

	export let strategyUrl: string = '';
	export let strategyName: string = '';
	export let rawDotrain: string = '';
	let strategyDetails: NameAndDescription;
	let dotrain: string;
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
			if (rawDotrain) {
				dotrain = rawDotrain;
			} else {
				const response = await fetch(strategyUrl);
				dotrain = await response.text();
			}
			try {
				strategyDetails = await DotrainOrderGui.getStrategyDetails(dotrain);
				console.log(strategyDetails);
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
			<div class="flex max-w-2xl flex-col gap-0 text-start lg:gap-6">
				<h1 class="text-4xl font-semibold text-gray-900 lg:text-6xl dark:text-white">
					{strategyDetails.name}
				</h1>
				{#if isMarkdownUrl(strategyDetails.description) && markdownContent}
					<div data-testId="markdown-content">
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
			<DeploymentsSection {dotrain} {strategyName} />
		</div>
	</div>
{:else if error}
	<div>
		<p>{error}</p>
		<p>{errorDetails}</p>
	</div>
{/if}
