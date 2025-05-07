import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { queryClient, invalidateTanstackQueries } from '../lib/queries/queryClient';
import { QueryClient } from '@tanstack/svelte-query';

describe('queryClient module', () => {
	describe('queryClient instance', () => {
		it('creates a QueryClient instance with correct configuration', () => {
			expect(queryClient).toBeInstanceOf(QueryClient);
		});
	});

	describe('invalidateTanstackQueries', () => {
		let testQueryClient: QueryClient;

		beforeEach(() => {
			testQueryClient = new QueryClient();
			vi.spyOn(testQueryClient, 'invalidateQueries');
		});

		afterEach(() => {
			vi.restoreAllMocks();
		});

		it('calls invalidateQueries with the correct parameters', () => {
			const queryKey = ['test', 'query'];

			invalidateTanstackQueries(testQueryClient, queryKey);

			expect(testQueryClient.invalidateQueries).toHaveBeenCalledTimes(1);
			expect(testQueryClient.invalidateQueries).toHaveBeenCalledWith({
				queryKey,
				refetchType: 'all',
				exact: false
			});
		});

		it('works with nested query keys', () => {
			const queryKey = ['orders', 'details', '123'];

			invalidateTanstackQueries(testQueryClient, queryKey);

			expect(testQueryClient.invalidateQueries).toHaveBeenCalledWith({
				queryKey,
				refetchType: 'all',
				exact: false
			});
		});

		it('works with single-item query keys', () => {
			const queryKey = ['globalData'];

			invalidateTanstackQueries(testQueryClient, queryKey);

			expect(testQueryClient.invalidateQueries).toHaveBeenCalledWith({
				queryKey,
				refetchType: 'all',
				exact: false
			});
		});

		it('invalidates all matching queries due to exact: false', async () => {
			// Setup some test queries
			const parentKey = ['orders'];
			const childKey1 = ['orders', '123'];
			const childKey2 = ['orders', '456'];
			const unrelatedKey = ['products'];

			vi.spyOn(testQueryClient, 'refetchQueries');

			await testQueryClient.prefetchQuery({
				queryKey: parentKey,
				queryFn: () => 'parent data'
			});

			await testQueryClient.prefetchQuery({
				queryKey: childKey1,
				queryFn: () => 'child1 data'
			});

			await testQueryClient.prefetchQuery({
				queryKey: childKey2,
				queryFn: () => 'child2 data'
			});

			await testQueryClient.prefetchQuery({
				queryKey: unrelatedKey,
				queryFn: () => 'unrelated data'
			});

			invalidateTanstackQueries(testQueryClient, parentKey);

			expect(testQueryClient.refetchQueries).toHaveBeenCalled();
		});
	});
});
