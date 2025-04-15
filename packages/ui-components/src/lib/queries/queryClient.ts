import { browser } from '$app/environment';
import { QueryClient } from '@tanstack/svelte-query';

export const queryClient = new QueryClient({
	defaultOptions: {
		queries: {
			enabled: browser
		}
	}
});

export const invalidateTanstackQueries = (queryClient: QueryClient, queryKey: string[]) => {
	queryClient.invalidateQueries({
		queryKey,
		refetchType: 'all',
		exact: false
	});
};
