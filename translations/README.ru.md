<p align="center">
  <a href="../README.md">English</a> | <a href="README.zh.md">简体中文</a> | <a href="README.zht.md">繁體中文</a> | <a href="README.ko.md">한국어</a> | <a href="README.de.md">Deutsch</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.it.md">Italiano</a> | <a href="README.da.md">Dansk</a> | <a href="README.ja.md">日本語</a> | <a href="README.pl.md">Polski</a> | Русский | <a href="README.bs.md">Bosanski</a> | <a href="README.ar.md">العربية</a> | <a href="README.no.md">Norsk</a> | <a href="README.br.md">Português (Brasil)</a> | <a href="README.th.md">ไทย</a> | <a href="README.tr.md">Türkçe</a> | <a href="README.uk.md">Українська</a> | <a href="README.bn.md">বাংলা</a> | <a href="README.gr.md">Ελληνικά</a> | <a href="README.vi.md">Tiếng Việt</a>
</p>

<p align="center">
  <a href="https://cypher.ai"><img width="250" alt="Cypher CLI logo" src="https://github.com/user-attachments/assets/bdb0c174-b9fd-40ad-a47b-f3aab9b54e8d" /></a>
</p>

<p align="center">Open source-агент для разработки с ИИ в VS Code, JetBrains или CLI.</p>

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

Cypher CLI — это AI-агент для написания кода, который работает там, где работаете вы: в [VS Code](https://cypher.ai/landing/vs-code), [JetBrains](https://cypher.ai/features/jetbrains-native) и [CLI](https://cypher.ai/cli). Он имеет открытый исходный код и открытую модель ценообразования. Вы выбираете из более чем 500 моделей, переключаетесь между ними во время задачи и платите по тарифу поставщика модели без наценки. Для начала не нужны API-ключи.

### Установка

Выберите, где вы хотите запускать Cypher.

<details open>
<summary><strong>VS Code</strong></summary>

<br>

Установите [расширение Cypher CLI](vscode:extension/cypher.cypher-cli) напрямую или скачайте его из [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=cypher.Cypher-Code). Создайте аккаунт и получите доступ к более чем 500 моделям, включая GPT-5.5, Claude Opus 4.7, Claude Sonnet 4.6 и Gemini 3.1 Pro Preview, все по ценам поставщиков.

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

Затем запустите `cypher` в любом каталоге проекта.

</details>

<details>
<summary><strong>JetBrains</strong></summary>

<br>

Установите [плагин Cypher CLI](https://plugins.jetbrains.com/plugin/28350-cypher-cli) из JetBrains Marketplace или найдите "Cypher CLI" в `Settings → Plugins` в любой IDE JetBrains.

</details>

<details>
<summary><strong>Cloud Agent</strong></summary>

<br>

Запускайте Cypher из веба без локальной машины на [app.cypher.ai/cloud](https://app.cypher.ai/cloud).

</details>

<details>
<summary><strong>Code Reviews</strong></summary>

<br>

Настройте автоматические AI-ревью кода для ваших pull request на [app.cypher.ai/code-reviews](https://app.cypher.ai/code-reviews).

</details>

<details>
<summary><strong>CypherClaw</strong></summary>

<br>

Запустите своего постоянно активного AI-агента на [app.cypher.ai/claw](https://app.cypher.ai/claw).

</details>

<details>
<summary>Установить CLI из GitHub Releases (бинарные файлы)</summary>

Скачайте последний бинарный файл со [страницы Releases](https://github.com/Cypher-Org/cypher/releases).

| Платформа | Файл |
|---|---|
| Windows (большинство ПК) | `cypher-windows-x64.zip` |
| macOS (Apple Silicon) | `cypher-darwin-arm64.zip` |
| macOS (Intel) | `cypher-darwin-x64.zip` |
| Linux x64 | `cypher-linux-x64.tar.gz` |
| Linux ARM | `cypher-linux-arm64.tar.gz` |

Примечания: `x64-baseline` — совместимая сборка для старых CPU без AVX. `musl` — статически связанная сборка для Alpine или минимальных Docker-образов без glibc. `cypher-vscode-*.vsix` — пакет расширения VS Code, а не CLI. Архивы `Source code` предназначены для сборки из исходного кода.

</details>

### Agents

Cypher поставляется со специализированными agents, между которыми можно переключаться в зависимости от задачи. Вы также можете создавать собственные agents.

- **Code** - По умолчанию. Реализует и редактирует код по описанию на естественном языке.
- **Plan** - Проектирует архитектуру и пишет планы реализации до написания кода.
- **Ask** - Отвечает на вопросы о кодовой базе, не изменяя файлы.
- **Debug** - Диагностирует и отслеживает проблемы.
- **Review** - Проверяет ваши изменения и выявляет проблемы производительности, безопасности, стиля и покрытия тестами.

Подробнее об [agents и пользовательских agents](https://cypher.ai/docs/code-with-ai/agents/using-agents).

### Возможности

- **Генерация кода** из естественного языка в нескольких файлах.
- **Встроенное автодополнение** с ghost-text-подсказками и принятием по Tab.
- **Самопроверка**, чтобы агент проверял и исправлял собственную работу.
- **Управление терминалом и браузером** для запуска команд и автоматизации веба.
- **MCP marketplace** для поиска и подключения MCP-серверов, расширяющих возможности агента.
- **Более 500 моделей** с переключением во время задачи, чтобы подобрать задержку, стоимость и reasoning под работу.

### Автономный режим (CI/CD)

Запустите `cypher run` с `--auto` для полностью автономной работы без prompts, предназначенной для CI/CD-пайплайнов:

```bash
cypher run --auto "run tests and fix any failures"
```

`--auto` отключает все запросы разрешений и позволяет агенту выполнять любые действия без подтверждения. Используйте только в доверенных средах.

### Документация

Для настройки и всего остального перейдите к [документации](https://cypher.ai/docs).

### Участие

Мы приветствуем вклад разработчиков, авторов и всех желающих. Начните с [Contributing Guide](/CONTRIBUTING.md), чтобы настроить окружение, изучить стандарты кода и узнать, как открыть pull request. См. [RELEASING.md](../RELEASING.md) для процесса релиза расширения VS Code и CLI, а также [packages/cypher-jetbrains/RELEASING.md](../packages/cypher-jetbrains/RELEASING.md) для плагина JetBrains.

Перед участием ознакомьтесь с нашим [Code of Conduct](/CODE_OF_CONDUCT.md).

### Лицензия

MIT. Вы можете использовать, изменять и распространять этот код, в том числе коммерчески, если сохраняете указания авторства и лицензионные уведомления. См. [License](/LICENSE).

### FAQ

<details>
<summary>Откуда появился Cypher CLI?</summary>

Cypher CLI — это fork [OpenCode](https://github.com/anomalyco/opencode), расширенный для работы в платформе agentic engineering Cypher.

</details>

---

**Присоединяйтесь к сообществу** [Discord](https://cypher.ai/discord) | [X](https://x.com/cypher) | [Reddit](https://www.reddit.com/r/cypher/)
