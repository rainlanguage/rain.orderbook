import { render } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import Layout from './+layout.svelte';

const { mockPageStore, initialPageState } = await vi.hoisted(
  () => import('$lib/__mocks__/stores')
);



// Create a mock ercKit object with an init method - HOISTED
const mockErcKit = vi.hoisted(() => ({
  init: vi.fn().mockResolvedValue(undefined)
}));

// Hoist the mock function declaration
const mockDefaultConfig = vi.hoisted(() => vi.fn().mockReturnValue(mockErcKit));


vi.mock('$app/stores', async (importOriginal) => {
  return {
    ...((await importOriginal()) as object),
    page: mockPageStore
  };
});

// Mock environment
vi.mock('$app/environment', () => ({
  browser: true
}));

vi.mock('../lib/components/Sidebar.svelte', async () => {
	const MockSidebar = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: MockSidebar };
});

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {

	const MockWalletProvider = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { 
        ...(await importOriginal()),
        
        WalletProvider: MockWalletProvider };
});

// Mock wagmi imports
vi.mock('$lib/stores/wagmi', () => ({
  defaultConfig: mockDefaultConfig,
  signerAddress: { subscribe: vi.fn() }
}));

vi.mock('$env/static/public', () => ({
  PUBLIC_WALLETCONNECT_PROJECT_ID: 'test-project-id'
}));

vi.mock('@wagmi/connectors', async (importOriginal) => 
    {
        return {
		...(await importOriginal()),

            injected: vi.fn().mockReturnValue('injected-connector'),
            walletConnect: vi.fn().mockReturnValue('wallet-connect-connector')
        }   
});

describe('Layout component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.resetAllMocks();
        mockDefaultConfig.mockReturnValue(mockErcKit);
  });

  it('initializes wallet when in browser environment', async () => {
    Object.defineProperty(global, 'navigator', {
      value: {},
      writable: true
    });

    mockPageStore.mockSetSubscribeValue(initialPageState);

    render(Layout);

    expect(mockErcKit.init).toHaveBeenCalled();
  });
  it('renders Homepage when on root path', async () => {
  mockPageStore.mockSetSubscribeValue({
    ...initialPageState,
    url: new URL('http://localhost/')
  });

  const { container } = render(Layout);
  
  expect(container.querySelector('main')).not.toBeInTheDocument();
});


it('renders Sidebar and main content when not on root path', async () => {

  mockPageStore.mockSetSubscribeValue({
    ...initialPageState,
    url: new URL('http://localhost/some-page')
  });

  const { container } = render(Layout);
  
  const main = container.querySelector('main');
  console.log("main",main)
  expect(container.querySelector('main')).toBeInTheDocument();
});

});