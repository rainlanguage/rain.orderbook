<script lang="ts" generics="T">
  import { Button, Dropdown, Radio } from 'flowbite-svelte';
  import { ChevronDownSolid } from 'flowbite-svelte-icons';

  // eslint-disable-next-line no-undef
  export let options: Record<string, T> = {};
  export let value: string | undefined = undefined;
  let open = false;

  $: value, open = false;
</script>

<Button color="alternative" class="w-full pl-2 pr-0 text-left flex justify-between overflow-hidden overflow-ellipsis">
  <div class="flex-grow overflow-hidden"><slot name="content" selectedRef={value} selectedOption={value !== undefined ? options[value] : undefined}></slot></div>
  <ChevronDownSolid class="w-3 h-3 mx-2 text-black dark:text-white" />
</Button>

<Dropdown class="py-0 w-72" bind:open>
  {#each Object.entries(options) as [ref, option]}
    <Radio bind:group={value} value={ref} class="w-full p-3 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-600">
      <div class="ml-2">
        <slot name="option" {option} {ref}></slot>
      </div>
    </Radio>
  {/each}
</Dropdown>