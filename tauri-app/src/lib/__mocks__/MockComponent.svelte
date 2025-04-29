<script lang="ts">
  import { props } from './MockComponent';
  $: $props = $$props;
</script>

<div data-testid="mock-component" {...$$props}>
  <slot />
</div>
