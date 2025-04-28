import { render, screen, waitFor } from '@testing-library/svelte';
import { test, expect } from 'vitest';
import TanstackAppTableTest from './TanstackAppTable.test.svelte';
import userEvent from '@testing-library/user-event';
import { writable, get } from 'svelte/store';
import type { CreateInfiniteQueryResult, InfiniteData } from '@tanstack/svelte-query';

vi.mock('@tanstack/svelte-query', () => ({
	useQueryClient: () => ({})
}));

const mockInvalidateTanstackQueries = vi.fn();

vi.mock('$lib/queries/queryClient', () => ({
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	invalidateTanstackQueries: (queryClient: any, queryKey: string[]) =>
		mockInvalidateTanstackQueries(queryClient, queryKey)
}));

// Helper function to create base pages
const createPages = (pageData: unknown[] = ['page0']) =>
	writable({
		pages: [pageData],
		pageParams: [0]
	});

// Helper function to create base mock query
const createMockQuery = (pages: ReturnType<typeof createPages>, overrides = {}) => {
	return writable({
		data: get(pages),
		isLoading: false,
		isFetching: false,
		isFetchingNextPage: false,
		hasNextPage: true,
		status: 'success' as const,
		fetchStatus: 'idle' as const,
		fetchNextPage: vi.fn(),
		...overrides
	});
};

// Helper function for common render props
const renderTable = (query: ReturnType<typeof createMockQuery>) => {
	return render(TanstackAppTableTest, {
		query: query as unknown as CreateInfiniteQueryResult<InfiniteData<unknown[], unknown>, Error>,
		emptyMessage: 'No rows',
		title: 'Test Table',
		head: 'Test head',
		queryKey: 'test'
	});
};

test('shows head and title', async () => {
	const pages = createPages();
	const mockQuery = createMockQuery(pages);
	renderTable(mockQuery);

	await waitFor(() => expect(screen.getByTestId('head')).toHaveTextContent('Test head'));
	expect(screen.getByTestId('title')).toHaveTextContent('Test Table');
});

test('renders rows', async () => {
	const pages = createPages();
	const mockQuery = createMockQuery(pages);
	renderTable(mockQuery);

	await waitFor(() => expect(screen.getByTestId('bodyRow')).toHaveTextContent('page0'));
});

test('shows empty message', async () => {
	const pages = createPages([]);
	const mockQuery = createMockQuery(pages);
	renderTable(mockQuery);

	await waitFor(() => expect(screen.getByTestId('emptyMessage')).toHaveTextContent('No rows'));
});

test('loads more rows', async () => {
	const pages = createPages();
	const mockQuery = createMockQuery(pages, {
		fetchNextPage: async () => {
			mockQuery.update((q) => ({ ...q, isFetchingNextPage: true }));
			await new Promise((resolve) => setTimeout(resolve, 0));
			pages.update((data) => ({
				pages: [...data.pages, [`page${data.pages.length}`]],
				pageParams: [...data.pageParams, data.pageParams.length]
			}));
			mockQuery.update((q) => ({
				...q,
				data: get(pages),
				isFetchingNextPage: false
			}));
		}
	});
	renderTable(mockQuery);

	await waitFor(() => expect(screen.getByTestId('bodyRow')).toHaveTextContent('page0'));

	const loadMoreButton = screen.getByTestId('loadMoreButton');
	await userEvent.click(loadMoreButton);

	await waitFor(() => {
		expect(screen.getAllByTestId('bodyRow')).toHaveLength(2);
	});

	let rows = screen.getAllByTestId('bodyRow');
	expect(rows[0]).toHaveTextContent('page0');
	expect(rows[1]).toHaveTextContent('page1');

	await userEvent.click(loadMoreButton);

	await waitFor(() => {
		expect(screen.getAllByTestId('bodyRow')).toHaveLength(3);
	});

	rows = screen.getAllByTestId('bodyRow');
	expect(rows[0]).toHaveTextContent('page0');
	expect(rows[1]).toHaveTextContent('page1');
	expect(rows[2]).toHaveTextContent('page2');
});

test('load more button message changes when loading', async () => {
	const pages = createPages();
	const mockQuery = createMockQuery(pages, {
		fetchNextPage: async () => {
			mockQuery.update((q) => ({ ...q, isFetchingNextPage: true }));
			await new Promise((resolve) => setTimeout(resolve, 100));
			mockQuery.update((q) => ({ ...q, isFetchingNextPage: false }));
		}
	});
	renderTable(mockQuery);

	expect(await screen.findByTestId('loadMoreButton')).toHaveTextContent('Load More');

	const loadMoreButton = screen.getByTestId('loadMoreButton');
	await userEvent.click(loadMoreButton);

	expect(await screen.findByTestId('loadMoreButton')).toHaveTextContent('Loading more...');

	await waitFor(() => {
		expect(screen.getByTestId('loadMoreButton')).toHaveTextContent('Load More');
	});
});

test('shows refresh icon', async () => {
	const pages = createPages();
	const mockQuery = createMockQuery(pages);
	renderTable(mockQuery);

	await waitFor(() => expect(screen.getByTestId('refreshButton')).toBeInTheDocument());
});

test('invalidates queries when refresh button is clicked', async () => {
	const mockRefetch = vi.fn();
	const mockQuery = createMockQuery(createPages(), {
		status: 'success',
		fetchStatus: 'idle',
		isLoading: false,
		isFetching: false,
		refetch: mockRefetch
	});
	renderTable(mockQuery);

	const refreshButton = screen.getByTestId('refreshButton');
	await userEvent.click(refreshButton);

	expect(mockInvalidateTanstackQueries).toHaveBeenCalledWith(expect.anything(), ['test']);
});
