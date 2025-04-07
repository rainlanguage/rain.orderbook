import { REGISTRY_URL } from "$lib/constants";

// Example registry manager class
export default class RegistryManager {
  private static STORAGE_KEY = 'registry';
  
  static getFromStorage(): string | null {
    return localStorage.getItem(this.STORAGE_KEY);
  }
  
  static setToStorage(value: string): void {
    localStorage.setItem(this.STORAGE_KEY, value);
  }
  
  static clearFromStorage(): void {
    localStorage.removeItem(this.STORAGE_KEY);
  }
  
  static updateUrlParam(value: string | null): void {
    if (value) {
      const url = new URL(window.location.href);
      url.searchParams.set('registry', value);
      window.history.pushState({}, '', url.toString());
    }
  }
  
  static isCustomRegistry(value: string | null): boolean {
    return !!value && value !== REGISTRY_URL;
  }
}

