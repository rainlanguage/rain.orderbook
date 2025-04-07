<script lang="ts">
	import Markdown from 'svelte-markdown';

	export let title: string;
	export let description: string = '';
</script>

<div class="flex max-w-xl flex-grow flex-col gap-y-4 text-left">
	<h1 class="break-words text-xl font-semibold text-gray-900 lg:text-2xl dark:text-white">
		{title}
	</h1>
	{#if description}
		<div
			class="prose prose-sm lg:prose-base prose-neutral dark:prose-invert text-gray-600 dark:text-gray-400"
		>
			<Markdown source={description} />
		</div>
	{/if}
</div>
