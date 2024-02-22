<script lang="ts">
  import { Helper, Label } from 'flowbite-svelte';
  import { settings, activeChainSettingsIndex } from '$lib/stores/settings';
  import DropdownRadio from '$lib/components/DropdownRadio.svelte';
  import SkeletonRow from '$lib/components/SkeletonRow.svelte';
</script>

<Label>Chain</Label>
{#await settings.load()}
  <SkeletonRow />
{:then}
  <DropdownRadio options={$settings.chains || []} bind:value={$activeChainSettingsIndex}>
    <svelte:fragment slot="content" let:selected>
      {selected.label ? selected.label : selected.rpc_url}
    </svelte:fragment>

    <svelte:fragment slot="option" let:option>
      {#if option.label}
        <div class="w-full overflow-hidden overflow-ellipsis">
          <div class="text-md mb-2 break-word">{option.label}</div>
          <Helper class="text-xs overflow-hidden overflow-ellipsis break-all">{option.rpc_url}</Helper>
        </div>
      {:else}
        <div class="w-full text-xs overflow-hidden overflow-ellipsis break-all">
          {option.rpc_url}
        </div>
      {/if}
    </svelte:fragment>
  </DropdownRadio>
{/await}
