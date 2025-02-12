<script lang="ts">
	import { fade } from 'svelte/transition';
	import { DeploymentsSection } from '@rainlanguage/ui-components';
	import { isMarkdownUrl } from '$lib/utils/markdown';
	import { SvelteMarkdown } from 'svelte-markdown';
	import { page } from '$app/stores';

	const { dotrain, key, name, description } = $page.data;
</script>

<div>
	<div in:fade class="flex flex-col gap-12">
		<div class="flex max-w-2xl flex-col gap-0 text-start lg:gap-6">
			<h1 class="text-4xl font-semibold text-gray-900 lg:text-6xl dark:text-white">
				{strategyDetails.name}
			</h1>
			<div>{strategyDetails.short_description}</div>
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
