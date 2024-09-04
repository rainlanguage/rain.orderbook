import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { get, writable } from 'svelte/store';
import DropdownOrderListWatchlist from './DropdownOrderListWatchlist.svelte';
import { activeWatchlistItems } from '$lib/stores/settings';
import { expect, test, vi } from 'vitest';

vi.mock('$lib/stores/settings', async (importOriginal) => {
  const { mockSettingsStore } = await import('$lib/mocks/settings');
  return {
    ...((await importOriginal()) as object),
    settings: mockSettingsStore,
    watchlist: writable(['address1', 'address2', 'address3']),
    activeWatchlist: writable([]),
    activeWatchlistItems: writable([]),
  };
});

test('renders correctly', () => {
  render(DropdownOrderListWatchlist);
  expect(screen.getByText('Watchlist')).toBeInTheDocument();
});

test('displays the correct number of options', async () => {
  render(DropdownOrderListWatchlist);

  await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));

  await waitFor(() => {
    const options = screen.getAllByTestId('dropdown-checkbox-option');
    expect(options).toHaveLength(4);
  });
});

test('updates active watchlist when an option is selected', async () => {
  render(DropdownOrderListWatchlist);

  await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));
  await fireEvent.click(screen.getByLabelText('address1'));

  await waitFor(() => {
    expect(get(activeWatchlistItems)).toEqual(['address1']);
  });
});

test('selects all items when "All addresses" is clicked', async () => {
  render(DropdownOrderListWatchlist);

  await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));
  await fireEvent.click(screen.getByText('All addresses'));

  await waitFor(() => {
    expect(get(activeWatchlistItems)).toEqual(['address1', 'address2', 'address3']);
  });
});

test('displays "No watchlist added" when watchlist is empty', async () => {
  vi.doUnmock('$lib/stores/settings');
  vi.resetModules();

  const { default: DropdownOrderListWatchlist } = await import(
    './DropdownOrderListWatchlist.svelte'
  );
  render(DropdownOrderListWatchlist);

  await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));

  await waitFor(() => {
    expect(screen.getByText('No watchlist added')).toBeInTheDocument();
  });
});
