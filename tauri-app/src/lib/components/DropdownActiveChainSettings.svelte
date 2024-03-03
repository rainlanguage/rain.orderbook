<script lang="ts">
  import { Helper, Label } from 'flowbite-svelte';
  import { activeNetworkIndex, networks } from '$lib/stores/settings';
  import DropdownRadio from '$lib/components/DropdownRadio.svelte';
  import SkeletonRow from '$lib/components/SkeletonRow.svelte';
</script>

<Label>Chain</Label>
{#if $networks === undefined || $networks.length === 0}
  <SkeletonRow />
{:else}
  <DropdownRadio options={$networks || []} bind:value={$activeNetworkIndex}>
    <svelte:fragment slot="content" let:selected>
      {selected[1].label ? selected[1].label : selected[0]}
    </svelte:fragment>

    <svelte:fragment slot="option" let:option>
      {#if option[1].label}
        <div class="w-full overflow-hidden overflow-ellipsis">
          <div class="text-md mb-2 break-word">{option[1].label}</div>
          <Helper class="text-xs overflow-hidden overflow-ellipsis break-all">{option[1].rpc}</Helper>
        </div>
      {:else}
        <div class="w-full text-xs overflow-hidden overflow-ellipsis break-all">
          {option[0]}
        </div>
      {/if}
    </svelte:fragment>
  </DropdownRadio>
{/if}
