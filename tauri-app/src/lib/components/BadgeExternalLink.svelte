<script lang="ts">
  import { Badge } from 'flowbite-svelte';
  import { IconExternalLink } from '@rainlanguage/ui-components';
  export let text: string;
  export let href: string;
  export let color:
    | 'none'
    | 'red'
    | 'yellow'
    | 'green'
    | 'indigo'
    | 'purple'
    | 'pink'
    | 'blue'
    | 'dark'
    | 'primary'
    | undefined = 'blue';
</script>

<Badge target="_blank" {href} class="p-1 py-0 hover:opacity-75" {color}>
  <IconExternalLink svgClass="!h-2 !w-2 mr-1" />
  {text}
</Badge>
