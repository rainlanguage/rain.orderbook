import { expect, vi, describe, it } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import ModalConnect from './ModalConnect.svelte';

const mockWalletConnectConnect = vi.hoisted(() =>
  vi.fn(async () => {
    return Promise.resolve();
  }),
);
const mockGetAddressFromLedger = vi.hoisted(() => vi.fn(async () => Promise.resolve()));

vi.mock('$lib/stores/walletconnect', async () => {
  const { writable } = await import('svelte/store');
  return {
    walletconnectAccount: writable(undefined),
    walletconnectIsDisconnecting: writable(false),
    walletconnectIsConnecting: writable(false),
    walletconnectProvider: writable(undefined),
    walletConnectNetwork: writable(1),
    walletconnectConnect: mockWalletConnectConnect,
    walletconnectDisconnect: vi.fn(),
  };
});

vi.mock('$lib/services/wallet', async () => {
  return {
    getAddressFromLedger: mockGetAddressFromLedger,
  };
});

vi.mock('$lib/stores/settings', async (importOriginal) => {
  return {
    ...((await importOriginal()) as object),
  };
});

describe('ModalConnect', () => {
  it('should reset after successful connection - walletconnect', async () => {
    render(ModalConnect);

    const button = screen.getByText('Connect to Wallet');
    fireEvent.click(button);

    const walletConnectButton = await screen.findByText('WalletConnect');
    expect(walletConnectButton).toBeInTheDocument();
    walletConnectButton.click();

    const connectButton = await screen.findByText('Connect');
    expect(connectButton).toBeInTheDocument();
    fireEvent.click(connectButton);

    expect(mockWalletConnectConnect).toHaveBeenCalled();

    expect(await screen.findByText('Connect to Wallet')).toBeInTheDocument();
  });

  it('should reset after successful connection - ledger', async () => {
    render(ModalConnect);

    const button = screen.getByText('Connect to Wallet');
    fireEvent.click(button);

    const ledgerButton = await screen.findByText('Ledger Wallet');
    expect(ledgerButton).toBeInTheDocument();
    ledgerButton.click();

    const connectButton = await screen.findByText('Connect');
    expect(connectButton).toBeInTheDocument();
    fireEvent.click(connectButton);

    expect(mockGetAddressFromLedger).toHaveBeenCalled();

    expect(await screen.findByText('Connect to Wallet')).toBeInTheDocument();
  });
});
