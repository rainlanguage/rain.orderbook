import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

// Define __filename and __dirname for ES module compatibility
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Define the components directory and the index file location
const componentsDir = path.join(__dirname, 'components'); // Adjust if needed
const indexFile = path.join(__dirname, 'index.ts'); // index.ts is in the same directory as this script

// Check if the components directory exists
if (!fs.existsSync(componentsDir)) {
  console.error(`Directory ${componentsDir} does not exist.`);
  process.exit(1);
}

/**
 * Recursively finds all .svelte files within a directory and its subdirectories.
 * @param dir - The directory to scan for .svelte files.
 * @returns An array of relative paths to each .svelte file.
 */
function getSvelteFiles(dir: string): string[] {
  let svelteFiles: string[] = [];
  
  const files = fs.readdirSync(dir);

  for (const file of files) {
    const fullPath = path.join(dir, file);
    const relativePath = path.relative(__dirname, fullPath);

    if (fs.statSync(fullPath).isDirectory()) {
      svelteFiles = svelteFiles.concat(getSvelteFiles(fullPath));
    } else if (file.endsWith('.svelte') && !file.includes('.test.')) {
      svelteFiles.push(relativePath);
    }
  }

  return svelteFiles;
}

// Get all .svelte files in the components directory recursively
const components = getSvelteFiles(componentsDir);

// Generate export statements for each .svelte file found
const exports = components.map((filePath) => {
  const componentName = path.basename(filePath, '.svelte');
  const importPath = `./${filePath.replace(/\\/g, '/')}`; // Normalize for cross-platform compatibility
  return `export { default as ${componentName} } from '${importPath}';`;
}).join('\n');

// Check if any exports were generated
if (exports.length === 0) {
  console.warn("No exports to write.");
} else {
  // Write the generated exports to index.ts
  fs.writeFileSync(indexFile, exports);
  console.log(`Generated exports for ${components.length} components in index.ts`);
}
