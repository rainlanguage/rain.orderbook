import { render, screen, waitFor } from '@testing-library/svelte';
import { test, expect } from 'vitest';
import TanstackAppTableTest from './TanstackAppTable.test.svelte';
import userEvent from '@testing-library/user-event';
import { createResolvableInfiniteQuery } from '../lib/__mocks__/queries';
import { writable } from 'svelte/store';
import type { CreateInfiniteQueryResult, InfiniteData } from '@tanstack/svelte-query';

test('shows head and title', async () => {
	const { query, resolve } = createResolvableInfiniteQuery((pageParam) => {
		return ['page' + pageParam];
	});

	render(TanstackAppTableTest, {
		query,
		emptyMessage: 'No rows',
		title: 'Test Table',
		head: 'Test head',
		queryKey: 'test'
	});

	resolve();

	await waitFor(() => expect(screen.getByTestId('head')).toHaveTextContent('Test head'));
	expect(screen.getByTestId('title')).toHaveTextContent('Test Table');
});

test('renders rows', async () => {
	const { query, resolve } = createResolvableInfiniteQuery((pageParam) => {
		return ['page' + pageParam];
	});

	render(TanstackAppTableTest, {
		query,
		emptyMessage: 'No rows',
		title: 'Test Table',
		head: 'Test head',
		queryKey: 'test'
	});

	resolve();
	await waitFor(() => expect(screen.getByTestId('bodyRow')).toHaveTextContent('page0'));
});

test('shows empty message', async () => {
	const { query, resolve } = createResolvableInfiniteQuery(() => {
		return [];
	});

	render(TanstackAppTableTest, {
		query,
		emptyMessage: 'No rows',
		title: 'Test Table',
		head: 'Test head',
		queryKey: 'test'
	});

	resolve();

	await waitFor(() => expect(screen.getByTestId('emptyMessage')).toHaveTextContent('No rows'));
});

test('loads more rows', async () => {
	const { query, resolve } = createResolvableInfiniteQuery((pageParam) => {
		return ['page' + pageParam];
	});

	render(TanstackAppTableTest, {
		query,
		emptyMessage: 'No rows',
		title: 'Test Table',
		head: 'Test head',
		queryKey: 'test'
	});

	resolve();

	await waitFor(() => expect(screen.getByTestId('bodyRow')).toHaveTextContent('page0'));

	// loading more rows
	const loadMoreButton = screen.getByTestId('loadMoreButton');
	await userEvent.click(loadMoreButton);

	resolve();

	await waitFor(() => {
		expect(screen.getAllByTestId('bodyRow')).toHaveLength(2);
	});

	let rows = screen.getAllByTestId('bodyRow');

	expect(rows).toHaveLength(2);
	expect(rows[0]).toHaveTextContent('page0');
	expect(rows[1]).toHaveTextContent('page1');

	// loading more rows
	await userEvent.click(loadMoreButton);

	resolve();

	await waitFor(() => {
		expect(screen.getAllByTestId('bodyRow')).toHaveLength(3);
	});

	rows = screen.getAllByTestId('bodyRow');

	expect(rows).toHaveLength(3);
	expect(rows[0]).toHaveTextContent('page0');
	expect(rows[1]).toHaveTextContent('page1');
	expect(rows[2]).toHaveTextContent('page2');
});

test('load more button message changes when loading', async () => {
	const { query, resolve } = createResolvableInfiniteQuery((pageParam) => {
		return ['page' + pageParam];
	});

	render(TanstackAppTableTest, {
		query,
		emptyMessage: 'No rows',
		title: 'Test Table',
		head: 'Test head',
		queryKey: 'test'	
	});

	resolve();

	expect(await screen.findByTestId('loadMoreButton')).toHaveTextContent('Load More');

	// loading more rows
	const loadMoreButton = screen.getByTestId('loadMoreButton');
	await userEvent.click(loadMoreButton);

	expect(await screen.findByTestId('loadMoreButton')).toHaveTextContent('Loading more...');

	resolve();

	await waitFor(() => {
		expect(screen.getByTestId('loadMoreButton')).toHaveTextContent('Load More');
	});
});

test('shows refresh icon', async () => {
	const { query, resolve } = createResolvableInfiniteQuery((pageParam) => {
		return ['page' + pageParam];
	});

	render(TanstackAppTableTest, {
		query,
		emptyMessage: 'No rows',
		title: 'Test Table',
		head: 'Test head',
		queryKey: 'test'
	});

	resolve();

	await waitFor(() => expect(screen.getByTestId('refreshButton')).toBeInTheDocument());
});

test('refetches data when refresh button is clicked', async () => {
	const mockRefetch = vi.fn();
	const mockQuery = writable({
		status: 'success',
		fetchStatus: 'idle',
		refetch: mockRefetch
	});

	render(TanstackAppTableTest, {
		query: mockQuery as unknown as CreateInfiniteQueryResult<
			InfiniteData<unknown[], unknown>,
			Error
		>,
		emptyMessage: 'No rows',
		title: 'Test Table',
		head: 'Test head',
		queryKey: 'test'
	});

	const refreshButton = screen.getByTestId('refreshButton');
	await userEvent.click(refreshButton);

	expect(mockRefetch).toHaveBeenCalled();
});


// TODO, add test that invalidate works