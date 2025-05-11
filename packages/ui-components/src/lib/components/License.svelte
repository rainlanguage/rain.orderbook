<script>
	import { Heading, Text, BlockQuote } from '@rainlanguage/ui-components';
	import Markdown from 'svelte-markdown';
	import { onMount } from 'svelte';
	import { LICENSE_URL } from '../consts';

	let source = '';

	onMount(async () => {
		try {
			const response = await fetch(LICENSE_URL);
			if (response.ok) {
				source = await response.text();
			} else {
				source = `Failed to fetch license: HTTP ${response.status}`;
			}
		} catch {
			source = 'Failed to fetch license.';
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
