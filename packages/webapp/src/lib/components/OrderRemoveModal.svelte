<script lang="ts">
	import { transactionStore, type OrderRemoveArgs } from '@rainlanguage/ui-components';
	import TransactionModal from './TransactionModal.svelte';
	import type { OrderSubgraph } from '@rainlanguage/orderbook/js_api';
	import { getRemoveOrderCalldata } from '@rainlanguage/orderbook/js_api';
	import { wagmiConfig } from '$lib/stores/wagmi';

	export let open: boolean;
	export let args: OrderRemoveArgs;

	const messages = {
		success: 'Order was successfully removed.',
		pending: 'Removing order...',
		error: 'Could not remove order.'
	};

	function handleClose() {
		transactionStore.reset();
		open = false;
	}

	async function handleTransaction() {
		const removeOrderCalldata = await getRemoveOrderCalldata(args.order);
		transactionStore.handleRemoveOrderTransaction({
			config: $wagmiConfig,
			...args,
			removeOrderCalldata
		});
	}

	handleTransaction();
</script>

<TransactionModal bind:open {messages} on:close={handleClose} />
