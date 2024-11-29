<script lang="ts">
  import { orderTradesListForChart } from '$lib/queries/orderTradesList';
  import { QKEY_ORDER_TRADES_LIST } from '@rainlanguage/ui-components';
  import { createQuery } from '@tanstack/svelte-query';
  import { subgraphUrl } from '$lib/stores/settings';
  import TanstackLightweightChartLine from './TanstackLightweightChartLine.svelte';

  export let id: string;

  $: query = createQuery({
    queryKey: [QKEY_ORDER_TRADES_LIST, id],
    queryFn: () => {
      return orderTradesListForChart(id, $subgraphUrl || '', 0, 1000);
    },
    enabled: !!$subgraphUrl,
  });
</script>

<TanstackLightweightChartLine
  title="Trades"
  {query}
  timeTransform={(d) => d.time}
  valueTransform={(d) => d.value}
  emptyMessage="No trades found"
/>
