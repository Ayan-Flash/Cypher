export const Npm = {
  name: "@cypher/cli",
  path: "@cypher%2fcli",
}

export const Brew = {
  name: "cypher",
  tap: "Cypher-Org/tap",
  formula: "Cypher-Org/tap/cypher",
  api: "https://formulae.brew.sh/api/formula/cypher.json",
}

export const Choco = {
  name: "cypher",
  api: "https://community.chocolatey.org/api/v2/Packages?$filter=Id%20eq%20%27cypher%27%20and%20IsLatestVersion&$select=Version",
}

export const Scoop = {
  name: "cypher",
  manifest: "https://raw.githubusercontent.com/ScoopInstaller/Main/master/bucket/cypher.json",
}

export const Release = {
  api: "https://api.github.com/repos/Cypher-Org/cypher/releases/latest",
  install: "https://cypher.ai/cli/install",
}
