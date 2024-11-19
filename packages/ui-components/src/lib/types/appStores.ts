export interface AppStoresInterface {
  settings: {
    subscribe: (callback: (value: Record<string, string>) => void) => void;
    set: (value: Record<string, string>) => void;
  };
  activeSubgraphs: {
    subscribe: (callback: (value: Record<string, string>) => void) => void;
    set: (value: Record<string, string>) => void;
  };
} 