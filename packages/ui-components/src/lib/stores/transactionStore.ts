import { writable } from 'svelte/store';
import type { Hex } from 'viem';
import type { Config } from '@wagmi/core';
import { sendTransaction, waitForTransactionReceipt } from '@wagmi/core';
import type { ApprovalCalldataResult, DepositAndAddOrderCalldataResult } from '@rainlanguage/orderbook/js_api';

export const ADDRESS_ZERO = '0x0000000000000000000000000000000000000000';
export const ONE = BigInt('1000000000000000000');

export enum TransactionStatus {
	IDLE = 'Idle',
	CHECKING_ALLOWANCE = 'Checking your approved sFLR spend...',
	PENDING_WALLET = 'Waiting for wallet confirmation...',
	PENDING_APPROVAL = 'Approving token spend...',
	PENDING_LOCK = 'Locking sFLR...',
	PENDING_UNLOCK = 'Unlocking sFLR...',
	SUCCESS = 'Success! Transaction confirmed',
	ERROR = 'Something went wrong',
	PENDING_DEPLOYMENT = 'Deploying strategy...',
}

export enum TransactionErrorMessage {
	USER_REJECTED = 'User rejected transaction',
	BAD_CALLLDATA = 'Bad calldata',
	DEPLOY_FAILED = 'Lock transaction failed',
	TIMEOUT = 'Transaction timed out',
	APPROVAL_FAILED = 'Approval transaction failed',
	DEPLOYMENT_FAILED = 'Deployment transaction failed',
}

export type DeploymentTransactionArgs = {
	config: Config;
	address: Hex;
	approvals: ApprovalCalldataResult;
	deploymentCalldata: DepositAndAddOrderCalldataResult;
	orderbookAddress: Hex;
};

export type TransactionState = {
	status: TransactionStatus;
	error: string;
	hash: string;
	data: null;
	functionName: string;
	message: string;
};

export type TransactionStore = {
	subscribe: (run: (value: TransactionState) => void) => () => void;
	reset: () => void;
	handleDeploymentTransaction: (args: DeploymentTransactionArgs) => Promise<void>;
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
	message: ''
};

const transactionStore = () => {
	const { subscribe, set, update } = writable(initialState);
	const reset = () => set(initialState);

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
	const awaitApprovalTx = (hash: string) =>
		update((state) => ({
			...state,
			hash: hash,
			status: TransactionStatus.PENDING_APPROVAL,
			message: ''
		}));
	const awaitDeployTx = (hash: string) =>
		update((state) => ({
			...state,
			hash: hash,
			status: TransactionStatus.PENDING_LOCK,
			message: ''
		}));
	const transactionSuccess = (hash: string, message?: string) =>
		update((state) => ({
			...state,
			status: TransactionStatus.SUCCESS,
			hash: hash,
			message: message || ''
		}));
	const transactionError = (message: TransactionErrorMessage, hash?: string) =>
		update((state) => ({
			...state,
			status: TransactionStatus.ERROR,
			error: message,
			hash: hash || ''
		}));

	const handleDeploymentTransaction = async ({
		config,
		address,
		approvals,
		deploymentCalldata,
		orderbookAddress
	}: DeploymentTransactionArgs) => {
		// Handle approvals first
		for (const approval of approvals) {
			try {
				awaitWalletConfirmation('Please approve token spend in your wallet...');
				const hash = await sendTransaction(config, {
					to: approval.token as `0x${string}`,
					data: approval.calldata as `0x${string}`
				});

				awaitApprovalTx(hash);
				await waitForTransactionReceipt(config, { hash });
			} catch {
				return transactionError(TransactionErrorMessage.APPROVAL_FAILED);
			}
		}
		try {
			awaitWalletConfirmation('Please confirm deployment in your wallet...');
			const hash = await sendTransaction(config, {
				to: orderbookAddress as `0x${string}`,
				data: deploymentCalldata as `0x${string}`
			});

			awaitDeployTx(hash);
			await waitForTransactionReceipt(config, { hash });
			return transactionSuccess(hash, 'Strategy deployed successfully!');
		} catch {
			return transactionError(TransactionErrorMessage.DEPLOYMENT_FAILED);
		}
	};


	return {
		subscribe,
		reset,
		handleDeploymentTransaction,
		checkingWalletAllowance,
		awaitWalletConfirmation,
		awaitApprovalTx,
		transactionSuccess,
		transactionError
	};
};

export default transactionStore();
