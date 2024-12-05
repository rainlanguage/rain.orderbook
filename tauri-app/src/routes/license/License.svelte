<script>
    import Heading from './Heading.svelte';
    import Markdown from 'svelte-markdown';

    import { onMount } from 'svelte';
    import Text from './Text.svelte';
    import BlockQuote from './BlockQuote.svelte';

    let source = '';

    onMount(async () => {
        try {
            const response = await fetch(
                'https://raw.githubusercontent.com/rainlanguage/decentralicense/refs/heads/master/README.md',
            );
            if (response.ok) {
                source = await response.text();
            }
        } catch {
            source = '';
        }
    });
</script>

<Markdown
        {source}
        renderers={{
    text: Text,
    heading: Heading,
    blockquote: BlockQuote,
  }}
/>
