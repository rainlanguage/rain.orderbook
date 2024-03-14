<script lang="ts">
  import { Helper, Label } from 'flowbite-svelte';
  import { activeNetworkRef, settings } from '$lib/stores/settings';
  import DropdownRadio from '$lib/components/DropdownRadio.svelte';
  import SkeletonRow from '$lib/components/SkeletonRow.svelte';
</script>

<Label>Chain</Label>
{#if $settings?.networks === undefined || Object.keys($settings?.networks).length === 0}
  <SkeletonRow />
{:else}
  <DropdownRadio options={$settings.networks} bind:value={$activeNetworkRef}>
    <svelte:fragment slot="content" let:selected>
      {selected ? selected : "Select a network"}
    </svelte:fragment>

    <svelte:fragment slot="option" let:option let:ref>
      <div class="w-full overflow-hidden overflow-ellipsis">
        <div class="text-md mb-2 break-word">{option.label ? option.label : ref}</div>
        <Helper class="text-xs overflow-hidden overflow-ellipsis break-all">{option.rpc}</Helper>
      </div>
    </svelte:fragment>
  </DropdownRadio>
{/if}
