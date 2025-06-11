<script lang="ts">
	import { TransactionProvider, useToasts } from '@rainlanguage/ui-components';
	import { wagmiConfig } from '$lib/stores/wagmi';

	const { addToast } = useToasts();
</script>

<TransactionProvider {addToast} {wagmiConfig}>
	<slot />
</TransactionProvider>
