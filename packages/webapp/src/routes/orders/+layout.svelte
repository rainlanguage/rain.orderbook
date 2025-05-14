<script lang="ts">
	import { page } from '$app/stores';
	import TransactionsListener from '$lib/components/TransactionsListener.svelte';
	import { useToasts, TransactionProvider } from '@rainlanguage/ui-components';
	import { TransactionManager } from '@rainlanguage/ui-components';
	import { wagmiConfig } from '$lib/stores/wagmi';
	import { useQueryClient } from '@tanstack/svelte-query';

	const { orderHash } = $page.params;
	const { addToast } = useToasts();
	const queryClient = useQueryClient();
	const manager = new TransactionManager(queryClient, addToast, $wagmiConfig);
</script>

<TransactionProvider {manager}>
	<TransactionsListener queryKey={orderHash}>
		<slot />
	</TransactionsListener>
</TransactionProvider>
