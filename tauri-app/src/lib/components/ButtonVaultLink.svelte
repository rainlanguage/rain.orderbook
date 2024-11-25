<script lang="ts">
  import { goto } from '$app/navigation';
  import type { Vault } from '$lib/typeshare/subgraphTypes';
  import { bigintStringToHex } from '@rainlanguage/ui-components';
  import { Tooltip } from 'flowbite-svelte';
  import { formatUnits } from 'viem';

  export let tokenVault: Vault;
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
  class="cursor-pointer rounded-lg"
  id="token-info"
  on:click={() => goto(`/vaults/${tokenVault.id}`)}
>
  <div class="flex flex-col space-y-2">
    <div class="flex items-center justify-between">
      <Tooltip triggeredBy="#token-info" class="w-96">
        ID: <span class="font-mono">{bigintStringToHex(tokenVault.vaultId)}</span>
      </Tooltip>
      <span class="font-medium">
        {tokenVault.token.name} ({tokenVault.token.symbol})
      </span>
      <span class="text-sm text-gray-500 dark:text-gray-400">
        {formatUnits(BigInt(tokenVault.balance), parseInt(tokenVault.token.decimals || '18'))}
      </span>
    </div>
  </div>
</div>
