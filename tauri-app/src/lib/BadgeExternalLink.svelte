<script lang="ts">
  import { Badge } from 'flowbite-svelte';
  import IconExternalLink from './IconExternalLink.svelte';
  export let text: string;
  export let href: string;
</script>

<Badge target="_blank" {href} class="p-1 py-0 hover:opacity-75">
  <IconExternalLink svgClass="!h-2 !w-2 mr-1" />
  {text}
</Badge>
