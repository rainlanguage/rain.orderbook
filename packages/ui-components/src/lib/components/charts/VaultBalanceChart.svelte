<script lang="ts">
	import { timestampSecondsToUTCTimestamp } from '../../utils/time';
	import { bigintToFloat } from '../../utils/number';
	import type { Vault } from '../../typeshare/subgraphTypes';
	import { createQuery } from '@tanstack/svelte-query';
	import { getVaultBalanceChanges } from '@rainlanguage/orderbook/js_api';
	import TanstackLightweightChartLine from '../charts/TanstackLightweightChartLine.svelte';
	import { QKEY_VAULT_CHANGES } from '../../queries/keys';

	export let vault: Vault;
	export let subgraphUrl: string;
	export let lightweightChartsTheme;

	$: query = createQuery({
		queryKey: [QKEY_VAULT_CHANGES, vault],
		queryFn: () => {
			return getVaultBalanceChanges(subgraphUrl || '', vault.id, {
				page: 1,
				pageSize: 1000
			});
		},
		enabled: !!subgraphUrl
	});
</script>

{#if vault && $query.data}
	<TanstackLightweightChartLine
		title="Balance history"
		priceSymbol={vault.token.symbol}
		{query}
		timeTransform={(d) => timestampSecondsToUTCTimestamp(BigInt(d.timestamp))}
		valueTransform={(d) =>
			bigintToFloat(BigInt(d.newVaultBalance), Number(vault.token.decimals ?? 0))}
		emptyMessage="No deposits or withdrawals found"
		{lightweightChartsTheme}
	/>
{/if}
