<script lang="ts">
  import { Helper, Label } from 'flowbite-svelte';
  import { activeChainSettings, activeOrderbookSettingsIndex } from '$lib/stores/settings';
  import DropdownRadio from '$lib/components/DropdownRadio.svelte';
  import SkeletonRow from '$lib/components/SkeletonRow.svelte';
</script>

<Label>Orderbook</Label>
{#if $activeChainSettings === undefined || $activeChainSettings.orderbooks.length === 0}
  <SkeletonRow />
{:else}
  <DropdownRadio options={$activeChainSettings?.orderbooks || []} bind:value={$activeOrderbookSettingsIndex}>
    <svelte:fragment slot="content" let:selected>
      {selected.label ? selected.label : selected.address}
    </svelte:fragment>

    <svelte:fragment slot="option" let:option>
      {#if option.label}
        <div class="w-full overflow-hidden overflow-ellipsis">
          <div class="text-md mb-2 break-word">{option.label}</div>
          <Helper class="text-xs overflow-hidden overflow-ellipsis break-all">{option.address}</Helper>
        </div>
      {:else}
        <div class="w-full text-xs overflow-hidden overflow-ellipsis break-all">
          {option.address}
        </div>
      {/if}
    </svelte:fragment>
  </DropdownRadio>
{/if}