<script lang="ts">
	import '../app.css';
	import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { colorTheme } from '$lib/darkMode';
	import { page } from '$app/stores';
	const queryClient = new QueryClient({
		defaultOptions: {
			queries: {
				staleTime: Infinity
			}
		}
	});
</script>

<QueryClientProvider client={queryClient}>
	<div class="flex min-h-screen w-full justify-start bg-white dark:bg-gray-900 dark:text-gray-400">
		<Sidebar {colorTheme} page={$page} />
		<main class="mx-auto h-full w-full grow overflow-x-auto p-8 lg:ml-64">
			<slot />
		</main>
	</div>
</QueryClientProvider>
