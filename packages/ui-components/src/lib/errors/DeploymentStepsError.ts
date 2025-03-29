import { writable } from 'svelte/store';

export enum DeploymentStepsErrorCode {
	NO_GUI_PROVIDER = 'No GUI provider found.',
	NO_GUI = 'Error loading GUI.',
	NO_STRATEGY = 'No valid strategy exists at this URL',
	NO_SELECT_TOKENS = 'Error loading tokens',
	NO_TOKEN_INFO = 'Error loading token information',
	NO_FIELD_DEFINITIONS = 'Error loading field definitions',
	NO_DEPOSITS = 'Error loading deposits',
	NO_TOKEN_INPUTS = 'Error loading token inputs',
	NO_TOKEN_OUTPUTS = 'Error loading token outputs',
	NO_GUI_DETAILS = 'Error getting GUI details',
	NO_CHAIN = 'Unsupported chain ID',
	NO_NETWORK_KEY = 'No network key found',
	SERIALIZE_ERROR = 'Error serializing state',
	ADD_ORDER_FAILED = 'Failed to add order'
}

export class DeploymentStepsError extends Error {
	private static errorStore = writable<DeploymentStepsError | null>(null);

	constructor(
		public code: DeploymentStepsErrorCode,
		public details?: string
	) {
		super(code);
		this.name = 'DeploymentStepsError';
	}

	static get error() {
		return this.errorStore;
	}

	static throwIfNull<T>(value: T | null | undefined, code: DeploymentStepsErrorCode): T {
		if (value === null || value === undefined) {
			throw new DeploymentStepsError(code);
		}
		return value;
	}

	static catch(e: unknown, code: DeploymentStepsErrorCode) {
		const error =
			e instanceof DeploymentStepsError
				? e
				: new DeploymentStepsError(code, e instanceof Error ? e.message : 'Unknown error');
		this.errorStore.set(error);
	}

	static clear() {
		this.errorStore.set(null);
	}
}
