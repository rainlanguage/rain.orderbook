<script lang="ts">
	import { timestampSecondsToUTCTimestamp } from '../../services/time';
	import { bigintToFloat } from '../../utils/number';
	import type { SgVault, SgVaultBalanceChangeUnwrapped } from '@rainlanguage/orderbook';
	import { createQuery } from '@tanstack/svelte-query';
	import { getVaultBalanceChanges } from '@rainlanguage/orderbook';
	import TanstackLightweightChartLine from '../charts/TanstackLightweightChartLine.svelte';
	import { QKEY_VAULT_CHANGES } from '../../queries/keys';

	export let vault: SgVault;
	export let id: string;
	export let subgraphUrl: string;
	export let lightweightChartsTheme;

	$: query = createQuery({
		queryKey: [id, QKEY_VAULT_CHANGES + id, QKEY_VAULT_CHANGES],
		queryFn: async () => {
			const result = await getVaultBalanceChanges(subgraphUrl || '', vault.id, {
				page: 1,
				pageSize: 1000
			});
			if (result.error) throw new Error(result.error.msg);
			return result.value;
		},
		enabled: !!subgraphUrl
	});

	const Chart = TanstackLightweightChartLine<SgVaultBalanceChangeUnwrapped>;
</script>

{#if vault && $query.data}
	<Chart
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
