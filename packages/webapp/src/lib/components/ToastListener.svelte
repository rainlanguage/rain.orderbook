<script lang="ts">
	import {
		TransactionStatus,
		useToasts,
		transactionStore,
		invalidateTanstackQueries
	} from '@rainlanguage/ui-components';
	import { useQueryClient } from '@tanstack/svelte-query';
	const { addToast } = useToasts();
	const queryClient = useQueryClient();

	export let queryKey: string;
	$: if ($transactionStore.status === TransactionStatus.SUCCESS) {
		addToast({
			message: $transactionStore.message,
			type: 'success',
			color: 'green'
		});
		invalidateTanstackQueries(queryClient, [queryKey]);
	}

	$: if ($transactionStore.status === TransactionStatus.ERROR) {
		addToast({
			message: $transactionStore.error,
			type: 'error',
			color: 'red'
		});
	}
</script>

<slot />
