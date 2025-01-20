<script lang="ts">
	import '../app.css';
	import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { colorTheme } from '$lib/darkMode';
	import { browser } from '$app/environment';
	import { SupportedChainsList } from '$lib/chains';
	import {
		connected,
		disconnectWagmi,
		signerAddress,
		wagmiConfig,
		configuredConnectors,
		loading,
		chainId,
		appKitModal,
		defaultConfig,
		wagmiLoaded
	} from '$lib/stores/wagmi';

	const queryClient = new QueryClient({
		defaultOptions: {
			queries: {
				staleTime: Infinity
			}
		}
	});

	const initWallet = async () => {
		const erckit = defaultConfig({
			chains: SupportedChainsList,
			projectId: '10e64d97c18d5dbeecfb11d7834e2682'
		});
		await erckit.init();
	};

	$: console.log('wagmiconfig', $wagmiConfig);

	$: if (browser && window.navigator) {
		initWallet();
	}
</script>

<QueryClientProvider client={queryClient}>
	<div class="flex min-h-screen w-full justify-start bg-white dark:bg-gray-900 dark:text-gray-400">
		<Sidebar {colorTheme} />
		<main class="ml-64 h-full w-full grow overflow-x-auto p-8">
			<slot />
		</main>
	</div>
</QueryClientProvider>
