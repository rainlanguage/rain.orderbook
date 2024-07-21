import { render, screen } from '@testing-library/svelte';
import { test } from 'vitest';
import { expect } from '$lib/test/matchers';
import TanstackAppTableTest from './TanstackAppTable.test.svelte';
import { QueryClient, createInfiniteQuery } from '@tanstack/svelte-query';
import userEvent from '@testing-library/user-event';

const createMockQuery = (maxPages?: number) => {
  return async (pageParam: number) => {
    if (maxPages && pageParam > maxPages) {
      return [];
    }
    const mockData = ['page' + pageParam];
    await new Promise((resolve) => setTimeout(resolve, 10));
    return mockData;
  };
};

test('shows head and title', async () => {
  const query = createInfiniteQuery(
    {
      queryKey: [],
      queryFn: ({ pageParam }) => {
        return createMockQuery()(pageParam);
      },
      initialPageParam: 0,
      getNextPageParam(_lastPage, _allPages, lastPageParam) {
        return lastPageParam + 1;
      },
    },
    new QueryClient({
      defaultOptions: {
        queries: {
          staleTime: Infinity,
        },
      },
    }),
  );

  render(TanstackAppTableTest, {
    query,
    emptyMessage: 'No rows',
    title: 'Test Table',
    head: 'Test head',
  });

  // letting the store update
  await new Promise((resolve) => setTimeout(resolve, 10));

  expect(screen.getByTestId('head')).toHaveTextContent('Test head');

  expect(screen.getByTestId('title')).toHaveTextContent('Test Table');
});

test('renders rows', async () => {
  const query = createInfiniteQuery(
    {
      queryKey: [],
      queryFn: ({ pageParam }) => {
        return createMockQuery()(pageParam);
      },
      initialPageParam: 0,
      getNextPageParam(_lastPage, _allPages, lastPageParam) {
        return lastPageParam + 1;
      },
    },
    new QueryClient({
      defaultOptions: {
        queries: {
          staleTime: Infinity,
        },
      },
    }),
  );

  render(TanstackAppTableTest, {
    query,
    emptyMessage: 'No rows',
    title: 'Test Table',
    head: 'Test head',
  });

  // letting the store update
  await new Promise((resolve) => setTimeout(resolve, 20));

  expect(screen.getByTestId('bodyRow')).toHaveTextContent('page0');
});

test('shows empty message', async () => {
  // creating a query that returns an empty array
  const query = createInfiniteQuery(
    {
      queryKey: [],
      queryFn: () => {
        return [];
      },
      initialPageParam: 0,
      getNextPageParam(_lastPage, _allPages, lastPageParam) {
        return lastPageParam + 1;
      },
    },
    new QueryClient({
      defaultOptions: {
        queries: {
          staleTime: Infinity,
        },
      },
    }),
  );

  render(TanstackAppTableTest, {
    query,
    emptyMessage: 'No rows',
    title: 'Test Table',
    head: 'Test head',
  });

  // letting the store update
  await new Promise((resolve) => setTimeout(resolve, 10));

  expect(screen.getByTestId('emptyMessage')).toHaveTextContent('No rows');
});

test('loads more rows', async () => {
  const query = createInfiniteQuery(
    {
      queryKey: [],
      queryFn: ({ pageParam }) => {
        return createMockQuery()(pageParam);
      },
      initialPageParam: 0,
      getNextPageParam(_lastPage, _allPages, lastPageParam) {
        return lastPageParam + 1;
      },
    },
    new QueryClient({
      defaultOptions: {
        queries: {
          staleTime: Infinity,
        },
      },
    }),
  );

  render(TanstackAppTableTest, {
    query,
    emptyMessage: 'No rows',
    title: 'Test Table',
    head: 'Test head',
  });

  // letting the store update
  await new Promise((resolve) => setTimeout(resolve, 20));

  expect(screen.getByTestId('bodyRow')).toHaveTextContent('page0');

  // loading more rows
  const loadMoreButton = screen.getByTestId('loadMoreButton');
  loadMoreButton.click();

  // letting the store update
  await new Promise((resolve) => setTimeout(resolve, 20));

  let rows = screen.getAllByTestId('bodyRow');

  expect(rows).toHaveLength(2);
  expect(rows[0]).toHaveTextContent('page0');
  expect(rows[1]).toHaveTextContent('page1');

  // loading more rows
  loadMoreButton.click();

  // letting the store update
  await new Promise((resolve) => setTimeout(resolve, 20));

  rows = screen.getAllByTestId('bodyRow');

  expect(rows).toHaveLength(3);
  expect(rows[0]).toHaveTextContent('page0');
  expect(rows[1]).toHaveTextContent('page1');
  expect(rows[2]).toHaveTextContent('page2');
});

test('load more buttton message changes when loading', async () => {
  const query = createInfiniteQuery(
    {
      queryKey: [],
      queryFn: ({ pageParam }) => {
        return createMockQuery()(pageParam);
      },
      initialPageParam: 0,
      getNextPageParam(_lastPage, _allPages, lastPageParam) {
        return lastPageParam + 1;
      },
    },
    new QueryClient({
      defaultOptions: {
        queries: {
          staleTime: Infinity,
        },
      },
    }),
  );

  render(TanstackAppTableTest, {
    query,
    emptyMessage: 'No rows',
    title: 'Test Table',
    head: 'Test head',
  });

  // letting the store update
  await new Promise((resolve) => setTimeout(resolve, 20));

  expect(screen.getByTestId('loadMoreButton')).toHaveTextContent('Load More');

  // loading more rows
  const loadMoreButton = screen.getByTestId('loadMoreButton');
  loadMoreButton.click();

  await new Promise((resolve) => setTimeout(resolve, 1));

  expect(screen.getByTestId('loadMoreButton')).toHaveTextContent('Loading more...');

  // letting the store update
  await new Promise((resolve) => setTimeout(resolve, 20));

  expect(screen.getByTestId('loadMoreButton')).toHaveTextContent('Load More');
});

test('load more buttton is disabled when loading', async () => {
  const query = createInfiniteQuery(
    {
      queryKey: [],
      queryFn: ({ pageParam }) => {
        return createMockQuery()(pageParam);
      },
      initialPageParam: 0,
      getNextPageParam(_lastPage, _allPages, lastPageParam) {
        return lastPageParam + 1;
      },
    },
    new QueryClient({
      defaultOptions: {
        queries: {
          staleTime: Infinity,
        },
      },
    }),
  );

  render(TanstackAppTableTest, {
    query,
    emptyMessage: 'No rows',
    title: 'Test Table',
    head: 'Test head',
  });

  // letting the store update
  await new Promise((resolve) => setTimeout(resolve, 20));

  expect(screen.getByTestId('loadMoreButton')).not.toHaveAttribute('disabled');

  // loading more rows
  const loadMoreButton = screen.getByTestId('loadMoreButton');
  loadMoreButton.click();

  await new Promise((resolve) => setTimeout(resolve, 1));

  expect(screen.getByTestId('loadMoreButton')).toHaveAttribute('disabled');

  // letting the store update
  await new Promise((resolve) => setTimeout(resolve, 20));

  expect(screen.getByTestId('loadMoreButton')).not.toHaveAttribute('disabled');
});

test('load more buttton is disabled when there are no more pages', async () => {
  const query = createInfiniteQuery(
    {
      queryKey: [],
      queryFn: ({ pageParam }) => {
        return createMockQuery(1)(pageParam);
      },
      initialPageParam: 0,
      getNextPageParam(_lastPage, _allPages, lastPageParam) {
        if (lastPageParam > 0) {
          return undefined;
        }
        return lastPageParam + 1;
      },
    },
    new QueryClient({
      defaultOptions: {
        queries: {
          staleTime: Infinity,
        },
      },
    }),
  );

  render(TanstackAppTableTest, {
    query,
    emptyMessage: 'No rows',
    title: 'Test Table',
    head: 'Test head',
  });

  // letting the store update
  await new Promise((resolve) => setTimeout(resolve, 20));

  expect(screen.getByTestId('loadMoreButton')).not.toHaveAttribute('disabled');

  // loading more rows
  const loadMoreButton = screen.getByTestId('loadMoreButton');
  loadMoreButton.click();

  await new Promise((resolve) => setTimeout(resolve, 1));

  expect(screen.getByTestId('loadMoreButton')).toHaveAttribute('disabled');

  // letting the store update
  await new Promise((resolve) => setTimeout(resolve, 20));

  expect(screen.getByTestId('loadMoreButton')).toHaveAttribute('disabled');
});

test('refetches data when refresh button is clicked', async () => {
  // create a mock query that increments a number every time it is called
  const createMockQuery = () => {
    let count = 0;
    return async () => {
      const mockData = ['page' + count];
      count++;
      await new Promise((resolve) => setTimeout(resolve, 10));
      return mockData;
    };
  };
  const query = createInfiniteQuery(
    {
      queryKey: [],
      queryFn: createMockQuery(),
      initialPageParam: 0,
      getNextPageParam(_lastPage, _allPages, lastPageParam) {
        return lastPageParam + 1;
      },
    },
    new QueryClient({
      defaultOptions: {
        queries: {
          staleTime: Infinity,
        },
      },
    }),
  );

  render(TanstackAppTableTest, {
    query,
    emptyMessage: 'No rows',
    title: 'Test Table',
    head: 'Test head',
  });

  await new Promise((resolve) => setTimeout(resolve, 20));

  expect(screen.getByTestId('bodyRow')).toHaveTextContent('page0');

  // refreshing
  const refreshButton = screen.getByTestId('refreshButton');
  await userEvent.click(refreshButton);

  await new Promise((resolve) => setTimeout(resolve, 1));

  // refreshButton should have the class animate-spin
  expect(refreshButton).toHaveClass('animate-spin');

  // letting the store update
  await new Promise((resolve) => setTimeout(resolve, 20));

  // refreshButton should not have the class animate-spin
  expect(refreshButton).not.toHaveClass('animate-spin');

  expect(screen.getByTestId('bodyRow')).toHaveTextContent('page1');

  // refreshing
  await userEvent.click(refreshButton);

  // letting the store update
  await new Promise((resolve) => setTimeout(resolve, 20));

  expect(screen.getByTestId('bodyRow')).toHaveTextContent('page2');
});
