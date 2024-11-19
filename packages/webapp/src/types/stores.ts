export interface AppStores {
  settings: {
    subscribe: (callback: (value: Record<string, any>) => void) => void;
    set: (value: Record<string, any>) => void;
  };
  activeSubgraphs: {
    subscribe: (callback: (value: Record<string, string>) => void) => void;
    set: (value: Record<string, string>) => void;
  };
} 