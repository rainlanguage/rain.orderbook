<script lang="ts">
  import { timestampSecondsToUTCTimestamp } from '$lib/utils/time';
  import { bigintToFloat } from '$lib/utils/number';
  import type { Vault } from '$lib/typeshare/subgraphTypes';
  import { createQuery } from '@tanstack/svelte-query';
  import { vaultBalanceChangesList } from '$lib/queries/vaultBalanceChangesList';
  import { subgraphUrl } from '$lib/stores/settings';
  import TanstackLightweightChartLine from './TanstackLightweightChartLine.svelte';
  import { QKEY_VAULT_CHANGES } from '$lib/queries/keys';

  export let vault: Vault;

  $: query = createQuery({
    queryKey: [QKEY_VAULT_CHANGES, vault.id],
    queryFn: () => {
      return vaultBalanceChangesList(vault.id, $subgraphUrl || '', 0, 1000);
    },
    enabled: !!$subgraphUrl,
  });
</script>

{#if vault}
  <TanstackLightweightChartLine
    title="Balance history"
    priceSymbol={vault.token.symbol}
    {query}
    timeTransform={(d) => timestampSecondsToUTCTimestamp(BigInt(d.timestamp))}
    valueTransform={(d) =>
      bigintToFloat(BigInt(d.newVaultBalance), Number(vault.token.decimals ?? 0))}
    emptyMessage="No deposits or withdrawals found"
  />
{/if}
