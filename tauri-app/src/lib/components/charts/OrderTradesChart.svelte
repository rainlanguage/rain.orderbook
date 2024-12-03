<script lang="ts">
  import { getOrderTradesList } from '@rainlanguage/orderbook/js_api';
  import {
    prepareHistoricalOrderChartData,
    QKEY_ORDER_TRADES_LIST,
  } from '@rainlanguage/ui-components';
  import { createQuery } from '@tanstack/svelte-query';
  import { subgraphUrl } from '$lib/stores/settings';
  import { colorTheme } from '$lib/stores/darkMode';
  import { get } from 'svelte/store';
  import { TanstackLightweightChartLine } from '@rainlanguage/ui-components';
  import { lightweightChartsTheme } from '$lib/stores/darkMode';

  export let id: string;

  $: query = createQuery({
    queryKey: [QKEY_ORDER_TRADES_LIST, id],
    queryFn: async () => {
      const data = await getOrderTradesList(
        $subgraphUrl || '',
        id,
        {
          page: 1,
          pageSize: 10,
        },
        BigInt(1000),
        undefined,
      );
      return prepareHistoricalOrderChartData(data, get(colorTheme));
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
  {lightweightChartsTheme}
/>
