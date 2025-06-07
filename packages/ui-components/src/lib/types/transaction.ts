import type { Hex } from 'viem';
import type {
	SgVault,
	ApprovalCalldata,
	VaultCalldataResult,
	DepositAndAddOrderCalldataResult
} from '@rainlanguage/orderbook';
import type { Config } from '@wagmi/core';
import type { ToastLink } from './toast';
import type { AwaitSubgraphConfig } from '$lib/services/awaitTransactionIndexing';

export type ExtendedApprovalCalldata = ApprovalCalldata & { symbol?: string };

export type DeploymentArgs = {
	approvals: ExtendedApprovalCalldata[];
	deploymentCalldata: DepositAndAddOrderCalldataResult;
	orderbookAddress: Hex;
	chainId: number;
	subgraphUrl?: string;
	network: string;
};

export type VaultActionArgs = {
	vault: SgVault;
	chainId: number;
	onDeposit?: () => void;
	// rpcUrl is used to check balances
	rpcUrl: string;
	subgraphUrl: string;
	account: Hex;
};

export enum TransactionName {
	REMOVAL = 'Order Removal',
	WITHDRAWAL = 'Vault Withdrawal',
	APPROVAL = 'Token Approval',
	DEPOSIT = 'Vault Deposit'
}

export enum TransactionStatusMessage {
	IDLE = 'Idle',
	STARTING = 'Starting transaction...',
	CHECKING_ALLOWANCE = 'Checking your allowance...',
	PENDING_WALLET = 'Waiting for wallet confirmation...',
	PENDING_APPROVAL = 'Approving token spend...',
	PENDING_RECEIPT = 'Waiting for transaction receipt...',
	PENDING_DEPLOYMENT = 'Deploying your order...',
	PENDING_WITHDRAWAL = 'Withdrawing tokens...',
	PENDING_DEPOSIT = 'Depositing tokens...',
	PENDING_SUBGRAPH = 'Awaiting transaction indexing...',
	SUCCESS = 'Transaction confirmed',
	ERROR = 'Something went wrong'
}

export enum TransactionStoreErrorMessage {
	SWITCH_CHAIN_FAILED = 'Failed to switch chain.',
	SUBGRAPH_TIMEOUT_ERROR = 'The subgraph took too long to respond. Your transaction may still be processing.',
	SUBGRAPH_FAILED = 'Failed to index transaction.',
	RECEIPT_FAILED = 'Failed to get transaction receipt.'
}

export type DepositOrWithdrawTransactionArgs = {
	config: Config;
	approvalCalldata?: VaultCalldataResult;
	transactionCalldata: VaultCalldataResult;
	action: 'deposit' | 'withdraw';
	chainId: number;
	vault: SgVault;
	subgraphUrl: string;
};

export type InternalTransactionArgs = {
	chainId: number;
	txHash: Hex;
	networkKey: string;
	queryKey: string;
};

export type TransactionArgs = InternalTransactionArgs & {
	name: string;
	// Used for toast notifications upon final completion/failure
	errorMessage: string;
	successMessage: string;
	queryKey: string;
	toastLinks: ToastLink[];
	// Optional subgraphConfig for transactions that need to wait for indexing (e.g. deposit, but not approval)
	awaitSubgraphConfig?: AwaitSubgraphConfig;
};

export type DeploymentTransactionArgs = Omit<DeploymentArgs, 'account'> & {
	config: Config;
};
