import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import {
  connected,
  wagmiLoaded,
  chainId,
  signerAddress,
  configuredConnectors,
  loading,
  defaultConfig,
  init,
  WC,
  disconnectWagmi
} from './wagmi';
import { createConfig, disconnect, getAccount, watchAccount } from '@wagmi/core';
import { mainnet } from '@wagmi/core/chains';

// Mock external dependencies
vi.mock('@wagmi/core', async (importOriginal) => ({
  ...(await importOriginal()),
  createConfig: vi.fn(),
  disconnect: vi.fn(),
  getAccount: vi.fn(),
  watchAccount: vi.fn(),
  reconnect: vi.fn(),
  http: vi.fn()

}));

vi.mock('@reown/appkit', () => ({
  createAppKit: vi.fn(() => ({
    open: vi.fn(),
    subscribeEvents: vi.fn()
  }))
}));

describe('wagmi store', () => {
  beforeEach(() => {
    // Reset all stores to initial state
    connected.set(false);
    wagmiLoaded.set(false);
    chainId.set(null);
    signerAddress.set(null);
    configuredConnectors.set([]);
    loading.set(true);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('defaultConfig', () => {
    it('should initialize with correct default values', () => {
      const mockConfig = { chains: [mainnet] };
      vi.mocked(createConfig).mockReturnValue(mockConfig);

      const result = defaultConfig({
        appName: 'Test App',
        projectId: 'test-project-id',
        connectors: []
      });

      expect(result).toHaveProperty('init');
      expect(get(wagmiLoaded)).toBe(true);
    });
  });

  describe('init', () => {
    it('should initialize wallet connection successfully', async () => {
      const mockAccount = {
        address: '0x123',
        chainId: 1,
        isConnected: true
      };

      vi.mocked(getAccount).mockReturnValue(mockAccount);
      vi.mocked(watchAccount).mockImplementation((_, { onChange }) => {
        onChange(mockAccount, mockAccount);
        return () => {};
      });

      await init();

      expect(get(connected)).toBe(true);
      expect(get(signerAddress)).toBe('0x123');
      expect(get(loading)).toBe(false);
    });

    it('should handle initialization failure', async () => {
      vi.mocked(getAccount).mockImplementation(() => {
        throw new Error('Connection failed');
      });

      await init();

      expect(get(connected)).toBe(false);
      expect(get(loading)).toBe(false);
    });
  });

  describe('disconnectWagmi', () => {
    it('should disconnect wallet and reset stores', async () => {
      connected.set(true);
      chainId.set(1);
      signerAddress.set('0x123');

      await disconnectWagmi();

      expect(vi.mocked(disconnect)).toHaveBeenCalled();
      expect(get(connected)).toBe(false);
      expect(get(chainId)).toBe(null);
      expect(get(signerAddress)).toBe(null);
      expect(get(loading)).toBe(false);
    });
  });
});