<script lang="ts">
  import { Helper, Label } from 'flowbite-svelte';
  import { activeNetworkRef, settings } from '$lib/stores/settings';
  import { DropdownRadio } from '@rainlanguage/ui-components';
  import { isEmpty } from 'lodash';
</script>

<Label>Network</Label>
{#if !isEmpty($settings?.networks)}
  <DropdownRadio options={$settings.networks} bind:value={$activeNetworkRef}>
    <svelte:fragment slot="content" let:selectedOption let:selectedRef>
      {#if selectedRef === undefined}
        <span>Select a network</span>
      {:else if selectedOption?.label}
        <span data-testid="dropdown-activenetwork">{selectedOption.label}</span>
      {:else}
        <span data-testid="dropdown-activenetwork">{selectedRef}</span>
      {/if}
    </svelte:fragment>

    <svelte:fragment slot="option" let:option let:ref>
      <div
        data-testid="dropdown-activenetwork-option"
        class="w-full overflow-hidden overflow-ellipsis"
      >
        <div class="text-md break-word mb-2">{option.label ? option.label : ref}</div>
        <Helper class="overflow-hidden overflow-ellipsis break-all text-xs">{option.rpc}</Helper>
      </div>
    </svelte:fragment>
  </DropdownRadio>
{/if}
