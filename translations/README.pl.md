<p align="center">
  <a href="../README.md">English</a> | <a href="README.zh.md">简体中文</a> | <a href="README.zht.md">繁體中文</a> | <a href="README.ko.md">한국어</a> | <a href="README.de.md">Deutsch</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.it.md">Italiano</a> | <a href="README.da.md">Dansk</a> | <a href="README.ja.md">日本語</a> | Polski | <a href="README.ru.md">Русский</a> | <a href="README.bs.md">Bosanski</a> | <a href="README.ar.md">العربية</a> | <a href="README.no.md">Norsk</a> | <a href="README.br.md">Português (Brasil)</a> | <a href="README.th.md">ไทย</a> | <a href="README.tr.md">Türkçe</a> | <a href="README.uk.md">Українська</a> | <a href="README.bn.md">বাংলা</a> | <a href="README.gr.md">Ελληνικά</a> | <a href="README.vi.md">Tiếng Việt</a>
</p>

<p align="center">
  <a href="https://cypher.ai"><img width="250" alt="Cypher CLI logo" src="https://github.com/user-attachments/assets/bdb0c174-b9fd-40ad-a47b-f3aab9b54e8d" /></a>
</p>

<p align="center">Open source'owy agent kodujący do pracy z AI w VS Code, JetBrains lub CLI.</p>

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

Cypher CLI to agent kodujący z AI, który działa wszędzie tam, gdzie pracujesz: w [VS Code](https://cypher.ai/landing/vs-code), [JetBrains](https://cypher.ai/features/jetbrains-native) i [CLI](https://cypher.ai/cli). Jest open source i ma otwarte ceny. Wybierasz spośród ponad 500 modeli, przełączasz się między nimi w trakcie zadania i płacisz stawkę dostawcy modelu bez narzutów. Do rozpoczęcia nie są wymagane klucze API.

### Instalacja

Wybierz, gdzie chcesz uruchomić Cypher.

<details open>
<summary><strong>VS Code</strong></summary>

<br>

Zainstaluj bezpośrednio [rozszerzenie Cypher CLI](vscode:extension/cypher.cypher-cli) albo pobierz je z [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=cypher.Cypher-Code). Utwórz konto, a otrzymasz dostęp do ponad 500 modeli, w tym GPT-5.5, Claude Opus 4.7, Claude Sonnet 4.6 i Gemini 3.1 Pro Preview, wszystkie w cenach dostawców.

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

Następnie uruchom `cypher` w dowolnym katalogu projektu.

</details>

<details>
<summary><strong>JetBrains</strong></summary>

<br>

Zainstaluj [plugin Cypher CLI](https://plugins.jetbrains.com/plugin/28350-cypher-cli) z JetBrains Marketplace albo wyszukaj "Cypher CLI" w `Settings → Plugins` w dowolnym IDE JetBrains.

</details>

<details>
<summary><strong>Cloud Agent</strong></summary>

<br>

Uruchom Cypher z poziomu przeglądarki, bez lokalnej maszyny, na [app.cypher.ai/cloud](https://app.cypher.ai/cloud).

</details>

<details>
<summary><strong>Przeglądy kodu</strong></summary>

<br>

Skonfiguruj automatyczne przeglądy kodu AI dla swoich pull requestów na [app.cypher.ai/code-reviews](https://app.cypher.ai/code-reviews).

</details>

<details>
<summary><strong>CypherClaw</strong></summary>

<br>

Uruchom swojego zawsze aktywnego agenta AI na [app.cypher.ai/claw](https://app.cypher.ai/claw).

</details>

<details>
<summary>Zainstaluj CLI z GitHub Releases (pliki binarne)</summary>

Pobierz najnowszy plik binarny ze [strony Releases](https://github.com/Cypher-Org/cypher/releases).

| Platforma | Zasób |
|---|---|
| Windows (większość PC) | `cypher-windows-x64.zip` |
| macOS (Apple Silicon) | `cypher-darwin-arm64.zip` |
| macOS (Intel) | `cypher-darwin-x64.zip` |
| Linux x64 | `cypher-linux-x64.tar.gz` |
| Linux ARM | `cypher-linux-arm64.tar.gz` |

Uwagi: `x64-baseline` to build zgodności dla starszych CPU bez AVX. `musl` to statycznie linkowany build dla Alpine lub minimalnych obrazów Docker bez glibc. `cypher-vscode-*.vsix` to pakiet rozszerzenia VS Code, nie CLI. Archiwa `Source code` służą do budowania ze źródeł.

</details>

### Agents

Cypher zawiera wyspecjalizowane agents, między którymi możesz przełączać się zależnie od zadania. Możesz też tworzyć własne niestandardowe agents.

- **Code** - Domyślny. Implementuje i edytuje kod z języka naturalnego.
- **Plan** - Projektuje architekturę i pisze plany implementacji przed napisaniem kodu.
- **Ask** - Odpowiada na pytania o bazę kodu bez modyfikowania plików.
- **Debug** - Diagnozuje i śledzi problemy.
- **Review** - Przegląda zmiany i wykrywa problemy z wydajnością, bezpieczeństwem, stylem i pokryciem testami.

Dowiedz się więcej o [agents i niestandardowych agents](https://cypher.ai/docs/code-with-ai/agents/using-agents).

### Co robi

- **Generowanie kodu** z języka naturalnego, w wielu plikach.
- **Autouzupełnianie inline** z sugestiami ghost-text i akceptacją przez Tab.
- **Samokontrola**, dzięki której agent sprawdza i poprawia własną pracę.
- **Sterowanie terminalem i przeglądarką** do uruchamiania poleceń i automatyzacji webu.
- **Marketplace MCP** do znajdowania i podłączania serwerów MCP rozszerzających możliwości agenta.
- **Ponad 500 modeli** z przełączaniem w trakcie zadania, aby dopasować opóźnienie, koszt i rozumowanie do pracy.

### Tryb autonomiczny (CI/CD)

Uruchom `cypher run` z `--auto`, aby działać w pełni autonomicznie bez promptów, z myślą o pipeline'ach CI/CD:

```bash
cypher run --auto "run tests and fix any failures"
```

`--auto` wyłącza wszystkie pytania o uprawnienia i pozwala agentowi wykonywać dowolne działania bez potwierdzenia. Używaj tylko w zaufanych środowiskach.

### Dokumentacja

Konfigurację i wszystko inne znajdziesz w [dokumentacji](https://cypher.ai/docs).

### Wkład

Zapraszamy do wkładu programistów, autorów i wszystkich innych. Zacznij od [Contributing Guide](/CONTRIBUTING.md), aby skonfigurować środowisko, poznać standardy kodowania i sposób otwierania pull requestów. Zobacz [RELEASING.md](../RELEASING.md) dla procesu wydawania rozszerzenia VS Code i CLI oraz [packages/cypher-jetbrains/RELEASING.md](../packages/cypher-jetbrains/RELEASING.md) dla pluginu JetBrains.

Przed zaangażowaniem przeczytaj nasz [Code of Conduct](/CODE_OF_CONDUCT.md).

### Licencja

MIT. Możesz używać, modyfikować i dystrybuować ten kod, również komercyjnie, o ile zachowasz informacje o autorstwie i licencji. Zobacz [License](/LICENSE).

### FAQ

<details>
<summary>Skąd pochodzi Cypher CLI?</summary>

Cypher CLI jest forkiem [OpenCode](https://github.com/anomalyco/opencode), rozszerzonym do działania w platformie agentic engineering Cypher.

</details>

---

**Dołącz do społeczności** [Discord](https://cypher.ai/discord) | [X](https://x.com/cypher) | [Reddit](https://www.reddit.com/r/cypher/)
