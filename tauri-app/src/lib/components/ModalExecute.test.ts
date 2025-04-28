import { expect, vi, describe, it, beforeEach } from 'vitest';
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

// Import components and stores after mocks
import ModalExecute from './ModalExecute.svelte';
import { settings } from '$lib/stores/settings';
import type { Config } from '@rainlanguage/orderbook';

describe('ModalExecute', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Reset settings store before each test
    settings.set({
      orderbook: {
        networks: {},
      },
    } as unknown as Config);
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
            rpc: 'https://test.com',
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
      settings.set({
        orderbook: {
          networks: {
            mainnet: {
              key: 'mainnet',
              chainId: 1,
              rpc: 'https://mainnet.com',
            },
          },
        },
      } as unknown as Config);

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
            rpc: 'https://test.com',
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
