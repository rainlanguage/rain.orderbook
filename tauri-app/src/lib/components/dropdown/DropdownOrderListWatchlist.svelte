<script lang="ts">
  import { watchlist, activeWatchlistItems } from '$lib/stores/settings';
  import DropdownCheckbox from './DropdownCheckbox.svelte';

  $: options = $watchlist;
  $: value = $activeWatchlistItems;

  function handleChange(event: CustomEvent<string[]>) {
    $activeWatchlistItems = event.detail;
  }
</script>

<DropdownCheckbox
  bind:options
  bind:value
  on:change={handleChange}
  label="Watchlist"
  allLabel="All addresses"
  emptyMessage="No watchlist added"
>
  <svelte:fragment slot="item" let:item>
    {item}
  </svelte:fragment>
</DropdownCheckbox>
