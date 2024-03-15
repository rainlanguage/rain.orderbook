// eslint-disable-next-line @typescript-eslint/no-var-requires
const fs = require("fs");

let config_content = fs.readFileSync("./tauri-app/src/lib/typeshare/config.ts", { encoding: "utf-8" });
config_content = config_content.replace(`export interface Network {
	rpc: string;
	label?: string;
	currency?: string;
}`, `export interface Network {
	rpc: string;
  "chain-id": number;
	label?: string;
  "network-id"?: number;
	currency?: string;
}`);
config_content = config_content.replace(`export interface Scenario {
	bindings: Record<string, string>;
	deployer: Deployer;
}`, `export interface Scenario {
	bindings: Record<string, string>;
  runs?: number;
	deployer: Deployer;
}`);
fs.writeFileSync("./tauri-app/src/lib/typeshare/config.ts", config_content);
