<script lang="ts">
	import { timestampSecondsToUTCTimestamp } from '../../utils/time';
	import { bigintToFloat } from '../../utils/number';
	import type { Vault } from '@rainlanguage/orderbook/js_api';
	import { createQuery } from '@tanstack/svelte-query';
	import {
		getVaultBalanceChanges,
		type ClearBounty,
		type Deposit,
		type TradeVaultBalanceChange,
		type Withdrawal
	} from '@rainlanguage/orderbook/js_api';
	import TanstackLightweightChartLine from '../charts/TanstackLightweightChartLine.svelte';
	import { QKEY_VAULT_CHANGES } from '../../queries/keys';

	export let vault: Vault;
	export let id: string;
	export let subgraphUrl: string;
	export let lightweightChartsTheme;

	$: query = createQuery({
		queryKey: [id, QKEY_VAULT_CHANGES + id, QKEY_VAULT_CHANGES],
		queryFn: () => {
			console.log('✅ geting vault balance chart');
			return getVaultBalanceChanges(subgraphUrl || '', vault.id, {
				page: 1,
				pageSize: 1000
			});
		},
		enabled: !!subgraphUrl
	});

	const Chart = TanstackLightweightChartLine<
		Withdrawal | Deposit | TradeVaultBalanceChange | ClearBounty
	>;
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
