<script lang="ts">
  import { Helper, Label } from 'flowbite-svelte';
  import { activeNetworkOrderbooks, activeOrderbookRef } from '$lib/stores/settings';
  import DropdownRadio from '$lib/components/DropdownRadio.svelte';
  import SkeletonRow from '$lib/components/SkeletonRow.svelte';
</script>

<Label>Orderbook</Label>
{#if $activeNetworkOrderbooks === undefined || Object.keys($activeNetworkOrderbooks).length === 0}
  <SkeletonRow />
{:else}
<DropdownRadio options={$activeNetworkOrderbooks} bind:value={$activeOrderbookRef}>
  <svelte:fragment slot="content" let:selected>
    {selected ? selected : "Select an orderbook"}
  </svelte:fragment>

    <svelte:fragment slot="option" let:option let:ref>
      <div class="w-full overflow-hidden overflow-ellipsis">
        <div class="text-md mb-2 break-word">{option.label ? option.label : ref}</div>
        <Helper class="text-xs overflow-hidden overflow-ellipsis break-all">{option.address}</Helper>
      </div>
    </svelte:fragment>
  </DropdownRadio>
{/if}