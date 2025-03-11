import { writable } from 'svelte/store';

export enum DeploymentStepsErrorCode {
	NO_GUI = 'Error loading GUI',
	DEPLOYMENT_UPDATE_ERROR = 'Error updating deployment',
	NO_SELECT_TOKENS = 'No select tokens',
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
