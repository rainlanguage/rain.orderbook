<script lang="ts">
  import LightweightChartLine from '$lib/components/charts/LightweightChartLine.svelte';
  import { timestampSecondsToUTCTimestamp } from '$lib/utils/time';
  import { sortBy } from 'lodash';
  import { bigintToFloat } from '$lib/utils/number';
  import type { UTCTimestamp } from 'lightweight-charts';
  import type { Vault } from '$lib/typeshare/vaultDetail';
  import { createQuery } from '@tanstack/svelte-query';
  import { vaultBalanceChangesList } from '$lib/queries/vaultBalanceChangesList';
  import { subgraphUrl } from '$lib/stores/settings';

  export let vault: Vault;

  $: balanceChangesQuery = createQuery({
    queryKey: ['vaultBalanceChanges', vault.id],
    queryFn: () => {
      return vaultBalanceChangesList(vault.id, $subgraphUrl || '', 0, 1000);
    },
    enabled: !!$subgraphUrl,
  });

  let vaultBalanceChangesChartData: { value: number; time: UTCTimestamp }[] = [];

  function prepareChartData(vault: Vault) {
    const transformedData = $balanceChangesQuery.data?.map((d) => ({
      value: bigintToFloat(BigInt(d.new_vault_balance), Number(vault.token.decimals ?? 0)),
      time: timestampSecondsToUTCTimestamp(BigInt(d.timestamp)),
    }));
    return sortBy(transformedData, (d) => d.time);
  }

  $: if ($balanceChangesQuery.data && vault) vaultBalanceChangesChartData = prepareChartData(vault);
</script>

<LightweightChartLine
  title="Balance history"
  priceSymbol={vault.token.symbol}
  data={vaultBalanceChangesChartData}
  loading={$balanceChangesQuery.isLoading}
  emptyMessage="No deposits or withdrawals found"
/>
