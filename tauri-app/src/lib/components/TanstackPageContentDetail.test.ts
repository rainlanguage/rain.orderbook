import { render, screen, waitFor } from '@testing-library/svelte';
import { test } from 'vitest';
import { expect } from '$lib/test/matchers';
import TanstackPageContentDetailTest from './TanstackPageContentDetail.test.svelte';
// import userEvent from '@testing-library/user-event';
import { createResolvableQuery } from '$lib/mocks/queries';

test('shows query data in correct places', async () => {
  const { query, resolve } = createResolvableQuery(() => {
    return 'test data';
  });

  render(TanstackPageContentDetailTest, {
    query,
    emptyMessage: 'No data',
    below: 'Below',
  });

  resolve();

  await waitFor(() => {
    expect(screen.getByTestId('top')).toHaveTextContent('test data');
    expect(screen.getByTestId('card')).toHaveTextContent('test data');
    expect(screen.getByTestId('chart')).toHaveTextContent('test data');
    expect(screen.getByTestId('below')).toHaveTextContent('Below');
  });
});

test('shows empty message', async () => {
  const { query, resolve } = createResolvableQuery(() => {
    return undefined;
  });

  render(TanstackPageContentDetailTest, {
    query,
    emptyMessage: 'No data',
    below: 'Below',
  });

  resolve();

  await waitFor(() => {
    expect(screen.getByTestId('emptyMessage')).toHaveTextContent('No data');
  });
});
