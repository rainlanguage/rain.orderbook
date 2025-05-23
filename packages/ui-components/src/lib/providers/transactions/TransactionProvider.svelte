<script lang="ts">
	import type { Config } from '@wagmi/core';
	import { setTransactionManagerContext } from './context';
	import { TransactionManager } from './TransactionManager';
	import { useQueryClient } from '@tanstack/svelte-query';
	import type { ToastProps } from '$lib/types/toast';
	import type { Writable } from 'svelte/store';

	export let addToast: (toast: ToastProps) => void;
	export let wagmiConfig: Writable<Config>;
	const queryClient = useQueryClient();

	const manager = new TransactionManager(queryClient, addToast, $wagmiConfig);

	setTransactionManagerContext(manager);
</script>

<slot />
