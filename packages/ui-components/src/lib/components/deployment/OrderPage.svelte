<script lang="ts">
	import { fade } from 'svelte/transition';
	import { DotrainOrderGui } from '@rainlanguage/orderbook';
	import DeploymentsSection from './DeploymentsSection.svelte';
	import SvelteMarkdown from 'svelte-markdown';

	export let orderName: string = '';
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

	const getOrderWithMarkdown = async () => {
		try {
			const result = await DotrainOrderGui.getOrderDetails(dotrain);
			if (result.error) {
				throw new Error(result.error.msg);
			}
			const orderDetails = result.value;

			if (orderDetails.description && isMarkdownUrl(orderDetails.description)) {
				await fetchMarkdownContent(orderDetails.description);
			}
			return orderDetails;
		} catch {
			throw new Error('Failed to get order details');
		}
	};
</script>

{#await getOrderWithMarkdown() then orderDetails}
	<div>
		<div in:fade class="flex flex-col gap-8">
			<div class="flex max-w-2xl flex-col gap-3 text-start lg:gap-6">
				<h1 class="text-4xl font-semibold text-gray-900 lg:text-6xl dark:text-white">
					{orderDetails.name}
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
							{orderDetails.description}
						</p>
					</div>
				{/if}
			</div>
			<div class="u flex flex-col gap-4">
				<h2 class="text-3xl font-semibold text-gray-900 dark:text-white">Deployments</h2>
				<DeploymentsSection {dotrain} {orderName} />
			</div>
		</div>
	</div>
{:catch error}
	<div>
		<p class="text-red-500">{error}</p>
	</div>
{/await}
