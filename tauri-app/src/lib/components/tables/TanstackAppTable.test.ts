import { render, screen, waitFor } from '@testing-library/svelte';
import { test } from 'vitest';
import { expect } from '$lib/test/matchers';
import TanstackAppTableTest from './TanstackAppTable.test.svelte';
import { QueryClient, createInfiniteQuery } from '@tanstack/svelte-query';
import userEvent from '@testing-library/user-event';

// A helper function to create a resolvable mock query.
// This gives us more control over when each query resolves.
const createResolvableMockQuery = (queryFn: (pageParam: number) => unknown) => {
  const resolveQueue: Array<() => void> = [];
  let currentPromise: Promise<void>;

  const createNewPromise = () => {
    currentPromise = new Promise<void>((res) => {
      resolveQueue.push(res);
    });
  };

  createNewPromise(); // Initialize the first promise

  const resolvableQuery = async (pageParam: number) => {
    const mockData = queryFn(pageParam);
    await currentPromise;
    createNewPromise(); // Create a new promise for the next call
    return mockData;
  };

  const resolve = () => {
    const resolver = resolveQueue.shift();
    if (resolver) {
      resolver();
    }
  };

  return { queryFn: resolvableQuery, resolve };
};

// A helper function to create a Tanstack query that resolves when you call
// the `resolve` function.
const createResolvableInfiniteQuery = (
  _queryFn: (pageParam: number) => unknown,
  getNextPageParam: (
    _lastPage: unknown,
    _allPages: unknown[],
    lastPageParam: number,
  ) => number | undefined = (_lastPage: unknown, _allPages: unknown[], lastPageParam: number) =>
    lastPageParam + 1,
) => {
  const { queryFn, resolve } = createResolvableMockQuery(_queryFn);

  const query = createInfiniteQuery(
    {
      queryKey: [],
      queryFn: ({ pageParam }) => {
        return queryFn(pageParam);
      },
      initialPageParam: 0,
      getNextPageParam,
    },
    new QueryClient({
      defaultOptions: {
        queries: {
          staleTime: Infinity,
        },
      },
    }),
  );

  return { query, resolve };
};

test('shows head and title', async () => {
  const { query, resolve } = createResolvableInfiniteQuery((pageParam) => {
    return ['page' + pageParam];
  });

  render(TanstackAppTableTest, {
    query,
    emptyMessage: 'No rows',
    title: 'Test Table',
    head: 'Test head',
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

test('load more buttton is disabled when loading', async () => {
  const { query, resolve } = createResolvableInfiniteQuery((pageParam) => {
    return ['page' + pageParam];
  });

  render(TanstackAppTableTest, {
    query,
    emptyMessage: 'No rows',
    title: 'Test Table',
    head: 'Test head',
  });

  resolve();

  await waitFor(() => expect(screen.getByTestId('loadMoreButton')).not.toHaveAttribute('disabled'));

  // loading more rows
  const loadMoreButton = screen.getByTestId('loadMoreButton');
  loadMoreButton.click();

  await waitFor(() => expect(screen.getByTestId('loadMoreButton')).toHaveAttribute('disabled'));

  resolve();

  await waitFor(() => expect(screen.getByTestId('loadMoreButton')).not.toHaveAttribute('disabled'));
});

test('load more buttton is disabled when there are no more pages', async () => {
  const { query, resolve } = createResolvableInfiniteQuery(
    (pageParam) => {
      if (!pageParam) return ['page' + pageParam];
      return [];
    },
    (_lastPage, _allPages, lastPageParam) => {
      if (lastPageParam === 0) return 1;
      return undefined;
    },
  );

  render(TanstackAppTableTest, {
    query,
    emptyMessage: 'No rows',
    title: 'Test Table',
    head: 'Test head',
  });

  resolve();

  await waitFor(() => expect(screen.getByTestId('loadMoreButton')).not.toHaveAttribute('disabled'));

  // loading more rows
  const loadMoreButton = screen.getByTestId('loadMoreButton');
  loadMoreButton.click();

  await waitFor(() => expect(screen.getByTestId('loadMoreButton')).toHaveAttribute('disabled'));

  resolve();

  await waitFor(() =>
    expect(screen.getByTestId('loadMoreButton')).toHaveTextContent('Nothing more to load'),
  );
});

test('refetches data when refresh button is clicked', async () => {
  let refreshCount = 0;
  const { query, resolve } = createResolvableInfiniteQuery(() => {
    refreshCount++;
    return ['refresh' + refreshCount];
  });

  render(TanstackAppTableTest, {
    query,
    emptyMessage: 'No rows',
    title: 'Test Table',
    head: 'Test head',
  });

  resolve();

  await waitFor(() => expect(screen.getByTestId('bodyRow')).toHaveTextContent('refresh1'));

  // refreshing
  const refreshButton = screen.getByTestId('refreshButton');
  await userEvent.click(refreshButton);

  // refreshButton should have the class animate-spin
  await waitFor(() => expect(refreshButton).toHaveClass('animate-spin'));

  resolve();

  await waitFor(() => expect(screen.getByTestId('bodyRow')).toHaveTextContent('refresh2'));

  // refreshButton should not have the class animate-spin
  await waitFor(() => expect(refreshButton).not.toHaveClass('animate-spin'));

  // refreshing
  await userEvent.click(refreshButton);

  resolve();

  await waitFor(() => expect(screen.getByTestId('bodyRow')).toHaveTextContent('refresh3'));
});
