<script lang="ts">
  import { props } from './MockComponent';

  // wheneveer the props update, we want to update the store with those
  $: $props = $$props;
</script>
