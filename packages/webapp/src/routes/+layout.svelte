<script lang="ts">
	import '../app.css';
	import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
	import { settings, activeSubgraphs } from '$lib/stores/settings';
	import type { AppStoresInterface } from '../types';

	const queryClient = new QueryClient({
		defaultOptions: {
			queries: {
				staleTime: Infinity
			}
		}
	});

	const stores: AppStoresInterface = {
		settings,
		activeSubgraphs
	};
</script>

<QueryClientProvider client={queryClient}>
	<div
		class="mb-10 flex h-[calc(100vh-2.5rem)] w-full justify-start bg-white dark:bg-gray-900 dark:text-gray-400"
	>
		<main class="ml-64 h-full w-full grow overflow-x-auto border-4 border-red-500 p-8">
			<slot {stores} />
		</main>
	</div>
</QueryClientProvider>
