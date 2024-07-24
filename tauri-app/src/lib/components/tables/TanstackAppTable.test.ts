import { render, screen, waitFor } from '@testing-library/svelte';
import { test } from 'vitest';
import { expect } from '$lib/test/matchers';
import TanstackAppTableTest from './TanstackAppTable.test.svelte';
import userEvent from '@testing-library/user-event';
import { createResolvableInfiniteQuery } from '$lib/mocks/queries';

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
