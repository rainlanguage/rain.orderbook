<script lang="ts">
  import { Helper } from 'flowbite-svelte';
  import InputHex from '$lib/components/InputHex.svelte';

  export let value: bigint;
</script>

<div class="w-full">
  <InputHex bind:value {...$$props} />
  <Helper class="mt-2 text-sm">
    A hex identifier to distinguish this Vault from others with the same Token and Owner
  </Helper>
</div>
