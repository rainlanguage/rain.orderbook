<script lang="ts">
  import { isEmpty } from 'lodash';
  import DropdownActiveNetwork from './DropdownActiveNetwork.svelte';
  import DropdownActiveOrderbook from './DropdownActiveOrderbook.svelte';
  import DropdownOrderListWatchlist from './dropdown/DropdownOrderListWatchlist.svelte';
  import DropdownOrderStatus from './dropdown/DropdownOrderStatus.svelte';
  import { settings } from '$lib/stores/settings';
  import { Alert } from 'flowbite-svelte';
</script>

<div class="flex min-w-[600px] items-center justify-end gap-x-2">
  {#if isEmpty($settings?.networks)}
    <Alert color="gray"
      >No networks added to <a class="underline" href="/settings">settings</a></Alert
    >
  {:else}
    <DropdownOrderStatus />
    <DropdownOrderListWatchlist />
    <DropdownActiveNetwork />
    <DropdownActiveOrderbook />
  {/if}
</div>
