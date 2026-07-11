<p align="center">
  <a href="../README.md">English</a> | <a href="README.zh.md">简体中文</a> | <a href="README.zht.md">繁體中文</a> | <a href="README.ko.md">한국어</a> | <a href="README.de.md">Deutsch</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.it.md">Italiano</a> | <a href="README.da.md">Dansk</a> | <a href="README.ja.md">日本語</a> | <a href="README.pl.md">Polski</a> | <a href="README.ru.md">Русский</a> | <a href="README.bs.md">Bosanski</a> | <a href="README.ar.md">العربية</a> | <a href="README.no.md">Norsk</a> | <a href="README.br.md">Português (Brasil)</a> | <a href="README.th.md">ไทย</a> | <a href="README.tr.md">Türkçe</a> | <a href="README.uk.md">Українська</a> | <a href="README.bn.md">বাংলা</a> | Ελληνικά | <a href="README.vi.md">Tiếng Việt</a>
</p>

<p align="center">
  <a href="https://cypher.ai"><img width="250" alt="Cypher CLI logo" src="https://github.com/user-attachments/assets/bdb0c174-b9fd-40ad-a47b-f3aab9b54e8d" /></a>
</p>

<p align="center">Ο open source agent προγραμματισμού για δημιουργία με AI σε VS Code, JetBrains ή CLI.</p>

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

Το Cypher CLI είναι ένας AI agent προγραμματισμού που σας συναντά παντού όπου εργάζεστε: [VS Code](https://cypher.ai/landing/vs-code), [JetBrains](https://cypher.ai/features/jetbrains-native) και [CLI](https://cypher.ai/cli). Είναι open source με ανοιχτή τιμολόγηση. Επιλέγετε από περισσότερα από 500 μοντέλα, αλλάζετε μεταξύ τους στη μέση μιας εργασίας και πληρώνετε την τιμή του παρόχου του μοντέλου χωρίς προσαύξηση. Δεν απαιτούνται API keys για να ξεκινήσετε.

### Εγκατάσταση

Επιλέξτε πού θέλετε να εκτελέσετε το Cypher.

<details open>
<summary><strong>VS Code</strong></summary>

<br>

Εγκαταστήστε απευθείας την [επέκταση Cypher CLI](vscode:extension/cypher.cypher-cli) ή κατεβάστε τη από το [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=cypher.Cypher-Code). Δημιουργήστε λογαριασμό και θα έχετε πρόσβαση σε περισσότερα από 500 μοντέλα, όπως GPT-5.5, Claude Opus 4.7, Claude Sonnet 4.6 και Gemini 3.1 Pro Preview, όλα στην τιμή του παρόχου.

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

Στη συνέχεια εκτελέστε `cypher` σε οποιονδήποτε κατάλογο έργου για να ξεκινήσετε.

</details>

<details>
<summary><strong>JetBrains</strong></summary>

<br>

Εγκαταστήστε το [plugin Cypher CLI](https://plugins.jetbrains.com/plugin/28350-cypher-cli) από το JetBrains Marketplace ή αναζητήστε "Cypher CLI" στο `Settings → Plugins` σε οποιοδήποτε JetBrains IDE.

</details>

<details>
<summary><strong>Cloud Agent</strong></summary>

<br>

Εκτελέστε το Cypher από τον ιστό, χωρίς τοπικό μηχάνημα, στο [app.cypher.ai/cloud](https://app.cypher.ai/cloud).

</details>

<details>
<summary><strong>Code Reviews</strong></summary>

<br>

Ρυθμίστε αυτοματοποιημένα AI code reviews στα pull requests σας στο [app.cypher.ai/code-reviews](https://app.cypher.ai/code-reviews).

</details>

<details>
<summary><strong>CypherClaw</strong></summary>

<br>

Εκκινήστε τον πάντα ενεργό AI agent σας στο [app.cypher.ai/claw](https://app.cypher.ai/claw).

</details>

<details>
<summary>Εγκατάσταση CLI από GitHub Releases (binaries)</summary>

Κατεβάστε το πιο πρόσφατο binary από τη [σελίδα Releases](https://github.com/Cypher-Org/cypher/releases).

| Πλατφόρμα | Asset |
|---|---|
| Windows (οι περισσότεροι υπολογιστές) | `cypher-windows-x64.zip` |
| macOS (Apple Silicon) | `cypher-darwin-arm64.zip` |
| macOS (Intel) | `cypher-darwin-x64.zip` |
| Linux x64 | `cypher-linux-x64.tar.gz` |
| Linux ARM | `cypher-linux-arm64.tar.gz` |

Σημειώσεις: Το `x64-baseline` είναι build συμβατότητας για παλαιότερους CPU χωρίς AVX. Το `musl` είναι το στατικά συνδεδεμένο build για Alpine ή ελάχιστες Docker images χωρίς glibc. Το `cypher-vscode-*.vsix` είναι το πακέτο επέκτασης VS Code, όχι το CLI. Τα αρχεία `Source code` είναι για build από τον πηγαίο κώδικα.

</details>

### Agents

Το Cypher περιλαμβάνει εξειδικευμένους agents ανάμεσα στους οποίους αλλάζετε ανάλογα με την εργασία. Μπορείτε επίσης να δημιουργήσετε τους δικούς σας custom agents.

- **Code** - Ο προεπιλεγμένος. Υλοποιεί και επεξεργάζεται κώδικα από φυσική γλώσσα.
- **Plan** - Σχεδιάζει αρχιτεκτονική και γράφει πλάνα υλοποίησης πριν γραφτεί κώδικας.
- **Ask** - Απαντά σε ερωτήσεις για το codebase σας χωρίς να πειράζει αρχεία.
- **Debug** - Αντιμετωπίζει και εντοπίζει προβλήματα.
- **Review** - Ελέγχει τις αλλαγές σας και εντοπίζει ζητήματα απόδοσης, ασφάλειας, στυλ και κάλυψης δοκιμών.

Μάθετε περισσότερα για τους [agents και custom agents](https://cypher.ai/docs/code-with-ai/agents/using-agents).

### Τι κάνει

- **Παραγωγή κώδικα** από φυσική γλώσσα, σε πολλά αρχεία.
- **Inline autocomplete** με ghost-text προτάσεις και Tab για αποδοχή.
- **Αυτοέλεγχος** ώστε ο agent να ελέγχει και να διορθώνει τη δουλειά του.
- **Έλεγχος terminal και browser** για εκτέλεση εντολών και αυτοματοποίηση του web.
- **MCP marketplace** για εύρεση και σύνδεση MCP servers που επεκτείνουν τις δυνατότητες του agent.
- **Περισσότερα από 500 μοντέλα** με αλλαγή στη μέση της εργασίας, ώστε να ταιριάζετε latency, κόστος και reasoning στη δουλειά.

### Αυτόνομη λειτουργία (CI/CD)

Εκτελέστε `cypher run` με `--auto` για πλήρως αυτόνομη λειτουργία χωρίς prompts, σχεδιασμένη για CI/CD pipelines:

```bash
cypher run --auto "run tests and fix any failures"
```

Το `--auto` απενεργοποιεί όλα τα prompts αδειών και επιτρέπει στον agent να εκτελεί οποιαδήποτε ενέργεια χωρίς επιβεβαίωση. Χρησιμοποιήστε το μόνο σε αξιόπιστα περιβάλλοντα.

### Τεκμηρίωση

Για ρυθμίσεις και όλα τα υπόλοιπα, δείτε την [τεκμηρίωση](https://cypher.ai/docs).

### Συνεισφορά

Οι συνεισφορές είναι ευπρόσδεκτες από developers, writers και όλους. Ξεκινήστε με τον [Contributing Guide](/CONTRIBUTING.md) για ρύθμιση περιβάλλοντος, πρότυπα κώδικα και άνοιγμα pull request. Δείτε το [RELEASING.md](../RELEASING.md) για τη διαδικασία release της επέκτασης VS Code και του CLI, και το [packages/cypher-jetbrains/RELEASING.md](../packages/cypher-jetbrains/RELEASING.md) για το JetBrains plugin.

Παρακαλούμε διαβάστε τον [Code of Conduct](/CODE_OF_CONDUCT.md) πριν συμμετάσχετε.

### Άδεια

MIT. Μπορείτε να χρησιμοποιήσετε, να τροποποιήσετε και να διανείμετε αυτόν τον κώδικα, ακόμη και εμπορικά, αρκεί να διατηρήσετε τις αναφορές απόδοσης και άδειας. Δείτε [License](/LICENSE).

### FAQ

<details>
<summary>Από πού προήλθε το Cypher CLI;</summary>

Το Cypher CLI είναι fork του [OpenCode](https://github.com/anomalyco/opencode), βελτιωμένο για να λειτουργεί μέσα στην Cypher agentic engineering platform.

</details>

---

**Γίνετε μέλος της κοινότητας** [Discord](https://cypher.ai/discord) | [X](https://x.com/cypher) | [Reddit](https://www.reddit.com/r/cypher/)
