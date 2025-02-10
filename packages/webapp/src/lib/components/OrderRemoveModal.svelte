<script lang="ts">
	import { transactionStore } from '@rainlanguage/ui-components';
	import TransactionModal from './TransactionModal.svelte';
	import type { OrderSubgraph } from '@rainlanguage/orderbook/js_api';
	import { getRemoveOrderCalldata } from '@rainlanguage/orderbook/js_api';
	import type { Config } from 'wagmi';
	import type { Hex } from 'viem';

	export let open: boolean;
	export let order: OrderSubgraph;
	export let onRemove: () => void;
	export let wagmiConfig: Config;
	export let chainId: number;
	export let orderbookAddress: Hex;

	const messages = {
		success: 'Order was successfully removed.',
		pending: 'Removing order...',
		error: 'Could not remove order.'
	};

	function handleSuccess() {
		onRemove();
	}

	function handleClose() {
		transactionStore.reset();
		open = false;
	}

	async function handleTransaction() {
		const removeOrderCalldata = await getRemoveOrderCalldata(order);
		transactionStore.handleRemoveOrderTransaction({
			config: wagmiConfig,
			removeOrderCalldata,
			orderbookAddress,
			chainId
		});
	}

	handleTransaction();
</script>

<TransactionModal bind:open {messages} on:close={handleClose} on:success={handleSuccess} />
