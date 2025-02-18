import { writable } from 'svelte/store';
import type { Hex } from 'viem';
import type { Config } from '@wagmi/core';
import { sendTransaction, switchChain, waitForTransactionReceipt } from '@wagmi/core';
import type {
	ApprovalCalldata,
	DepositAndAddOrderCalldataResult,
	DepositCalldataResult,
	Transaction,
	RemoveOrderCalldata,
	Vault,
	WithdrawCalldataResult
} from '@rainlanguage/orderbook/js_api';
import { getTransaction, getTransactionAddOrders, getTransactionRemoveOrders } from '@rainlanguage/orderbook/js_api';

export const ADDRESS_ZERO = '0x0000000000000000000000000000000000000000';
export const ONE = BigInt('1000000000000000000');

export enum TransactionStatus {
	IDLE = 'Idle',
	CHECKING_ALLOWANCE = 'Checking your allowance...',
	PENDING_WALLET = 'Waiting for wallet confirmation...',
	PENDING_APPROVAL = 'Approving token spend...',
	PENDING_DEPLOYMENT = 'Deploying your strategy...',
	PENDING_SUBGRAPH = 'Awaiting subgraph...',
	SUCCESS = 'Success! Transaction confirmed',
	ERROR = 'Something went wrong'
}

export enum TransactionErrorMessage {
	BAD_CALLLDATA = 'Bad calldata.',
	DEPLOY_FAILED = 'Lock transaction failed.',
	TIMEOUT = 'Transaction timed out.',
	APPROVAL_FAILED = 'Approval transaction failed.',
	USER_REJECTED_APPROVAL = 'User rejected approval transaction.',
	USER_REJECTED_TRANSACTION = 'User rejected the transaction.',
	DEPLOYMENT_FAILED = 'Deployment transaction failed.',
	SWITCH_CHAIN_FAILED = 'Failed to switch chain.',
	DEPOSIT_FAILED = 'Failed to deposit tokens.',
	WITHDRAWAL_FAILED = 'Failed to withdraw tokens.',
	REMOVE_ORDER_FAILED = 'Failed to remove order.'
}

export type ExtendedApprovalCalldata = ApprovalCalldata & { symbol?: string };

export type DeploymentTransactionArgs = {
	config: Config;
	approvals: ExtendedApprovalCalldata[];
	deploymentCalldata: DepositAndAddOrderCalldataResult;
	orderbookAddress: Hex;
	chainId: number;
	subgraphUrl: string;
	network: string;
};

export type DepositOrWithdrawTransactionArgs = {
	config: Config;
	approvalCalldata?: ApprovalCalldata;
	transactionCalldata: DepositCalldataResult | WithdrawCalldataResult;
	action: 'deposit' | 'withdraw';
	chainId: number;
	vault: Vault;
	subgraphUrl: string;
};

export type RemoveOrderTransactionArgs = {
	config: Config;
	orderbookAddress: Hex;
	removeOrderCalldata: RemoveOrderCalldata;
	chainId: number;
	subgraphUrl: string;
	network: string;
};

export type TransactionState = {
	status: TransactionStatus;
	error: string;
	hash: string;
	data: null;
	functionName: string;
	message: string;
	newOrderId: string;
	network: string;
};

export type TransactionStore = {
	subscribe: (run: (value: TransactionState) => void) => () => void;
	reset: () => void;
	handleDeploymentTransaction: (args: DeploymentTransactionArgs) => Promise<void>;
	handleDepositOrWithdrawTransaction: (args: DepositOrWithdrawTransactionArgs) => Promise<void>;
	handleRemoveOrderTransaction: (args: RemoveOrderTransactionArgs) => Promise<void>;
	checkingWalletAllowance: (message?: string) => void;
	awaitWalletConfirmation: (message?: string) => void;
	awaitApprovalTx: (hash: string) => void;
	transactionSuccess: (hash: string, message?: string) => void;
	transactionError: (message: TransactionErrorMessage, hash?: string) => void;
};

const initialState: TransactionState = {
	status: TransactionStatus.IDLE,
	error: '',
	hash: '',
	data: null,
	functionName: '',
	message: '',
	newOrderId: '',
	network: ''
};

const transactionStore = () => {
	const { subscribe, set, update } = writable(initialState);
	const reset = () => set(initialState);

	const awaitTransactionIndexing = async (
		subgraphUrl: string,
		txHash: string,
		successMessage: string
	) => {
		update((state) => ({
			...state,
			status: TransactionStatus.PENDING_SUBGRAPH,
			message: 'Checking for transaction indexing...'
		}));

		let attempts = 0;
		let newTx: Transaction;

		const interval: NodeJS.Timeout = setInterval(async () => {
			attempts++;

			newTx = await getTransaction(subgraphUrl, txHash);
			if (newTx) {
				clearInterval(interval);
				transactionSuccess(txHash, successMessage);
			} else if (attempts >= 10) {
				update((state) => ({
					...state,
					message: 'The subgraph took too long to respond. Please check again later.'
				}));
				clearInterval(interval);
				return transactionError(TransactionErrorMessage.TIMEOUT);
			}
		}, 1000);
	};

	const awaitNewOrderIndexing = async (subgraphUrl: string, txHash: string, network?: string) => {
		update((state) => ({
			...state,
			status: TransactionStatus.PENDING_SUBGRAPH,
			message: 'Waiting for new Order to be indexed...'
		}));

		let attempts = 0;
		const interval: NodeJS.Timeout = setInterval(async () => {
			attempts++;
			const addOrders = await getTransactionAddOrders(subgraphUrl, txHash);
			if (attempts >= 10) {
				update((state) => ({
					...state,
					message: 'The subgraph took too long to respond. Please check again later.'
				}));
				clearInterval(interval);
				return transactionError(TransactionErrorMessage.TIMEOUT);
			} else if (addOrders?.length > 0) {
				clearInterval(interval);
				return transactionSuccess(txHash, '', addOrders[0].order.id, network);
			}
		}, 1000);
	};

	const awaitRemoveOrderIndexing = async (subgraphUrl: string, txHash: string, network?: string) => {
		update((state) => ({
			...state,
			status: TransactionStatus.PENDING_SUBGRAPH,
			message: 'Waiting for order removal to be indexed...'
		}));

		let attempts = 0;
		const interval: NodeJS.Timeout = setInterval(async () => {
			attempts++;
			const removeOrders = await getTransactionRemoveOrders(subgraphUrl, txHash);
			if (attempts >= 10) {
				update((state) => ({
					...state,
					message: 'The subgraph took too long to respond. Please check again later.'
				}));
				clearInterval(interval);
				return transactionError(TransactionErrorMessage.TIMEOUT);
			} else if (removeOrders?.length > 0) {
				clearInterval(interval);
				return transactionSuccess(txHash);
			}
		}, 1000);
	};

	const checkingWalletAllowance = (message?: string) =>
		update((state) => ({
			...state,
			status: TransactionStatus.CHECKING_ALLOWANCE,
			message: message || ''
		}));
	const awaitWalletConfirmation = (message?: string) =>
		update((state) => ({
			...state,
			status: TransactionStatus.PENDING_WALLET,
			message: message || ''
		}));
	const awaitApprovalTx = (hash: string, symbol: string | undefined) =>
		update((state) => ({
			...state,
			hash: hash,
			status: TransactionStatus.PENDING_APPROVAL,
			message: `Approving ${symbol || 'token'} spend...`
		}));
	const awaitDeployTx = (hash: string) =>
		update((state) => ({
			...state,
			hash: hash,
			status: TransactionStatus.PENDING_DEPLOYMENT,
			message: 'Confirming transaction...'
		}));
	const transactionSuccess = (
		hash: string,
		message?: string,
		newOrderId?: string,
		network?: string
	) => {
		update((state) => ({
			...state,
			status: TransactionStatus.SUCCESS,
			hash: hash,
			message: message || '',
			newOrderId: newOrderId || '',
			network: network || ''
		}));
	};
	const transactionError = (message: TransactionErrorMessage, hash?: string) =>
		update((state) => ({
			...state,
			status: TransactionStatus.ERROR,
			error: message,
			hash: hash || ''
		}));

	const handleDeploymentTransaction = async ({
		config,
		approvals,
		deploymentCalldata,
		orderbookAddress,
		chainId,
		subgraphUrl,
		network
	}: DeploymentTransactionArgs) => {
		try {
			await switchChain(config, { chainId });
		} catch {
			return transactionError(TransactionErrorMessage.SWITCH_CHAIN_FAILED);
		}
		for (const approval of approvals) {
			let approvalHash: Hex;
			try {
				awaitWalletConfirmation(
					`Please approve ${approval.symbol || approval.token} spend in your wallet...`
				);
				approvalHash = await sendTransaction(config, {
					to: approval.token as `0x${string}`,
					data: approval.calldata as `0x${string}`
				});
			} catch {
				return transactionError(TransactionErrorMessage.USER_REJECTED_APPROVAL);
			}
			try {
				awaitApprovalTx(approvalHash, approval.symbol);
				await waitForTransactionReceipt(config, { hash: approvalHash });
			} catch {
				return transactionError(TransactionErrorMessage.APPROVAL_FAILED);
			}
		}

		let hash: Hex;
		try {
			awaitWalletConfirmation('Please confirm deployment in your wallet...');
			hash = await sendTransaction(config, {
				to: orderbookAddress as `0x${string}`,
				data: deploymentCalldata as `0x${string}`
			});
		} catch {
			return transactionError(TransactionErrorMessage.USER_REJECTED_TRANSACTION);
		}
		try {
			awaitDeployTx(hash);
			await waitForTransactionReceipt(config, { hash });
			return awaitNewOrderIndexing(subgraphUrl, hash, network);
		} catch {
			return transactionError(TransactionErrorMessage.DEPLOYMENT_FAILED);
		}
	};

	const handleDepositOrWithdrawTransaction = async ({
		config,
		approvalCalldata,
		transactionCalldata,
		action,
		chainId,
		vault,
		subgraphUrl
	}: DepositOrWithdrawTransactionArgs) => {
		try {
			await switchChain(config, { chainId });
		} catch {
			return transactionError(TransactionErrorMessage.SWITCH_CHAIN_FAILED);
		}
		if (approvalCalldata) {
			let approvalHash: Hex;
			try {
				awaitWalletConfirmation(`Please approve ${vault.token.symbol} spend in your wallet...`);
				approvalHash = await sendTransaction(config, {
					to: vault.token.address as `0x${string}`,
					data: approvalCalldata as unknown as `0x${string}`
				});
			} catch {
				return transactionError(TransactionErrorMessage.USER_REJECTED_APPROVAL);
			}
			try {
				awaitApprovalTx(approvalHash, vault.token.symbol);
				await waitForTransactionReceipt(config, { hash: approvalHash });
			} catch {
				return transactionError(TransactionErrorMessage.APPROVAL_FAILED);
			}
		}
		let hash: Hex;
		try {
			awaitWalletConfirmation(
				`Please confirm ${action === 'deposit' ? 'deposit' : 'withdrawal'} in your wallet...`
			);
			hash = await sendTransaction(config, {
				to: vault.orderbook.id as `0x${string}`,
				data: transactionCalldata as unknown as `0x${string}`
			});
		} catch {
			return transactionError(TransactionErrorMessage.USER_REJECTED_TRANSACTION);
		}
		try {
			awaitDeployTx(hash);
			await waitForTransactionReceipt(config, { hash });
			return awaitTransactionIndexing(
				subgraphUrl,
				hash,
				`The ${action === 'deposit' ? 'deposit' : 'withdrawal'} was successful.`
			);
		} catch {
			return transactionError(
				action === 'deposit'
					? TransactionErrorMessage.DEPOSIT_FAILED
					: TransactionErrorMessage.WITHDRAWAL_FAILED
			);
		}
	};

	const handleRemoveOrderTransaction = async ({
		config,
		orderbookAddress,
		removeOrderCalldata,
		chainId,
		subgraphUrl,
		network
	}: RemoveOrderTransactionArgs) => {
		try {
			await switchChain(config, { chainId });
		} catch {
			return transactionError(TransactionErrorMessage.SWITCH_CHAIN_FAILED);
		}

		let hash: Hex;
		try {
			awaitWalletConfirmation('Please confirm order removal in your wallet...');
			hash = await sendTransaction(config, {
				to: orderbookAddress,
				data: removeOrderCalldata as `0x${string}`
			});
		} catch {
			return transactionError(TransactionErrorMessage.USER_REJECTED_TRANSACTION);
		}

		try {
			awaitDeployTx(hash);
			await waitForTransactionReceipt(config, { hash });
			return awaitRemoveOrderIndexing(subgraphUrl, hash, network);
		} catch {
			return transactionError(TransactionErrorMessage.REMOVE_ORDER_FAILED);
		}
	};

	return {
		subscribe,
		reset,
		handleDeploymentTransaction,
		handleDepositOrWithdrawTransaction,
		handleRemoveOrderTransaction,
		checkingWalletAllowance,
		awaitWalletConfirmation,
		awaitApprovalTx,
		transactionSuccess,
		transactionError,
		awaitTransactionIndexing,
		awaitNewOrderIndexing,
		awaitRemoveOrderIndexing
	};
};

export default transactionStore();
