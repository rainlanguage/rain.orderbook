<script lang="ts">
	import '../app.css';
	import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';

	const queryClient = new QueryClient({
		defaultOptions: {
			queries: {
				staleTime: Infinity
			}
		}
	});
</script>

<QueryClientProvider client={queryClient}>
	<div
		class="mb-10 flex h-[calc(100vh-2.5rem)] w-full justify-start bg-white dark:bg-gray-900 dark:text-gray-400"
	>
		<main class="h-full w-full grow overflow-x-auto p-8">
			<slot />
		</main>
	</div>
</QueryClientProvider>
