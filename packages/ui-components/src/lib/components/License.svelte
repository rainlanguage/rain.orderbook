<script>
	import { Heading, Text, BlockQuote } from '@rainlanguage/ui-components';
	import Markdown from 'svelte-markdown';
	import { onMount } from 'svelte';

	let source = '';

	onMount(async () => {
		try {
			const response = await fetch(
				'https://raw.githubusercontent.com/rainlanguage/decentralicense/refs/heads/master/README.md'
			);
			if (response.ok) {
				source = await response.text();
			}
		} catch {
			source = 'Failed to fetch license';
		}
	});
</script>

<Markdown
	{source}
	renderers={{
		text: Text,
		heading: Heading,
		blockquote: BlockQuote
	}}
/>
