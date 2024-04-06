<script lang="ts">
  import { isEmpty } from 'lodash';
  import DropdownActiveNetwork from './DropdownActiveNetwork.svelte';
  import DropdownActiveOrderbook from './DropdownActiveOrderbook.svelte';
  import { settings } from '$lib/stores/settings';
  import { Alert } from 'flowbite-svelte';
</script>

<div class="flex min-w-[500px] items-center justify-end gap-x-2">
  {#if isEmpty($settings?.networks)}
    <Alert color="gray"
      >No networks added to <a class="underline" href="/settings">settings</a></Alert
    >
  {:else}
    <DropdownActiveNetwork />
    <DropdownActiveOrderbook />
  {/if}
</div>
