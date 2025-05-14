<script lang="ts">
	import { page } from '$app/stores';
	import TransactionsListener from '$lib/components/TransactionsListener.svelte';
	import { useToasts, TransactionProvider } from '@rainlanguage/ui-components';
	import { TransactionManager } from '@rainlanguage/ui-components';
	import { wagmiConfig } from '$lib/stores/wagmi';
	import { useQueryClient } from '@tanstack/svelte-query';

	const { orderHash } = $page.params;
</script>

<TransactionsListener queryKey={orderHash}>
	<slot />
</TransactionsListener>
