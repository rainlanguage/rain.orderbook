<script lang="ts">
	import '../app.css';
	import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { colorTheme } from '$lib/darkMode';
	import { page } from '$app/stores';
	import { browser } from '$app/environment';
	import { supportedChainsList } from '$lib/chains';
	import { defaultConfig } from '$lib/stores/wagmi';
	import { injected } from '@wagmi/connectors';
	import { type Chain } from '@wagmi/core/chains';
	import { PUBLIC_WALLETCONNECT_PROJECT_ID } from '$env/static/public';

	const queryClient = new QueryClient({
		defaultOptions: {
			queries: {
				staleTime: Infinity
			}
		}
	});

	const initWallet = async () => {
		const erckit = defaultConfig({
			appName: 'Rain Language',
			connectors: [injected()],
			chains: supportedChainsList as unknown as Chain[],
			projectId: PUBLIC_WALLETCONNECT_PROJECT_ID
		});
		await erckit.init();
	};

	$: if (browser && window.navigator) {
		initWallet();
	}
</script>

<QueryClientProvider client={queryClient}>
	<div class="flex min-h-screen w-full justify-start bg-white dark:bg-gray-900 dark:text-gray-400">
		<Sidebar {colorTheme} page={$page} />
		<main class="mx-auto h-full w-full grow overflow-x-auto pl-20 pt-8 lg:ml-64 lg:p-8">
			<slot />
		</main>
	</div>
</QueryClientProvider>
