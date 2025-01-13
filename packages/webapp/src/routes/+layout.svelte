<script lang="ts">
	import '../app.css';
	import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
	import { SidebarWebapp } from '@rainlanguage/ui-components';
	import { colorTheme } from '$lib/darkMode';
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
		<SidebarWebapp {colorTheme} />
		<main class="ml-128 h-full w-full grow overflow-x-auto p-8">
			<slot />
		</main>
	</div>
</QueryClientProvider>
