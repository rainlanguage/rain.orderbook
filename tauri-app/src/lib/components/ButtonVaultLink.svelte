<script lang="ts">
  import { goto } from '$app/navigation';
  import type { TokenVault } from '$lib/typeshare/vaultDetail';
  import { bigintStringToHex } from '$lib/utils/hex';

  export let tokenVault: TokenVault;
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
  class="hover:text-primary-600 cursor-pointer"
  on:click={() => goto(`/vaults/${tokenVault.id}`)}
>
  <div class="grid grid-cols-2 gap-x-2 break-normal leading-tight">
    <div class="flex flex-col gap-y-5">
      <div style="margin-top: 1px">
        <span>{tokenVault.token.name} </span>
        <span class="break-normal">({tokenVault.token.symbol})</span>
      </div>
      <div>
        <p class="text-gray-500 dark:text-gray-400">Balance</p>
        <span>{tokenVault.balance_display}</span>
      </div>
    </div>
    <div>
      <span class="text-gray-500 dark:text-gray-400">ID</span>
      <span class="mb-1 break-all">
        {bigintStringToHex(tokenVault.vault_id)}
      </span>
    </div>
  </div>
</div>
