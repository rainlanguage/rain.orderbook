import { expect, vi, describe, it, beforeEach, type Mock } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { writable } from 'svelte/store';
import type { Hex } from 'viem';

// Move imports that are mocked to after the vi.mock declarations
vi.mock('$lib/stores/walletconnect', () => ({
  walletconnectAccount: writable('0x123' as Hex),
  walletconnectIsDisconnecting: writable(false),
  walletconnectIsConnecting: writable(false),
  walletconnectProvider: writable(undefined),
  walletConnectNetwork: writable(1),
  walletConnectConnect: vi.fn(),
  walletconnectDisconnect: vi.fn(),
}));

vi.mock('@walletconnect/modal', () => ({
  WalletConnectModal: vi.fn(),
}));

vi.mock('$lib/stores/settings', async (importOriginal) => ({
  ...((await importOriginal()) as object),
}));

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
  const original = (await importOriginal()) as object;
  return {
    ...original,
    getNetworkName: vi.fn(),
    useRaindexClient: vi.fn(() => ({
      getNetworkByChainId: vi.fn().mockReturnValue({ value: {} as NetworkCfg }),
      getAllNetworks: vi.fn().mockReturnValue({ value: new Map() }),
    })),
  };
});

// Import components and stores after mocks
import ModalExecute from './ModalExecute.svelte';
import { getNetworkName } from '@rainlanguage/ui-components';
import type { NetworkCfg } from '@rainlanguage/orderbook';

describe('ModalExecute', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('network connection error', () => {
    it('should show unknown network name when network is not in settings', () => {
      render(ModalExecute, {
        props: {
          open: true,
          title: 'Test',
          execButtonLabel: 'Execute',
          executeLedger: vi.fn(),
          executeWalletconnect: vi.fn(),
          isSubmitting: false,
          overrideNetwork: {
            key: 'test',
            rpcs: ['https://test.com'],
            chainId: 2,
          },
        },
      });

      const errorElement = screen.getByTestId('network-connection-error');
      expect(errorElement).toHaveTextContent(
        'You are connected to an unknown network. Please connect your wallet to test network.',
      );
    });

    it('should show current connected network name when network is in settings', () => {
      (getNetworkName as Mock).mockReturnValue('mainnet');

      render(ModalExecute, {
        props: {
          open: true,
          title: 'Test',
          execButtonLabel: 'Execute',
          executeLedger: vi.fn(),
          executeWalletconnect: vi.fn(),
          isSubmitting: false,
          overrideNetwork: {
            key: 'test',
            rpcs: ['https://test.com'],
            chainId: 2,
          },
        },
      });

      const errorElement = screen.getByTestId('network-connection-error');
      expect(errorElement).toHaveTextContent(
        'You are connected to mainnet network. Please connect your wallet to test network.',
      );
    });
  });
});
