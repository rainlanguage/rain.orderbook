<script lang="ts">
    import { fade } from 'svelte/transition';
    import DeploymentsSection from './DeploymentsSection.svelte';
    import SvelteMarkdown from 'svelte-markdown';
    import { useRegistry } from '$lib/providers/registry/useRegistry';
    import { type NameAndDescriptionCfg, type DotrainRegistry } from '@rainlanguage/orderbook';

    export let orderName: string = '';
    let markdownContent: string = '';
    let error: string | undefined;

    const { registry, loading, error: providerError } = useRegistry();

    const isMarkdownUrl = (url: string): boolean => url.trim().toLowerCase().endsWith('.md');

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

    const getOrderFromRegistry = async (reg: DotrainRegistry): Promise<NameAndDescriptionCfg> => {
        try {
            const result = reg.getAllOrderDetails();
            if (result.error || !result.value) throw new Error(result.error?.msg ?? 'No details');
            const details = result.value.get(orderName);
            if (!details) throw new Error('Order not found');
            if (details.description && isMarkdownUrl(details.description)) {
                await fetchMarkdownContent(details.description);
            }
            return details;
        } catch (e) {
            throw new Error('Failed to get order details');
        }
    };
</script>

{#if $loading}
    <div>Loading order…</div>
{:else if $providerError}
    <div>Failed to initialize registry: {$providerError}</div>
{:else if $registry}
{#await getOrderFromRegistry($registry) then orderDetails}
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
				<DeploymentsSection {orderName} />
            </div>
        </div>
    </div>
{:catch error}
    <div>
        <p class="text-red-500">{error}</p>
    </div>
{/await}
{/if}
