import { browser } from '$app/environment';
import { QueryClient } from '@tanstack/svelte-query';

export const queryClient = new QueryClient({
	defaultOptions: {
		queries: {
			enabled: browser
		}
	}
});

export const invalidateIdQuery = async (queryClient: QueryClient, id: string) => {
	await queryClient.invalidateQueries({
		queryKey: [id],
		refetchType: 'all',
		exact: false
	});
};
