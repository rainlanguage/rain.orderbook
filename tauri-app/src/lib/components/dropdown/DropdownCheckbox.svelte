<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Button, Dropdown, Label, Checkbox } from 'flowbite-svelte';
  import { ChevronDownSolid } from 'flowbite-svelte-icons';
  import { isEmpty } from 'lodash';

  const dispatch = createEventDispatcher();

  export let options: string[] = [];
  export let value: string[] = [];

  export let label: string = 'Select items';
  export let allLabel: string = 'All items';
  export let emptyMessage: string = 'No items available';

  function toggleAll() {
    if (value.length === options.length) {
      value = [];
    } else {
      value = [...options];
    }
    dispatch('change', value);
  }

  function toggleItem(item: string) {
    if (value.includes(item)) {
      value = value.filter((i) => i !== item);
    } else {
      value = [...value, item];
    }
    dispatch('change', value);
  }

  $: selectedCount = value.length;
</script>

<Label>{label}</Label>
<div>
  <Button
    color="alternative"
    class="flex w-full justify-between overflow-hidden overflow-ellipsis pl-2 pr-0 text-left"
  >
    <div class="flex-grow overflow-hidden text-ellipsis whitespace-nowrap">
      {selectedCount === 0
        ? `Select items`
        : selectedCount === options.length
          ? allLabel
          : `${selectedCount} item${selectedCount > 1 ? 's' : ''}`}
    </div>
    <ChevronDownSolid class="mx-2 h-3 w-3 text-black dark:text-white" />
  </Button>

  <Dropdown class="w-full min-w-72 py-0">
    {#if isEmpty(options)}
      <div class="ml-2 w-full rounded-lg p-3">{emptyMessage}</div>
    {:else if options.length > 1}
      <Checkbox
        class="w-full rounded-lg p-3 hover:bg-gray-100 dark:hover:bg-gray-600"
        on:click={toggleAll}
        checked={value.length === options.length}
      >
        <div class="ml-2">{allLabel}</div>
      </Checkbox>
    {/if}

    {#each options as item}
      <Checkbox
        class="w-full rounded-lg p-3 hover:bg-gray-100 dark:hover:bg-gray-600"
        on:click={() => toggleItem(item)}
        checked={value.includes(item)}
      >
        <div class="ml-2">
          <slot name="item" {item}>
            {item}
          </slot>
        </div>
      </Checkbox>
    {/each}
  </Dropdown>
</div>
