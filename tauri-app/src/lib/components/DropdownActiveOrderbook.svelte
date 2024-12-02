<script lang="ts">
  import { Helper, Label } from 'flowbite-svelte';
  import { activeNetworkOrderbooks, activeOrderbookRef } from '$lib/stores/settings';
  import { DropdownRadio } from '@rainlanguage/ui-components';
  import SkeletonRow from '$lib/components/SkeletonRow.svelte';
  import { isEmpty } from 'lodash';
</script>

<Label>Orderbook</Label>
{#if isEmpty($activeNetworkOrderbooks)}
  <SkeletonRow />
{:else}
  <DropdownRadio options={$activeNetworkOrderbooks} bind:value={$activeOrderbookRef}>
    <svelte:fragment slot="content" let:selectedOption let:selectedRef>
      {#if selectedRef === undefined}
        <span>Select an orderbook</span>
      {:else if selectedOption?.label}
        <span data-testid="dropdown-activeorderbook">{selectedOption.label}</span>
      {:else}
        <span data-testid="dropdown-activeorderbook">{selectedRef}</span>
      {/if}
    </svelte:fragment>

    <svelte:fragment slot="option" let:option let:ref>
      <div
        data-testid="dropdown-activeorderbook-option"
        class="w-full overflow-hidden overflow-ellipsis"
      >
        <div class="text-md break-word mb-2">{option.label ? option.label : ref}</div>
        <Helper class="overflow-hidden overflow-ellipsis break-all text-xs">{option.address}</Helper
        >
      </div>
    </svelte:fragment>
  </DropdownRadio>
{/if}
