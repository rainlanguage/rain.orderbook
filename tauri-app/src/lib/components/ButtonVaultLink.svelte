<script lang="ts">
  import { goto } from '$app/navigation';
  import type { Vault } from '$lib/typeshare/subgraphTypes';
  import { bigintStringToHex } from '$lib/utils/hex';

  export let tokenVault: Vault;
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
  class="cursor-pointer hover:text-primary-600"
  on:click={() => goto(`/vaults/${tokenVault.id}`)}
>
  <div class="grid grid-cols-2 gap-x-2 break-normal">
    <div>
      {tokenVault.token.name} <span class="break-normal">({tokenVault.token.symbol})</span>
    </div>
    <div>
      <span class="text-gray-500 dark:text-gray-400">ID</span>
      <span class="mb-1 break-all">
        {bigintStringToHex(tokenVault.vault_id)}
      </span>
    </div>
  </div>
</div>
