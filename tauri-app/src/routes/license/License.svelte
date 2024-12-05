<script>
  import { Heading } from '@rainlanguage/ui-components';
  import Markdown from 'svelte-markdown';

  import { onMount } from 'svelte';
  import { Text } from '@rainlanguage/ui-components';
  import { BlockQuote } from '@rainlanguage/ui-components';

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
