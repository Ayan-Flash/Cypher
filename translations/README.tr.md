<p align="center">
  <a href="../README.md">English</a> | <a href="README.zh.md">简体中文</a> | <a href="README.zht.md">繁體中文</a> | <a href="README.ko.md">한국어</a> | <a href="README.de.md">Deutsch</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.it.md">Italiano</a> | <a href="README.da.md">Dansk</a> | <a href="README.ja.md">日本語</a> | <a href="README.pl.md">Polski</a> | <a href="README.ru.md">Русский</a> | <a href="README.bs.md">Bosanski</a> | <a href="README.ar.md">العربية</a> | <a href="README.no.md">Norsk</a> | <a href="README.br.md">Português (Brasil)</a> | <a href="README.th.md">ไทย</a> | Türkçe | <a href="README.uk.md">Українська</a> | <a href="README.bn.md">বাংলা</a> | <a href="README.gr.md">Ελληνικά</a> | <a href="README.vi.md">Tiếng Việt</a>
</p>

<p align="center">
  <a href="https://cypher.ai"><img width="250" alt="Cypher CLI logo" src="https://github.com/user-attachments/assets/bdb0c174-b9fd-40ad-a47b-f3aab9b54e8d" /></a>
</p>

<p align="center">VS Code, JetBrains veya CLI'de AI ile geliştirme yapmak için açık kaynak kodlama ajanı.</p>

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

Cypher CLI, çalıştığınız her yerde size eşlik eden bir AI kodlama ajanıdır: [VS Code](https://cypher.ai/landing/vs-code), [JetBrains](https://cypher.ai/features/jetbrains-native) ve [CLI](https://cypher.ai/cli). Açık kaynaktır ve açık fiyatlandırma sunar. 500'den fazla model arasından seçim yapabilir, görev sırasında model değiştirebilir ve hiçbir ek ücret olmadan model sağlayıcısının fiyatını ödersiniz. Başlamak için API anahtarı gerekmez.

### Kurulum

Cypher'yu nerede çalıştırmak istediğinizi seçin.

<details open>
<summary><strong>VS Code</strong></summary>

<br>

[Cypher CLI uzantısını](vscode:extension/cypher.cypher-cli) doğrudan kurun veya [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=cypher.Cypher-Code) üzerinden edinin. Bir hesap oluşturduğunuzda GPT-5.5, Claude Opus 4.7, Claude Sonnet 4.6 ve Gemini 3.1 Pro Preview dahil 500'den fazla modele sağlayıcı fiyatıyla erişebilirsiniz.

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

Ardından başlamak için herhangi bir proje dizininde `cypher` çalıştırın.

</details>

<details>
<summary><strong>JetBrains</strong></summary>

<br>

[Cypher CLI eklentisini](https://plugins.jetbrains.com/plugin/28350-cypher-cli) JetBrains Marketplace'ten kurun veya herhangi bir JetBrains IDE içinde `Settings → Plugins` bölümünde "Cypher CLI" arayın.

</details>

<details>
<summary><strong>Cloud Agent</strong></summary>

<br>

Cypher'yu yerel makine gerekmeden web üzerinden [app.cypher.ai/cloud](https://app.cypher.ai/cloud) adresinde çalıştırın.

</details>

<details>
<summary><strong>Kod İncelemeleri</strong></summary>

<br>

Pull request'leriniz için otomatik AI kod incelemelerini [app.cypher.ai/code-reviews](https://app.cypher.ai/code-reviews) adresinde ayarlayın.

</details>

<details>
<summary><strong>CypherClaw</strong></summary>

<br>

Her zaman açık AI ajanınızı [app.cypher.ai/claw](https://app.cypher.ai/claw) adresinde başlatın.

</details>

<details>
<summary>CLI'yi GitHub Releases üzerinden kurun (ikili dosyalar)</summary>

En son ikili dosyayı [Releases sayfasından](https://github.com/Cypher-Org/cypher/releases) indirin.

| Platform | Asset |
|---|---|
| Windows (çoğu PC) | `cypher-windows-x64.zip` |
| macOS (Apple Silicon) | `cypher-darwin-arm64.zip` |
| macOS (Intel) | `cypher-darwin-x64.zip` |
| Linux x64 | `cypher-linux-x64.tar.gz` |
| Linux ARM | `cypher-linux-arm64.tar.gz` |

Notlar: `x64-baseline`, AVX olmayan eski CPU'lar için uyumluluk derlemesidir. `musl`, Alpine veya glibc olmayan minimal Docker imajları için statik bağlı derlemedir. `cypher-vscode-*.vsix` CLI değil VS Code uzantı paketidir. `Source code` arşivleri kaynaktan derlemek içindir.

</details>

### Agents

Cypher, göreve göre aralarında geçiş yapabileceğiniz özelleşmiş agents ile gelir. Kendi özel agents'larınızı da oluşturabilirsiniz.

- **Code** - Varsayılan. Doğal dilden kod uygular ve düzenler.
- **Plan** - Kod yazılmadan önce mimari tasarlar ve uygulama planları yazar.
- **Ask** - Dosyalara dokunmadan kod tabanınız hakkında soruları yanıtlar.
- **Debug** - Sorunları giderir ve izler.
- **Review** - Değişikliklerinizi inceler ve performans, güvenlik, stil ve test kapsamı sorunlarını ortaya çıkarır.

[Agents ve özel agents](https://cypher.ai/docs/code-with-ai/agents/using-agents) hakkında daha fazla bilgi edinin.

### Ne yapar

- Birden çok dosyada doğal dilden **kod üretimi**.
- Ghost-text önerileri ve kabul etmek için Tab ile **satır içi otomatik tamamlama**.
- Ajanın kendi çalışmasını inceleyip düzeltmesi için **öz denetim**.
- Komut çalıştırmak ve web'i otomatikleştirmek için **terminal ve tarayıcı kontrolü**.
- Ajanın yapabileceklerini genişleten MCP sunucularını bulmak ve bağlamak için **MCP marketplace**.
- Gecikme, maliyet ve akıl yürütmeyi işe uygun seçmek için görev sırasında geçiş destekli **500'den fazla model**.

### Otonom Mod (CI/CD)

CI/CD pipeline'ları için prompts olmadan tamamen otonom çalıştırmak üzere `cypher run` komutunu `--auto` ile çalıştırın:

```bash
cypher run --auto "run tests and fix any failures"
```

`--auto` tüm izin istemlerini devre dışı bırakır ve ajanın herhangi bir işlemi onay olmadan yürütmesine izin verir. Yalnızca güvenilir ortamlarda kullanın.

### Dokümantasyon

Yapılandırma ve diğer her şey için [dokümantasyona](https://cypher.ai/docs) bakın.

### Katkıda bulunma

Geliştiricilerden, yazarlardan ve herkesten katkı bekliyoruz. Ortam kurulumu, kodlama standartları ve pull request açma hakkında bilgi için [Contributing Guide](/CONTRIBUTING.md) ile başlayın. VS Code uzantısı ve CLI yayın süreci için [RELEASING.md](../RELEASING.md), JetBrains eklentisi için [packages/cypher-jetbrains/RELEASING.md](../packages/cypher-jetbrains/RELEASING.md) dosyasına bakın.

Katılmadan önce lütfen [Code of Conduct](/CODE_OF_CONDUCT.md) belgemizi okuyun.

### Lisans

MIT. Atıf ve lisans bildirimlerini koruduğunuz sürece bu kodu ticari olarak da kullanabilir, değiştirebilir ve dağıtabilirsiniz. Bkz. [License](/LICENSE).

### FAQ

<details>
<summary>Cypher CLI nereden geldi?</summary>

Cypher CLI, Cypher agentic engineering platformunda çalışacak şekilde geliştirilmiş bir [OpenCode](https://github.com/anomalyco/opencode) fork'udur.

</details>

---

**Topluluğa katılın** [Discord](https://cypher.ai/discord) | [X](https://x.com/cypher) | [Reddit](https://www.reddit.com/r/cypher/)
