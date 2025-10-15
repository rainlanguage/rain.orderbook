import { describe, it, expect, vi, beforeAll, beforeEach, afterEach } from 'vitest';
import { render, waitFor } from '@testing-library/svelte';
import { get } from 'svelte/store';
import * as registryContextModule from '$lib/providers/registry/context';
import type { RegistryContext } from '$lib/providers/registry/context';
import RegistryProviderHarness from './components/RegistryProviderHarness.svelte';

const pageUrlState = vi.hoisted(() => ({ current: new URL('http://localhost') }));

vi.mock('$app/stores', async () => {
  const { readable, writable } = await import('svelte/store');

  const createStores = () => ({
    navigating: readable(null),
    page: readable({
      url: pageUrlState.current,
      params: {},
      searchParams: pageUrlState.current.searchParams
    }),
    session: writable(null),
    updated: readable(false)
  });

  return {
    getStores: () => createStores(),
    navigating: {
      subscribe(fn: (value: null) => void) {
        return createStores().navigating.subscribe(fn);
      }
    },
    page: {
      subscribe(fn: (value: { url: URL; params: Record<string, string>; searchParams: URLSearchParams }) => void) {
        return createStores().page.subscribe(fn);
      }
    },
    session: {
      subscribe(fn: (value: unknown) => void) {
        return createStores().session.subscribe(fn);
      }
    },
    updated: {
      subscribe(fn: (value: boolean) => void) {
        return createStores().updated.subscribe(fn);
      }
    }
  };
});

// Mock SDK
vi.mock('@rainlanguage/orderbook', () => {
  return {
    DotrainRegistry: {
      new: vi.fn(async (url: string) => ({ value: { url } }))
    }
  };
});

const mockHistoryPush = vi.fn();
let actualSetRegistryContext: typeof registryContextModule.setRegistryContext;
let providedContext: RegistryContext | undefined;
let setRegistryContextSpy: ReturnType<typeof vi.spyOn>;

describe('RegistryProvider', () => {
  beforeAll(async () => {
    actualSetRegistryContext = (await vi.importActual<typeof import('$lib/providers/registry/context')>('$lib/providers/registry/context')).setRegistryContext;
  });

  beforeEach(() => {
    mockHistoryPush.mockReset();
    providedContext = undefined;
    setRegistryContextSpy = vi
      .spyOn(registryContextModule, 'setRegistryContext')
      .mockImplementation((context: RegistryContext) => {
        providedContext = context;
        return actualSetRegistryContext(context);
      });
    // JSDOM allows overriding pushState on history
    // @ts-ignore
    window.history.pushState = mockHistoryPush;
  });

  afterEach(() => {
    setRegistryContextSpy.mockRestore();
  });

  it('initializes from query param and exposes appendRegistryToHref', async () => {
    const url = new URL('http://localhost/deploy?registry=abc');
    pageUrlState.current = url;
    Object.defineProperty(window, 'location', { value: url, writable: true });

    render(RegistryProviderHarness, { props: { defaultUrl: undefined } });

    expect(providedContext).toBeDefined();

    await waitFor(() => {
      expect(get(providedContext!.registryUrl)).toBe('abc');
      expect(get(providedContext!.isCustomRegistry)).toBe(true);
    });

    expect(providedContext!.appendRegistryToHref('/deploy/test')).toBe('/deploy/test?registry=abc');
  });

  it('appendRegistryToHref leaves href unchanged when no registry set', async () => {
    const url = new URL('http://localhost/deploy');
    pageUrlState.current = url;
    Object.defineProperty(window, 'location', { value: url, writable: true });

    render(RegistryProviderHarness, { props: { defaultUrl: 'https://example.com/registry' } });

    expect(providedContext).toBeDefined();

    await waitFor(() => {
      expect(get(providedContext!.registryUrl)).toBe('');
      expect(get(providedContext!.isCustomRegistry)).toBe(false);
    });

    expect(providedContext!.appendRegistryToHref('/deploy/test')).toBe('/deploy/test');
  });

  it('setRegistryUrl updates registryUrl and history, and affects link builder', async () => {
    const url = new URL('http://localhost/deploy');
    pageUrlState.current = url;
    Object.defineProperty(window, 'location', { value: url, writable: true });

    render(RegistryProviderHarness, { props: { defaultUrl: 'https://example.com/registry' } });

    expect(providedContext).toBeDefined();

    providedContext!.setRegistryUrl('custom');

    await waitFor(() => {
      expect(get(providedContext!.registryUrl)).toBe('custom');
      expect(get(providedContext!.isCustomRegistry)).toBe(true);
      expect(mockHistoryPush).toHaveBeenCalled();
    });

    expect(providedContext!.appendRegistryToHref('/deploy/test')).toBe('/deploy/test?registry=custom');
  });

  it('appendRegistryToHref preserves existing query parameters', async () => {
    const url = new URL('http://localhost/deploy?registry=abc');
    pageUrlState.current = url;
    Object.defineProperty(window, 'location', { value: url, writable: true });

    render(RegistryProviderHarness);

    expect(providedContext).toBeDefined();

    await waitFor(() => {
      expect(get(providedContext!.registryUrl)).toBe('abc');
    });

    expect(providedContext!.appendRegistryToHref('/deploy/test?foo=bar')).toBe('/deploy/test?foo=bar&registry=abc');
  });
});
