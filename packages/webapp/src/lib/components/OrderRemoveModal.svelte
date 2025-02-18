<script lang="ts">
	import { transactionStore } from '@rainlanguage/ui-components';
	import TransactionModal from './TransactionModal.svelte';
	import type { SgOrder } from '@rainlanguage/orderbook/js_api';
	import { getRemoveOrderCalldata } from '@rainlanguage/orderbook/js_api';
	import type { Config } from 'wagmi';

	export let open: boolean;
	export let order: SgOrder;
	export let onRemove: () => void;
	export let wagmiConfig: Config;
	export let chainId: number;
	export let orderbookAddress: string;

	const messages = {
		success: 'Order was successfully removed.',
		pending: 'Removing order...',
		error: 'Could not remove order.'
	};

	function handleSuccess() {
		setTimeout(() => {
			onRemove();
		}, 5000);
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
			orderbookAddress: orderbookAddress as `0x${string}`,
			chainId
		});
	}

	handleTransaction();
</script>

<TransactionModal bind:open {messages} on:close={handleClose} on:success={handleSuccess} />
