import { render, screen } from '@testing-library/svelte';
import { describe, it, expect, vi, type Mock } from 'vitest';
import Page from './+page.svelte';
import { readable } from 'svelte/store';
import { resetActiveOrderbookRef, resetActiveNetworkRef } from '$lib/stores/settings';

const { mockPageStore } = await vi.hoisted(
  () => import('../../lib/__mocks__/stores'),
);

vi.mock('$app/stores', async (importOriginal) => {
  const original = (await importOriginal()) as object;
  return {
    ...original,
    page: mockPageStore,
  };
});

vi.mock('../../lib/stores/settings', async (importOriginal) => {
  const actual = (await importOriginal()) as object;
  return {
    ...actual,
    activeOrderbook: readable(undefined),
    resetActiveOrderbookRef: vi.fn(),
    resetActiveNetworkRef: vi.fn(),
  };
});

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	const MockComponent = (await import('../../lib/__mocks__/MockComponent.svelte')).default;
	return {
        ...((await importOriginal()) as object),
        VaultsListTable: MockComponent,
        PageHeader: MockComponent
	};
});

describe('+page.svelte', () => {
  it('should reset active orderbook when no orderbook is found', () => {
    render(Page);
    expect(screen.getByText('Vaults')).toBeInTheDocument();
    expect(resetActiveOrderbookRef as Mock).toHaveBeenCalled();
    expect(resetActiveNetworkRef as Mock).toHaveBeenCalled();
  });
});
