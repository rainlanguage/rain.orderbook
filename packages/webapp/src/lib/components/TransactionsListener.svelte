<script lang="ts">
	import {
		TransactionStatus,
		useToasts,
		transactionStore,
		invalidateTanstackQueries
	} from '@rainlanguage/ui-components';
	import { useQueryClient } from '@tanstack/svelte-query';
	const { addToast, errToast } = useToasts();
	const queryClient = useQueryClient();

	/**
	 * The query key to invalidate when a transaction is successful.
	 * This ensures that data is refreshed after a transaction completes.
	 */
	export let queryKey: string;

	/**
	 * Listens for successful transactions and shows a success toast.
	 * Also invalidates the specified query to refresh data.
	 */
	$: if ($transactionStore.status === TransactionStatus.SUCCESS) {
		addToast({
			message: $transactionStore.message,
			type: 'success',
			color: 'green'
		});
		invalidateTanstackQueries(queryClient, [queryKey]);
	}

	/**
	 * Listens for transaction errors and shows an error toast.
	 */
	$: if ($transactionStore.status === TransactionStatus.ERROR) {
		errToast('Transaction failed.', $transactionStore.error);
	}
</script>

<slot />
