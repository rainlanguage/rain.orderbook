export interface AppStoresInterface {
	settings: {
		subscribe: (callback: (value: Record<string, string>) => void) => void;
		set: (value: Record<string, string>) => void;
	};
	activeSubgraphs: {
		subscribe: (callback: (value: Record<string, string>) => void) => void;
		set: (value: Record<string, string>) => void;
	};
	accounts: {
		subscribe: (callback: (value: Record<string, string>) => void) => void;
		set: (value: Record<string, string>) => void;
	};
	activeAccountsItems: {
		subscribe: (callback: (value: Record<string, string>) => void) => void;
		set: (value: Record<string, string>) => void;
	};
}
