<script lang="ts">
	import { transactionStore, type OrderRemoveArgs } from '@rainlanguage/ui-components';
	import TransactionModal from './TransactionModal.svelte';
	import { getRemoveOrderCalldata } from '@rainlanguage/orderbook';
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

	function handleSuccess() {
		setTimeout(() => {
			args.onRemove();
		}, 5000);
	}

	async function handleTransaction() {
		const res = await getRemoveOrderCalldata(args.order);
		if (res.error) {
			throw new Error(res.error.msg);
		}
		transactionStore.handleRemoveOrderTransaction({
			config: $wagmiConfig,
			...args,
			removeOrderCalldata: res.value
		});
	}

	handleTransaction();
</script>

<TransactionModal bind:open {messages} on:close={handleClose} on:success={handleSuccess} />
