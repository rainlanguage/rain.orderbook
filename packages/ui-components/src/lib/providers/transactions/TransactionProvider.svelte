<script lang="ts">
	import { setTransactionManagerContext } from './context';
	import type { TransactionManager } from './TransactionManager';

	export let manager: TransactionManager;

	setTransactionManagerContext(manager);
</script>

<slot />
