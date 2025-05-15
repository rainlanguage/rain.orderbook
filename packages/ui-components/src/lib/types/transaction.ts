import type { Hex } from 'viem';
import type {
	SgOrder,
	SgVault,
	WithdrawCalldataResult,
	ApprovalCalldata,
	DepositCalldataResult,
	DepositAndAddOrderCalldataResult
} from '@rainlanguage/orderbook';
import type { Config } from '@wagmi/core';
import type { ToastLink } from './toast';

export type DeploymentArgs = {
	approvals: ExtendedApprovalCalldata[];
	deploymentCalldata: DepositAndAddOrderCalldataResult;
	orderbookAddress: Hex;
	chainId: number;
	subgraphUrl?: string;
	network: string;
};

export type DepositOrWithdrawArgs = {
	vault: SgVault;
	onDepositOrWithdraw: () => void;
	action: 'deposit' | 'withdraw';
	chainId: number;
	rpcUrl: string;
	subgraphUrl: string;
	account: Hex;
};

export type OrderRemoveArgs = {
	order: SgOrder;
	onRemove: () => void;
	chainId: number;
	orderbookAddress: Hex;
	subgraphUrl: string;
};

export enum TransactionStatusMessage {
	IDLE = 'Idle',
	CHECKING_ALLOWANCE = 'Checking your allowance...',
	PENDING_WALLET = 'Waiting for wallet confirmation...',
	PENDING_APPROVAL = 'Approving token spend...',
	PENDING_DEPLOYMENT = 'Deploying your order...',
	PENDING_WITHDRAWAL = 'Withdrawing tokens...',
	PENDING_DEPOSIT = 'Depositing tokens...',
	PENDING_REMOVE_ORDER = 'Removing order...',
	PENDING_SUBGRAPH = 'Awaiting subgraph...',
	SUCCESS = 'Success! Transaction confirmed',
	ERROR = 'Something went wrong'
}

export enum TransactionErrorMessage {
	BAD_CALLLDATA = 'Bad calldata.',
	DEPLOY_FAILED = 'Lock transaction failed.',
	TIMEOUT = 'The subgraph took too long to respond.',
	APPROVAL_FAILED = 'Approval transaction failed.',
	USER_REJECTED_APPROVAL = 'User rejected approval transaction.',
	USER_REJECTED_TRANSACTION = 'User rejected the transaction.',
	DEPLOYMENT_FAILED = 'Deployment transaction failed.',
	SWITCH_CHAIN_FAILED = 'Failed to switch chain.',
	DEPOSIT_FAILED = 'Failed to deposit tokens.',
	WITHDRAWAL_FAILED = 'Failed to withdraw tokens.',
	REMOVE_ORDER_FAILED = 'Failed to remove order.',
	SUBGRAPH_FAILED = 'Failed to index order removal.'
}

export type DepositOrWithdrawTransactionArgs = {
	config: Config;
	approvalCalldata?: ApprovalCalldata;
	transactionCalldata: DepositCalldataResult | WithdrawCalldataResult;
	action: 'deposit' | 'withdraw';
	chainId: number;
	vault: SgVault;
	subgraphUrl: string;
};

export type InternalTransactionArgs = {
	orderHash: string;
	chainId: number;
	subgraphUrl: string;
	txHash: Hex;
};

export type TransactionArgs = InternalTransactionArgs & {
	errorMessage: string;
	successMessage: string;
	fetchEntityFn: () => Promise<void>;
	queryKey: string;
	toastLinks: ToastLink[];
};

export type TransactionState = {
	status: TransactionStatusMessage;
	error: string;
	hash: string;
	data: null;
	functionName: string;
	message: string;
	newOrderHash: string;
	network: string;
	explorerLink: string;
};

export type ExtendedApprovalCalldata = ApprovalCalldata & { symbol?: string };

export type DeploymentArgsWithoutAccount = Omit<DeploymentArgs, 'account'>;

export type DeploymentTransactionArgs = DeploymentArgsWithoutAccount & {
	config: Config;
};
