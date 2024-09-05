<script lang="ts">
  import { watchlist, activeWatchlistItems } from '$lib/stores/settings';
  import DropdownCheckbox from './DropdownCheckbox.svelte';

  $: options = $watchlist;
</script>

<DropdownCheckbox
  {options}
  bind:value={$activeWatchlistItems}
  label="Watchlist"
  allLabel="All addresses"
  emptyMessage="No watchlist added"
/>
