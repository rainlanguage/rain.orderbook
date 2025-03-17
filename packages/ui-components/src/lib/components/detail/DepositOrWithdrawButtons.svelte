<script lang="ts">
	import type { SgVault } from '@rainlanguage/orderbook/js_api';
	import { Button } from 'flowbite-svelte';
	import { ArrowDownOutline, ArrowUpOutline } from 'flowbite-svelte-icons';
	import type { DepositOrWithdrawModalProps } from '../../types/modal';
	export let handleDepositOrWithdrawModal: (props: DepositOrWithdrawModalProps) => void;

	export let vault: SgVault;
	export let chainId: number;
	export let rpcUrl: string;
	export let refetch: () => void;
	export let subgraphUrl: string;
</script>

<Button
	data-testid="depositOrWithdrawButton"
	color="light"
	size="xs"
	on:click={() =>
		handleDepositOrWithdrawModal({
			open: true,
			args: {
				vault,
				onDepositOrWithdraw: refetch,
				action: 'deposit',
				chainId,
				rpcUrl,
				subgraphUrl
			}
		})}><ArrowUpOutline size="xs" /></Button
>
<Button
	data-testid="depositOrWithdrawButton"
	color="light"
	size="xs"
	on:click={() =>
		handleDepositOrWithdrawModal({
			open: true,
			args: {
				vault,
				onDepositOrWithdraw: refetch,
				action: 'withdraw',
				chainId,
				rpcUrl,
				subgraphUrl
			}
		})}><ArrowDownOutline size="xs" /></Button
>
