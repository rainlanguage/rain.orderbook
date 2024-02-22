<script lang="ts" generics="T">
  import { Button, Dropdown, Radio } from 'flowbite-svelte';
  import { ChevronDownSolid } from 'flowbite-svelte-icons';

  // eslint-disable-next-line no-undef
  export let options: Array<T> = [];
  export let value: number = 0;
  let open = false;

  $: value, open = false;
</script>

<Button color="alternative" class="w-full pl-2 pr-0 text-left flex justify-between overflow-hidden overflow-ellipsis">
  <div class="flex-grow overflow-hidden"><slot name="content" selected={options[value]}></slot></div>
  <ChevronDownSolid class="w-3 h-3 mx-2 text-black dark:text-white" />
</Button>

<Dropdown class="w-72 p-0" bind:open>
  {#each options as option, index}
    <Radio bind:group={value} value={index} class="w-full p-4 rounded hover:bg-gray-100 dark:hover:bg-gray-600">
      <div class="ml-2">
        <slot name="option" {option} {index}></slot>
      </div>
    </Radio>
  {/each}
</Dropdown>