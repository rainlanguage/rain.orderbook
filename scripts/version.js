const fs = require("fs");
const { execSync } = require("child_process");

const [level] = process.argv.slice(2);

if (level === "alpha") {
  execSync("npm version prerelease --preid alpha --no-git-tag-version");
}
if (level === "beta") {
  execSync("npm version prerelease --preid beta --no-git-tag-version");
}
if (level === "rc") {
  execSync("npm version prerelease --preid rc --no-git-tag-version");
}
if (level === "patch") {
  execSync("npm version patch --no-git-tag-version");
}
if (level === "minor") {
  execSync("npm version minor --no-git-tag-version");
}
if (level === "major") {
  execSync("npm version major --no-git-tag-version");
}
if (level === "release") {
  execSync("npm version patch --no-git-tag-version");
}

const package = JSON.parse(fs.readFileSync(`./package.json`));
console.log(package.version);
