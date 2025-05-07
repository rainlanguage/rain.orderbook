import { writable } from 'svelte/store';
import type { Hex } from 'viem';
import type { Config } from '@wagmi/core';
import { sendTransaction, switchChain, waitForTransactionReceipt } from '@wagmi/core';
import type {
	ApprovalCalldata,
	DepositCalldataResult,
	RemoveOrderCalldata,
	SgVault,
	WithdrawCalldataResult
} from '@rainlanguage/orderbook';

import { getExplorerLink } from '../services/getExplorerLink';
import type { DeploymentArgs } from '$lib/types/transaction';
import {
	awaitSubgraphIndexing,
	getNewOrderConfig,
	getRemoveOrderConfig,
	getTransactionConfig
} from '$lib/services/awaitTransactionIndexing';

export const ADDRESS_ZERO = '0x0000000000000000000000000000000000000000';
export const ONE = BigInt('1000000000000000000');

export enum TransactionStatus {
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
	REMOVE_ORDER_FAILED = 'Failed to remove order.'
}

export type ExtendedApprovalCalldata = ApprovalCalldata & { symbol?: string };

export type DeploymentArgsWithoutAccount = Omit<DeploymentArgs, 'account'>;
export type DeploymentTransactionArgs = DeploymentArgsWithoutAccount & {
	config: Config;
};

export type DepositOrWithdrawTransactionArgs = {
	config: Config;
	approvalCalldata?: ApprovalCalldata;
	transactionCalldata: DepositCalldataResult | WithdrawCalldataResult;
	action: 'deposit' | 'withdraw';
	chainId: number;
	vault: SgVault;
	subgraphUrl: string;
};

export type RemoveOrderTransactionArgs = {
	config: Config;
	orderbookAddress: Hex;
	removeOrderCalldata: RemoveOrderCalldata;
	chainId: number;
	subgraphUrl: string;
};

export type TransactionState = {
	status: TransactionStatus;
	error: string;
	hash: string;
	data: null;
	functionName: string;
	message: string;
	newOrderHash: string;
	network: string;
	explorerLink: string;
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

export const initialState: TransactionState = {
	status: TransactionStatus.IDLE,
	error: '',
	hash: '',
	data: null,
	functionName: '',
	message: '',
	newOrderHash: '',
	network: '',
	explorerLink: ''
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
			message: 'Waiting for transaction to be indexed...'
		}));

		const result = await awaitSubgraphIndexing(
			getTransactionConfig(subgraphUrl, txHash, successMessage)
		);

		if (result.error) {
			return transactionError(TransactionErrorMessage.TIMEOUT);
		}

		if (result.value) {
			return transactionSuccess(result.value.txHash, result.value.successMessage);
		}
	};

	const awaitNewOrderIndexing = async (subgraphUrl: string, txHash: string, network?: string) => {
		update((state) => ({
			...state,
			status: TransactionStatus.PENDING_SUBGRAPH,
			message: 'Waiting for new order to be indexed...'
		}));

		const result = await awaitSubgraphIndexing(getNewOrderConfig(subgraphUrl, txHash, '', network));

		if (result.error) {
			return transactionError(TransactionErrorMessage.TIMEOUT);
		}

		if (result.value) {
			return transactionSuccess(
				result.value.txHash,
				result.value.successMessage,
				result.value.orderHash,
				result.value.network
			);
		}
	};

	const awaitRemoveOrderIndexing = async (subgraphUrl: string, txHash: string) => {
		update((state) => ({
			...state,
			status: TransactionStatus.PENDING_SUBGRAPH,
			message: 'Waiting for order removal to be indexed...'
		}));

		const result = await awaitSubgraphIndexing(
			getRemoveOrderConfig(subgraphUrl, txHash, 'Order removed successfully')
		);

		if (result.error) {
			return transactionError(TransactionErrorMessage.TIMEOUT);
		}

		if (result.value) {
			return transactionSuccess(result.value.txHash, result.value.successMessage);
		}
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
	const awaitTx = (
		hash: string,
		status: TransactionStatus,
		explorerLink?: string,
		message?: string
	) =>
		update((state) => ({
			...state,
			hash: hash,
			status: status,
			message: message || 'Waiting for transaction...',
			explorerLink: explorerLink || ''
		}));
	const transactionSuccess = (
		hash: string,
		message?: string,
		newOrderHash?: string,
		network?: string
	) => {
		update((state) => ({
			...state,
			status: TransactionStatus.SUCCESS,
			hash: hash,
			message: message || '',
			newOrderHash: newOrderHash || '',
			network: network || ''
		}));
	};
	const transactionError = (error: TransactionErrorMessage, hash?: string) =>
		update((state) => ({
			...state,
			status: TransactionStatus.ERROR,
			error: error,
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
			const transactionExplorerLink = await getExplorerLink(hash, chainId, 'tx');
			awaitTx(hash, TransactionStatus.PENDING_DEPLOYMENT, transactionExplorerLink);
			await waitForTransactionReceipt(config, { hash });
			if (subgraphUrl) {
				return awaitNewOrderIndexing(subgraphUrl, hash, network);
			}
			return transactionSuccess(
				hash,
				'Deployment successful. Check the Orders page for your new order.',
				'',
				network
			);
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
			const transactionExplorerLink = await getExplorerLink(hash, chainId, 'tx');
			awaitTx(
				hash,
				action === 'deposit'
					? TransactionStatus.PENDING_DEPOSIT
					: TransactionStatus.PENDING_WITHDRAWAL,
				transactionExplorerLink
			);
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
		subgraphUrl
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
			const transactionExplorerLink = await getExplorerLink(hash, chainId, 'tx');
			awaitTx(hash, TransactionStatus.PENDING_REMOVE_ORDER, transactionExplorerLink);
			await waitForTransactionReceipt(config, { hash });
			return awaitRemoveOrderIndexing(subgraphUrl, hash);
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
