<script lang="ts">
  import { Button, Dropdown, Label, Radio } from 'flowbite-svelte';
  import { ChevronDownSolid } from 'flowbite-svelte-icons';
  import { settings, watchlist } from '$lib/stores/settings';
  import { isEmpty } from 'lodash';

  let selectedAddresses: Set<string> = new Set();
  let open = false;

  function toggleAllAddresses() {
    if (selectedAddresses.size === $watchlist.length) {
      selectedAddresses = new Set();
    } else {
      selectedAddresses = new Set($watchlist);
    }
  }

  function toggleAddress(address: string) {
    if (selectedAddresses.has(address)) {
      selectedAddresses.delete(address);
    } else {
      selectedAddresses.add(address);
    }
    selectedAddresses = selectedAddresses;
  }

  $: selectedAddressesCount = selectedAddresses.size;
</script>

<Label>Watchlist</Label>
<div>
  <Button
    color="alternative"
    class="flex w-full justify-between overflow-hidden overflow-ellipsis pl-2 pr-0 text-left"
  >
    <div class="flex-grow overflow-hidden text-ellipsis whitespace-nowrap">
      {selectedAddressesCount === 0
        ? 'Select addresses'
        : selectedAddressesCount === $watchlist.length
          ? 'All addresses'
          : `${selectedAddressesCount} address${selectedAddressesCount > 1 ? 'es' : ''}`}
    </div>
    <ChevronDownSolid class="mx-2 h-3 w-3 text-black dark:text-white" />
  </Button>

  <Dropdown class="w-full min-w-72 py-0" bind:open>
    <Radio
      class="w-full rounded-lg p-3 hover:bg-gray-100 dark:hover:bg-gray-600"
      on:click={toggleAllAddresses}
      checked={selectedAddressesCount === $watchlist.length}
    >
      <div class="ml-2">All addresses</div>
    </Radio>
    {#each $watchlist as address}
      <Radio
        class="w-full rounded-lg p-3 hover:bg-gray-100 dark:hover:bg-gray-600"
        on:click={() => toggleAddress(address)}
        checked={selectedAddresses.has(address)}
      >
        <div class="ml-2">{address}</div>
      </Radio>
    {/each}
  </Dropdown>
</div>

{#if !$settings?.watchlist || isEmpty($settings?.watchlist)}
  <span>No watchlist added to <a href="/settings">settings</a></span>
{/if}
