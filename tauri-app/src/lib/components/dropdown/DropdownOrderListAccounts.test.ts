import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { get, writable } from 'svelte/store';
import DropdownOrderListAccounts from './DropdownOrderListAccounts.svelte';
import { activeAccountsItems } from '$lib/stores/settings';
import { expect, test, vi } from 'vitest';

vi.mock('$lib/stores/settings', async (importOriginal) => {
  const { mockSettingsStore } = await import('@rainlanguage/ui-components');
  return {
    ...((await importOriginal()) as object),
    settings: mockSettingsStore,
    accounts: writable({
      address1: 'Label 1',
      address2: 'Label 2',
      address3: 'Label 3',
    }),
    activeAccounts: writable({}),
    activeAccountItems: writable({}),
  };
});

test('renders correctly', () => {
  render(DropdownOrderListAccounts);
  expect(screen.getByText('Accounts')).toBeInTheDocument();
});

test('displays the correct number of options', async () => {
  render(DropdownOrderListAccounts);

  await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));

  await waitFor(() => {
    const options = screen.getAllByTestId('dropdown-checkbox-option');
    expect(options).toHaveLength(4);
  });
});

test('updates active accounts when an option is selected', async () => {
  render(DropdownOrderListAccounts);

  await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));
  await fireEvent.click(screen.getByText('Label 1'));

  await waitFor(() => {
    expect(get(activeAccountsItems)).toEqual({ address1: 'Label 1' });
  });
});

test('selects all items when "All accounts" is clicked', async () => {
  render(DropdownOrderListAccounts);

  await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));
  await fireEvent.click(screen.getByText('All accounts'));

  await waitFor(() => {
    expect(get(activeAccountsItems)).toEqual({
      address1: 'Label 1',
      address2: 'Label 2',
      address3: 'Label 3',
    });
  });
});

test('displays "No accounts added" when accounts list is empty', async () => {
  vi.doUnmock('$lib/stores/settings');
  vi.resetModules();

  const { default: DropdownOrderListAccounts } = await import('./DropdownOrderListAccounts.svelte');

  render(DropdownOrderListAccounts);

  await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));

  await waitFor(() => {
    expect(screen.getByText('No accounts added')).toBeInTheDocument();
  });
});
