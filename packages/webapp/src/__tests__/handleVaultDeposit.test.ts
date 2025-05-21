import { handleVaultDeposit } from '../lib/services/handleVaultDeposit';
import { getVaultApprovalCalldata, getVaultDepositCalldata } from '@rainlanguage/orderbook';
import type { TransactionManager } from '@rainlanguage/ui-components';
import type { Hex } from 'viem';
import { vi, describe, it, expect, beforeEach } from 'vitest';

vi.mock('@rainlanguage/orderbook', () => ({
  getVaultApprovalCalldata: vi.fn(),
  getVaultDepositCalldata: vi.fn()
}));

describe('handleVaultDeposit', () => {
  const mockRpcUrl = 'https://mock-rpc.url';
  const mockSubgraphUrl = 'https://mock-subgraph.url';
  const mockNetworkKey = 'mock-network';
  const mockChainId = 1;
  const mockVault = { id: 'mock-vault-id' } as any;
  const mockAmount = BigInt(1000);
  const mockCreateApprovalTransaction = vi.fn();
  const mockCreateDepositTransaction = vi.fn();
  const mockManager = {
    createApprovalTransaction: mockCreateApprovalTransaction,
    createDepositTransaction: mockCreateDepositTransaction
  } as unknown as TransactionManager;
  const mockErrToast = vi.fn();
  const mockTxHash = '0x123' as Hex;

  beforeEach(() => {
    vi.resetAllMocks();
  });

  it('should return approval transaction data when approval is required', async () => {
    vi.mocked(getVaultApprovalCalldata).mockResolvedValue({
      value: 'approval-calldata',
      error: undefined
    });

    const result = await handleVaultDeposit(
      mockRpcUrl,
      mockSubgraphUrl,
      mockNetworkKey,
      mockChainId,
      mockVault,
      mockAmount,
      mockManager,
      mockErrToast
    );

    expect(result).toBeTruthy();
    expect(result?.calldata).toBe('approval-calldata');
    
    result?.onConfirm(mockTxHash);
    expect(mockCreateApprovalTransaction).toHaveBeenCalledWith({
      subgraphUrl: mockSubgraphUrl,
      txHash: mockTxHash,
      chainId: mockChainId,
      networkKey: mockNetworkKey,
      queryKey: mockVault.id,
      entity: mockVault
    });
    
    expect(mockCreateDepositTransaction).not.toHaveBeenCalled();
    expect(mockErrToast).not.toHaveBeenCalled();
  });

  it('should return deposit transaction data when approval is not required', async () => {
    vi.mocked(getVaultApprovalCalldata).mockResolvedValue({
      value: undefined,
      error: { msg: 'Already approved', readableMsg: 'Already approved' }
    });
    
    vi.mocked(getVaultDepositCalldata).mockResolvedValue({
      value: 'deposit-calldata',
      error: undefined
    });

    const result = await handleVaultDeposit(
      mockRpcUrl,
      mockSubgraphUrl,
      mockNetworkKey,
      mockChainId,
      mockVault,
      mockAmount,
      mockManager,
      mockErrToast
    );

    expect(result).toBeTruthy();
    expect(result?.calldata).toBe('deposit-calldata');
    
    result?.onConfirm(mockTxHash);
    expect(mockCreateDepositTransaction).toHaveBeenCalledWith({
      subgraphUrl: mockSubgraphUrl,
      txHash: mockTxHash,
      chainId: mockChainId,
      networkKey: mockNetworkKey,
      queryKey: mockVault.id,
      entity: mockVault
    });
    
    expect(mockCreateApprovalTransaction).not.toHaveBeenCalled();
    expect(mockErrToast).not.toHaveBeenCalled();
  });

  it('should show error toast and return null when deposit fails', async () => {
    vi.mocked(getVaultApprovalCalldata).mockResolvedValue({
      value: undefined,
      error: { msg: 'Already approved', readableMsg: 'Already approved' }
    });
    
    const errorMsg = 'Deposit error occurred';
    vi.mocked(getVaultDepositCalldata).mockResolvedValue({
      value: undefined,
      error: { msg: errorMsg, readableMsg: errorMsg }
    });

    const result = await handleVaultDeposit(
      mockRpcUrl,
      mockSubgraphUrl,
      mockNetworkKey,
      mockChainId,
      mockVault,
      mockAmount,
      mockManager,
      mockErrToast
    );

    expect(result).toBeNull();
    expect(mockErrToast).toHaveBeenCalledWith(errorMsg);
    expect(mockCreateApprovalTransaction).not.toHaveBeenCalled();
    expect(mockCreateDepositTransaction).not.toHaveBeenCalled();
  });

  it('should return null when deposit calldata is missing', async () => {
    vi.mocked(getVaultApprovalCalldata).mockResolvedValue({
      value: undefined,
      error: { msg: 'Already approved', readableMsg: 'Already approved' }
    });
    
    vi.mocked(getVaultDepositCalldata).mockResolvedValue({
      value: undefined,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      error: undefined as any
    });

    const result = await handleVaultDeposit(
      mockRpcUrl,
      mockSubgraphUrl,
      mockNetworkKey,
      mockChainId,
      mockVault,
      mockAmount,
      mockManager,
      mockErrToast
    );

    // Verify null was returned
    expect(result).toBeNull();
    expect(mockErrToast).not.toHaveBeenCalled();
    expect(mockCreateApprovalTransaction).not.toHaveBeenCalled();
    expect(mockCreateDepositTransaction).not.toHaveBeenCalled();
  });
}); 