<script lang="ts">
	import { TransactionStatus, useToasts, transactionStore } from '@rainlanguage/ui-components';

	const { addToast } = useToasts();

	$: if ($transactionStore.status === TransactionStatus.SUCCESS) {
		addToast({
			message: $transactionStore.message,
			type: 'success',
			color: 'green'
		});
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
