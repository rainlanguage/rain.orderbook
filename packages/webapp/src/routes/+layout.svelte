<script lang="ts">
	import '../app.css';
	import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { colorTheme } from '$lib/darkMode';
	import { browser } from '$app/environment';
	import { page } from '$app/stores';
	import Homepage from '$lib/components/Homepage.svelte';
	import LoadingWrapper from '$lib/components/LoadingWrapper.svelte';
	import {
		ToastProvider,
		WalletProvider,
		FixedBottomTransaction,
		RaindexClientProvider
	} from '@rainlanguage/ui-components';
	import { signerAddress } from '$lib/stores/wagmi';
	import ErrorPage from '$lib/components/ErrorPage.svelte';
	import TransactionProviderWrapper from '$lib/components/TransactionProviderWrapper.svelte';
	import { initWallet } from '$lib/services/handleWalletInitialization';

	const { errorMessage, raindexClient } = $page.data;

	// Query client for caching
	const queryClient = new QueryClient({
		defaultOptions: {
			queries: {
				staleTime: Infinity
			}
		}
	});

	let walletInitError: string | null = null;

	$: if (browser && window.navigator) {
		initWallet().then((error) => {
			walletInitError = error;
		});
	}
</script>

{#if walletInitError}
	<div
		class="fixed bottom-4 left-1/2 z-[100] -translate-x-1/2 transform rounded-lg bg-red-500 px-6 py-3 text-white shadow-md"
	>
		{walletInitError}
	</div>
{/if}

<ToastProvider>
	<WalletProvider account={signerAddress}>
		<QueryClientProvider client={queryClient}>
			<TransactionProviderWrapper>
				<LoadingWrapper>
					{#if $page.url.pathname === '/'}
						<Homepage {colorTheme} />
					{:else if errorMessage}
						<ErrorPage />
					{:else}
						<RaindexClientProvider {raindexClient}>
							<div
								data-testid="layout-container"
								class="flex min-h-screen w-full justify-start bg-white dark:bg-gray-900 dark:text-gray-400"
							>
								<Sidebar {colorTheme} page={$page} />
								<main class="mx-auto h-full w-full grow overflow-x-auto px-4 pt-14 lg:ml-64 lg:p-8">
									<slot />
								</main>
							</div>
						</RaindexClientProvider>
					{/if}
					<FixedBottomTransaction />
				</LoadingWrapper>
			</TransactionProviderWrapper>
		</QueryClientProvider>
	</WalletProvider>
</ToastProvider>
