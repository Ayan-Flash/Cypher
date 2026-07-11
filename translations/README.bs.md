<p align="center">
  <a href="../README.md">English</a> | <a href="README.zh.md">简体中文</a> | <a href="README.zht.md">繁體中文</a> | <a href="README.ko.md">한국어</a> | <a href="README.de.md">Deutsch</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.it.md">Italiano</a> | <a href="README.da.md">Dansk</a> | <a href="README.ja.md">日本語</a> | <a href="README.pl.md">Polski</a> | <a href="README.ru.md">Русский</a> | Bosanski | <a href="README.ar.md">العربية</a> | <a href="README.no.md">Norsk</a> | <a href="README.br.md">Português (Brasil)</a> | <a href="README.th.md">ไทย</a> | <a href="README.tr.md">Türkçe</a> | <a href="README.uk.md">Українська</a> | <a href="README.bn.md">বাংলা</a> | <a href="README.gr.md">Ελληνικά</a> | <a href="README.vi.md">Tiếng Việt</a>
</p>

<p align="center">
  <a href="https://cypher.ai"><img width="250" alt="Cypher CLI logo" src="https://github.com/user-attachments/assets/bdb0c174-b9fd-40ad-a47b-f3aab9b54e8d" /></a>
</p>

<p align="center">Open source agent za kodiranje s AI-jem u VS Codeu, JetBrainsu ili CLI-ju.</p>

<p align="center">
  <a href="https://marketplace.visualstudio.com/items?itemName=cypher.Cypher-Code"><img src="https://raster.shields.io/badge/VS_Code_Marketplace-007ACC?style=flat&logo=visualstudiocode&logoColor=white" alt="VS Code Marketplace" height="20"></a>
  <a href="https://www.npmjs.com/package/@cypher/cli"><img alt="npm" src="https://raster.shields.io/npm/v/@cypher/cli?style=flat" height="20" /></a>
  <a href="https://x.com/cypher"><img src="https://raster.shields.io/badge/cypher-000000?style=flat&logo=x&logoColor=white" alt="X (Twitter)" height="20"></a>
  <a href="https://blog.cypher.ai"><img src="https://raster.shields.io/badge/Blog-555?style=flat&logo=substack&logoColor=white" alt="Blog" height="20"></a>
  <a href="https://cypher.ai/discord"><img src="https://raster.shields.io/badge/Join%20Discord-5865F2?style=flat&logo=discord&logoColor=white" alt="Discord" height="20"></a>
  <a href="https://www.reddit.com/r/cypher/"><img src="https://raster.shields.io/badge/Join%20r%2Fcypher-D84315?style=flat&logo=reddit&logoColor=white" alt="Reddit" height="20"></a>
</p>

![Cypher-in-VS-Code-and-CLI](https://github.com/user-attachments/assets/0536ca59-ed81-4512-9e05-d186187a1b52)

---

Cypher CLI je AI agent za kodiranje koji vas prati svugdje gdje radite: [VS Code](https://cypher.ai/landing/vs-code), [JetBrains](https://cypher.ai/features/jetbrains-native) i [CLI](https://cypher.ai/cli). Open source je i ima otvorene cijene. Birate između više od 500 modela, mijenjate ih usred zadatka i plaćate cijenu pružaoca modela bez dodatne marže. API ključevi nisu potrebni za početak.

### Instalacija

Odaberite gdje želite pokrenuti Cypher.

<details open>
<summary><strong>VS Code</strong></summary>

<br>

Instalirajte [Cypher CLI ekstenziju](vscode:extension/cypher.cypher-cli) direktno ili je preuzmite sa [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=cypher.Cypher-Code). Kreirajte račun i imat ćete pristup za više od 500 modela, uključujući GPT-5.5, Claude Opus 4.7, Claude Sonnet 4.6 i Gemini 3.1 Pro Preview, sve po cijenama pružaoca.

</details>

<details open>
<summary><strong>CLI</strong></summary>

<br>

```bash
# npm
npm install -g @cypher/cli

# curl
curl -fsSL https://cypher.ai/cli/install | bash

# pnpm
pnpm add -g @cypher/cli

# bun
bun add -g @cypher/cli

# Homebrew (macOS / Linux)
brew install Cypher-Org/tap/cypher

# Arch Linux (AUR)
paru -S cypher-bin
```

Zatim pokrenite `cypher` u bilo kojem direktoriju projekta.

</details>

<details>
<summary><strong>JetBrains</strong></summary>

<br>

Instalirajte [Cypher CLI plugin](https://plugins.jetbrains.com/plugin/28350-cypher-cli) sa JetBrains Marketplacea ili potražite "Cypher CLI" u `Settings → Plugins` unutar bilo kojeg JetBrains IDE-a.

</details>

<details>
<summary><strong>Cloud Agent</strong></summary>

<br>

Pokrenite Cypher s weba, bez lokalne mašine, na [app.cypher.ai/cloud](https://app.cypher.ai/cloud).

</details>

<details>
<summary><strong>Pregledi koda</strong></summary>

<br>

Postavite automatske AI preglede koda na svojim pull requestovima na [app.cypher.ai/code-reviews](https://app.cypher.ai/code-reviews).

</details>

<details>
<summary><strong>CypherClaw</strong></summary>

<br>

Pokrenite svog uvijek aktivnog AI agenta na [app.cypher.ai/claw](https://app.cypher.ai/claw).

</details>

<details>
<summary>Instalirajte CLI iz GitHub Releases (binarne datoteke)</summary>

Preuzmite najnoviju binarnu datoteku sa [Releases stranice](https://github.com/Cypher-Org/cypher/releases).

| Platforma | Asset |
|---|---|
| Windows (većina PC računara) | `cypher-windows-x64.zip` |
| macOS (Apple Silicon) | `cypher-darwin-arm64.zip` |
| macOS (Intel) | `cypher-darwin-x64.zip` |
| Linux x64 | `cypher-linux-x64.tar.gz` |
| Linux ARM | `cypher-linux-arm64.tar.gz` |

Napomene: `x64-baseline` je kompatibilna verzija za starije CPU-e bez AVX-a. `musl` je statički linkovana verzija za Alpine ili minimalne Docker slike bez glibc-a. `cypher-vscode-*.vsix` je paket VS Code ekstenzije, ne CLI. `Source code` arhive služe za build iz izvornog koda.

</details>

### Agents

Cypher dolazi sa specijaliziranim agents koje mijenjate zavisno od zadatka. Možete napraviti i vlastite prilagođene agents.

- **Code** - Zadani. Implementira i uređuje kod iz prirodnog jezika.
- **Plan** - Dizajnira arhitekturu i piše implementacijske planove prije pisanja koda.
- **Ask** - Odgovara na pitanja o codebaseu bez mijenjanja datoteka.
- **Debug** - Rješava i prati probleme.
- **Review** - Pregleda vaše promjene i pronalazi probleme u performansama, sigurnosti, stilu i pokrivenosti testovima.

Saznajte više o [agents i prilagođenim agents](https://cypher.ai/docs/code-with-ai/agents/using-agents).

### Šta radi

- **Generisanje koda** iz prirodnog jezika, kroz više datoteka.
- **Inline autocomplete** sa ghost-text prijedlozima i Tab za prihvatanje.
- **Samoprovjera** kako bi agent pregledao i ispravio vlastiti rad.
- **Kontrola terminala i browsera** za pokretanje komandi i automatizaciju weba.
- **MCP marketplace** za pronalaženje i povezivanje MCP servera koji proširuju mogućnosti agenta.
- **Više od 500 modela** sa prebacivanjem usred zadatka, da uskladite latenciju, cijenu i rezonovanje s poslom.

### Autonomni način rada (CI/CD)

Pokrenite `cypher run` s `--auto` za potpuno autonoman rad bez promptova, napravljen za CI/CD pipelineove:

```bash
cypher run --auto "run tests and fix any failures"
```

`--auto` isključuje sve upite za dozvole i dopušta agentu da izvrši bilo koju radnju bez potvrde. Koristite samo u pouzdanim okruženjima.

### Dokumentacija

Za konfiguraciju i sve ostalo posjetite [dokumentaciju](https://cypher.ai/docs).

### Doprinos

Doprinosi su dobrodošli od developera, pisaca i svih ostalih. Počnite sa [Contributing Guide](/CONTRIBUTING.md) za podešavanje okruženja, standarde kodiranja i otvaranje pull requesta. Pogledajte [RELEASING.md](../RELEASING.md) za proces izdavanja VS Code ekstenzije i CLI-ja, te [packages/cypher-jetbrains/RELEASING.md](../packages/cypher-jetbrains/RELEASING.md) za JetBrains plugin.

Prije uključivanja pročitajte naš [Code of Conduct](/CODE_OF_CONDUCT.md).

### Licenca

MIT. Možete koristiti, mijenjati i distribuirati ovaj kod, uključujući komercijalno, dok god zadržite atribuciju i obavijesti o licenci. Pogledajte [License](/LICENSE).

### FAQ

<details>
<summary>Odakle dolazi Cypher CLI?</summary>

Cypher CLI je fork [OpenCode](https://github.com/anomalyco/opencode), poboljšan za rad unutar Cypher agentic engineering platforme.

</details>

---

**Pridružite se zajednici** [Discord](https://cypher.ai/discord) | [X](https://x.com/cypher) | [Reddit](https://www.reddit.com/r/cypher/)
