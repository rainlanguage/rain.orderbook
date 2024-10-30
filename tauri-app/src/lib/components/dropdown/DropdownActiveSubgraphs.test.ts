import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { get, writable } from 'svelte/store';
import { activeSubgraphs } from '$lib/stores/settings';
import { expect, test, vi } from 'vitest';
import DropdownActiveSubgraphs from './DropdownActiveSubgraphs.svelte';

vi.mock('$lib/stores/settings', async (importOriginal) => {
  const { mockConfigSource } = await import('$lib/mocks/settings');
  return {
    ...((await importOriginal()) as object),
    settings: writable({
      ...mockConfigSource,
      subgraphs: {
        mainnet: 'mainnet',
        testnet: 'testnet',
        local: 'local',
      },
    }),
    activeSubgraphs: writable({}),
  };
});

test('renders correctly', () => {
  render(DropdownActiveSubgraphs);
  expect(screen.getByText('Networks')).toBeInTheDocument();
});

test('displays the correct number of options', async () => {
  render(DropdownActiveSubgraphs);

  await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));

  await waitFor(() => {
    const options = screen.getAllByTestId('dropdown-checkbox-option');
    expect(options).toHaveLength(3);
  });
});

test('updates active subgraphs when an option is selected', async () => {
  render(DropdownActiveSubgraphs);

  await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));
  await fireEvent.click(screen.getByText('mainnet'));
  await waitFor(() => {
    expect(get(activeSubgraphs)).toEqual({ mainnet: 'mainnet' });
  });

  await fireEvent.click(screen.getByText('testnet'));
  await waitFor(() => {
    expect(get(activeSubgraphs)).toEqual({ mainnet: 'mainnet', testnet: 'testnet' });
  });
});
