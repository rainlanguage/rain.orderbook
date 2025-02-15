<script lang="ts">
	import type { Vault } from '@rainlanguage/orderbook/js_api';
	import type { CreateQueryResult } from '@tanstack/svelte-query';
	import { Button } from 'flowbite-svelte';
	import { ArrowDownOutline, ArrowUpOutline } from 'flowbite-svelte-icons';

	export let handleDepositOrWithdrawModal: (args: {
		vault: Vault;
		onDepositOrWithdraw: () => void;
		action: 'deposit' | 'withdraw';
		chainId: number;
		rpcUrl: string;
		subgraphUrl: string;
	}) => void;

	export let vault: Vault;
	export let chainId: number;
	export let rpcUrl: string;
	export let query: CreateQueryResult;
	export let subgraphUrl: string;
</script>

<Button
	data-testid="depositOrWithdrawButton"
	color="light"
	size="xs"
	on:click={() =>
		handleDepositOrWithdrawModal({
			vault,
			onDepositOrWithdraw: $query.refetch,
			action: 'deposit',
			chainId,
			rpcUrl,
			subgraphUrl
		})}><ArrowUpOutline size="xs" /></Button
>
<Button
	data-testid="depositOrWithdrawButton"
	color="light"
	size="xs"
	on:click={() =>
		handleDepositOrWithdrawModal({
			vault,
			onDepositOrWithdraw: $query.refetch,
			action: 'withdraw',
			chainId,
			rpcUrl,
			subgraphUrl
		})}><ArrowDownOutline size="xs" /></Button
>
