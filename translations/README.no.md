<p align="center">
  <a href="../README.md">English</a> | <a href="README.zh.md">简体中文</a> | <a href="README.zht.md">繁體中文</a> | <a href="README.ko.md">한국어</a> | <a href="README.de.md">Deutsch</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.it.md">Italiano</a> | <a href="README.da.md">Dansk</a> | <a href="README.ja.md">日本語</a> | <a href="README.pl.md">Polski</a> | <a href="README.ru.md">Русский</a> | <a href="README.bs.md">Bosanski</a> | <a href="README.ar.md">العربية</a> | Norsk | <a href="README.br.md">Português (Brasil)</a> | <a href="README.th.md">ไทย</a> | <a href="README.tr.md">Türkçe</a> | <a href="README.uk.md">Українська</a> | <a href="README.bn.md">বাংলা</a> | <a href="README.gr.md">Ελληνικά</a> | <a href="README.vi.md">Tiếng Việt</a>
</p>

<p align="center">
  <a href="https://cypher.ai"><img width="250" alt="Cypher CLI logo" src="https://github.com/user-attachments/assets/bdb0c174-b9fd-40ad-a47b-f3aab9b54e8d" /></a>
</p>

<p align="center">Den åpne kildekodeagenten for å bygge med AI i VS Code, JetBrains eller CLI.</p>

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

Cypher CLI er en AI-kodeagent som møter deg overalt du jobber: [VS Code](https://cypher.ai/landing/vs-code), [JetBrains](https://cypher.ai/features/jetbrains-native) og [CLI](https://cypher.ai/cli). Den er åpen kildekode med åpen prising. Du velger blant mer enn 500 modeller, bytter mellom dem midt i en oppgave og betaler modellleverandørens pris uten påslag. Ingen API-nøkler kreves for å starte.

### Installasjon

Velg hvor du vil kjøre Cypher.

<details open>
<summary><strong>VS Code</strong></summary>

<br>

Installer [Cypher CLI-utvidelsen](vscode:extension/cypher.cypher-cli) direkte, eller hent den fra [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=cypher.Cypher-Code). Opprett en konto, og du får tilgang til mer enn 500 modeller, inkludert GPT-5.5, Claude Opus 4.7, Claude Sonnet 4.6 og Gemini 3.1 Pro Preview, alle til leverandørpris.

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

Kjør deretter `cypher` i en prosjektmappe for å starte.

</details>

<details>
<summary><strong>JetBrains</strong></summary>

<br>

Installer [Cypher CLI-pluginen](https://plugins.jetbrains.com/plugin/28350-cypher-cli) fra JetBrains Marketplace, eller søk etter "Cypher CLI" i `Settings → Plugins` i en JetBrains IDE.

</details>

<details>
<summary><strong>Cloud Agent</strong></summary>

<br>

Kjør Cypher fra nettet, uten lokal maskin, på [app.cypher.ai/cloud](https://app.cypher.ai/cloud).

</details>

<details>
<summary><strong>Kodegjennomganger</strong></summary>

<br>

Sett opp automatiske AI-kodegjennomganger på pull requestene dine på [app.cypher.ai/code-reviews](https://app.cypher.ai/code-reviews).

</details>

<details>
<summary><strong>CypherClaw</strong></summary>

<br>

Start din alltid aktive AI-agent på [app.cypher.ai/claw](https://app.cypher.ai/claw).

</details>

<details>
<summary>Installer CLI fra GitHub Releases (binærfiler)</summary>

Last ned den nyeste binærfilen fra [Releases-siden](https://github.com/Cypher-Org/cypher/releases).

| Plattform | Asset |
|---|---|
| Windows (de fleste PC-er) | `cypher-windows-x64.zip` |
| macOS (Apple Silicon) | `cypher-darwin-arm64.zip` |
| macOS (Intel) | `cypher-darwin-x64.zip` |
| Linux x64 | `cypher-linux-x64.tar.gz` |
| Linux ARM | `cypher-linux-arm64.tar.gz` |

Merknader: `x64-baseline` er en kompatibilitetsbygg for eldre CPU-er uten AVX. `musl` er den statisk lenkede byggen for Alpine eller minimale Docker-bilder uten glibc. `cypher-vscode-*.vsix` er VS Code-utvidelsespakken, ikke CLI-en. `Source code`-arkiver er for bygging fra kildekode.

</details>

### Agents

Cypher leveres med spesialiserte agents du kan bytte mellom avhengig av oppgaven. Du kan også bygge dine egne egendefinerte agents.

- **Code** - Standard. Implementerer og redigerer kode fra naturlig språk.
- **Plan** - Designer arkitektur og skriver implementeringsplaner før kode skrives.
- **Ask** - Svarer på spørsmål om kodebasen uten å endre filer.
- **Debug** - Feilsøker og sporer problemer.
- **Review** - Gjennomgår endringene dine og finner problemer med ytelse, sikkerhet, stil og testdekning.

Les mer om [agents og egendefinerte agents](https://cypher.ai/docs/code-with-ai/agents/using-agents).

### Hva den gjør

- **Kodegenerering** fra naturlig språk, på tvers av flere filer.
- **Inline-autofullføring** med ghost-text-forslag og Tab for å godta.
- **Selvsjekking** slik at agenten vurderer og retter sitt eget arbeid.
- **Terminal- og nettleserkontroll** for å kjøre kommandoer og automatisere nettet.
- **MCP-markedsplass** for å finne og koble til MCP-servere som utvider hva agenten kan gjøre.
- **Mer enn 500 modeller** med bytte midt i oppgaven, slik at du kan matche latenstid, kostnad og resonnering til jobben.

### Autonom modus (CI/CD)

Kjør `cypher run` med `--auto` for helt autonom drift uten spørsmål, bygget for CI/CD-pipelines:

```bash
cypher run --auto "run tests and fix any failures"
```

`--auto` deaktiverer alle tillatelsesspørsmål og lar agenten utføre enhver handling uten bekreftelse. Bruk det bare i betrodde miljøer.

### Dokumentasjon

For konfigurasjon og alt annet, se [dokumentasjonen](https://cypher.ai/docs).

### Bidra

Bidrag er velkomne fra utviklere, skribenter og alle andre. Start med [Contributing Guide](/CONTRIBUTING.md) for miljøoppsett, kodestandarder og hvordan du åpner en pull request. Se [RELEASING.md](../RELEASING.md) for releaseprosessen for VS Code-utvidelsen og CLI-en, og [packages/cypher-jetbrains/RELEASING.md](../packages/cypher-jetbrains/RELEASING.md) for JetBrains-pluginen.

Les vår [Code of Conduct](/CODE_OF_CONDUCT.md) før du deltar.

### Lisens

MIT. Du kan bruke, endre og distribuere denne koden, også kommersielt, så lenge du beholder attribusjons- og lisensmerknadene. Se [License](/LICENSE).

### FAQ

<details>
<summary>Hvor kommer Cypher CLI fra?</summary>

Cypher CLI er en fork av [OpenCode](https://github.com/anomalyco/opencode), forbedret for å fungere i Cypher agentic engineering-plattformen.

</details>

---

**Bli med i fellesskapet** [Discord](https://cypher.ai/discord) | [X](https://x.com/cypher) | [Reddit](https://www.reddit.com/r/cypher/)
