<p align="center">
  <img src="res/logo-header.svg" alt="QuinyxDesk">
</p>

# QuinyxDesk

Acesso remoto da Quinyx Company para lousas, mesas digitais e computadores. É um fork do [RustDesk](https://github.com/rustdesk/rustdesk) que se conecta somente ao servidor próprio da Quinyx.

| | Versão |
|---|---|
| QuinyxDesk (app) | **1.0.0** |
| Baseado no RustDesk | **1.4.8** |

> [!IMPORTANT]
> A versão do app fica em `flutter/pubspec.yaml` (`version`). A versão em `Cargo.toml` (1.4.8) **não deve ser alterada** junto: ela é enviada aos outros aparelhos na conexão e libera recursos conforme o número (área de transferência, transferência de arquivos, telas virtuais etc.). Ela só muda ao incorporar uma nova versão do RustDesk.

## Diferenças em relação ao RustDesk

- **Servidor fixo.** O servidor, a chave pública e a senha mestra são gravados no app durante o build (`option_env!` em [config.rs](libs/hbb_common/src/config.rs)). O usuário não consegue trocar o servidor nem a senha.
- **Sem conta nem login.** A opção de Login/Conta está oculta em todas as plataformas, porque o servidor é o hbbs/hbbr open source, que não tem API.
- **Sem atualização automática.** O atualizador do RustDesk baixaria a versão oficial por cima do QuinyxDesk, por isso está desligado. Atualizações saem pelos workflows deste repositório.
- **Marca Quinyx.** Nome, ícones e textos foram trocados, incluindo o crédito "Alterado por Quinyx Company" e os links para [quinyxcompany.com](https://quinyxcompany.com).
- **Integração com os launchers (Android).** O app envia o ID de acesso aos launchers da Quinyx (veja [Ponte com os launchers](#ponte-com-os-launchers-android)).
- **Pacotes com nome próprio.** O pacote Android é `com.quinyx.rustdesk`. No Linux, o binário, o serviço e o `.deb` se chamam `quinyxdesk`.

## Plataformas

| Plataforma | Workflow | Artefato |
|---|---|---|
| Android | **Meu Build Android** ([meu-android.yml](.github/workflows/meu-android.yml)) | `rustdesk-1.0.0-<arch>.apk` (arm64-v8a, armeabi-v7a, x86_64) |
| Windows | **Meu Build Windows** ([meu-windows.yml](.github/workflows/meu-windows.yml)) | `.exe` portátil e/ou `.msi` |
| Linux | **Meu Build Linux** ([meu-linux.yml](.github/workflows/meu-linux.yml)) | `quinyxdesk-1.0.0-ubuntu26.04-amd64.deb` |

Os builds de produção saem **somente** por esses workflows, executados manualmente em *Actions › Run workflow*. Os demais workflows (`flutter-*.yml`, `ci.yml` etc.) vieram do RustDesk e não são usados.

## Secrets do build

Configurados em *Settings › Secrets and variables › Actions* do repositório. Para builds locais, os mesmos valores ficam em `.env`, que está no `.gitignore` e **nunca deve ser commitado**.

| Secret | Uso | Obrigatório |
|---|---|---|
| `RENDEZVOUS_SERVER` | Endereço do servidor hbbs | Sim |
| `RS_PUB_KEY` | Chave pública do servidor | Sim |
| `PRESET_PASSWORD` | Senha mestra aceita por todos os aparelhos | Sim |
| `API_SERVER` | API HTTP do RustDesk Server Pro. **Deixe sem definir**: o servidor open source não tem API, e sem ela o app desliga heartbeat, sysinfo e auditoria | Não |
| `REMOTE_SESSION_TOKEN` | Token da ponte com os launchers. Deve ser igual ao usado no build dos launchers | Android (release) |
| `KEYSTORE_BASE64`, `KEYSTORE_PASSWORD`, `KEY_PASSWORD`, `KEY_ALIAS` | Assinatura do APK | Android |
| `ADI_REGISTRATION_SNIPPET` | Arquivo de verificação de desenvolvedor do Android | Android |

> [!WARNING]
> Sem `RENDEZVOUS_SERVER` ou `RS_PUB_KEY`, o código cairia no servidor público do RustDesk, mantendo a senha mestra. Por isso todo workflow começa pelo job `check-secrets`, que interrompe o build se algum deles faltar.

## Ponte com os launchers (Android)

[LauncherBridge.kt](flutter/android/app/src/main/kotlin/com/carriez/flutter_hbb/LauncherBridge.kt) envia o ID de acesso por broadcast para:

| Pacote | Launcher |
|---|---|
| `br.com.quinyx.launcher` | Mesinha |
| `com.quinyx.launcher.board` | Lousa |
| `com.quinyx.launcher.desk` | Mesa Digital |

- Action: `br.com.quinyx.launcher.action.REMOTE_ID`
- Extras: `id`, `token` (o `REMOTE_SESSION_TOKEN`) e `sender_package`

O contrato precisa bater com o `QuinyxDeskBridgeReceiver` dos launchers. Um build de release sem `REMOTE_SESSION_TOKEN` é abortado pelo Gradle, para que o APK nunca saia com o token público de desenvolvimento.

## Build local

O caminho recomendado é o CI. Localmente, as versões que precisam bater com ele são:

| Ferramenta | Versão |
|---|---|
| Rust | 1.75 |
| Flutter | 3.24.5, com o patch `.github/patches/flutter_3.24.4_dropdown_menu_enableFilter.diff` |
| `flutter_rust_bridge_codegen` | 1.80.1 |
| JDK (Android) | 17 |
| NDK (Android) | r28c |

Passo a passo de alto nível:

1. Gere o binding Flutter ↔ Rust (`flutter/lib/generated_bridge.dart`) como em [bridge.yml](.github/workflows/bridge.yml).
2. Compile a biblioteca Rust com os secrets do `.env` no ambiente (Android: scripts `flutter/ndk_*.sh`, via `cargo ndk`; desktop: `python3 build.py --flutter --hwcodec`, com as flags de cada workflow).
3. Compile o app Flutter (Android: `flutter build apk`; Windows/Linux: o `build.py` já faz isso).

As dependências de sistema (vcpkg, libvpx, libyuv, opus, aom e pacotes do Linux) são as mesmas do RustDesk e estão listadas nos workflows e na [documentação de build do RustDesk](https://rustdesk.com/docs/en/dev/build/).

## Estrutura

| Caminho | Conteúdo |
|---|---|
| [src/](src/) | App em Rust: conexão, servidor de captura e entrada, plataformas |
| [src/lang/](src/lang/) | Traduções (`ptbr.rs` é o idioma principal) |
| [libs/hbb_common/](libs/hbb_common/) | Configuração (servidor e senha embutidos), protocolo e utilitários |
| [libs/scrap/](libs/scrap/) | Captura de tela e codecs de vídeo |
| [libs/enigo/](libs/enigo/) | Controle de teclado e mouse |
| [flutter/lib/](flutter/lib/) | Interface (`desktop/`, `mobile/`, `common/`, `models/`) |
| [flutter/android/](flutter/android/) | Código nativo Android: serviço de captura, ícone flutuante, ponte com os launchers |
| [res/](res/) | Ícones, serviço systemd, `.desktop` e scripts do pacote Debian |
| [build.py](build.py) | Build e empacotamento do desktop |
| [docs/](docs/) | Revisões de código e notas das versões Quinyx |

Regras de código para quem edita o projeto (inclusive agentes de IA): [AGENTS.md](AGENTS.md).

## Licença

Este projeto é derivado do RustDesk e distribuído sob a [GNU AGPL v3](LICENCE). Quem distribuir o app ou oferecê-lo como serviço deve disponibilizar o código-fonte correspondente.

> [!CAUTION]
> Use o QuinyxDesk somente para acessar aparelhos com autorização de quem os utiliza. Acesso não autorizado ou invasão de privacidade não são permitidos.
