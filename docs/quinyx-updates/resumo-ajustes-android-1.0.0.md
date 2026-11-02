# QuinyxDesk Android — ícone flutuante, textos pt-BR e versão 1.0.0

Data: 30/10/2026 · Branch: `feat/launcher-board-desk-packages`

## Commits

| Commit | Resumo |
|---|---|
| `5ee33d957` | Novo ícone flutuante, menu em pt-BR e remoção das referências restantes ao RustDesk |
| `e167497e2` | Versão 1.0.0, ícone com o símbolo do app recortado na borda e novo aviso antes de compartilhar a tela |

Nada foi enviado ao `origin` (sem push).

## 1. Ícone flutuante (serviço ativo)

Antes ele tinha o logo do RustDesk, aparecia cortado ao meio na borda e parecia em baixa resolução.

- **Visual:** círculo azul da marca (`#04A2FD`) com borda branca e o símbolo do ícone do app em branco (monitor + Q + cursor). O vetor veio de `res/scalable.svg`, o arquivo oficial.
  - Arquivo: `flutter/android/app/src/main/res/drawable/floating_window.xml`
- **Encaixe na borda:** fica 75% visível e o restante é cortado pela borda. A janela tem 75% da largura do ícone, e o ícone é deslocado com um `Matrix` no `ImageView`. Esse recorte não depende da janela sair da tela, coisa que muda de uma versão do Android para outra.
- **Tamanho padrão:** 48dp, calculado pela densidade da tela. Antes eram 120px fixos. A opção `floating-window-size` ainda tem prioridade.
- O código antigo que fatiava o bitmap em metades foi removido.
  - Arquivo: `flutter/android/app/src/main/kotlin/com/carriez/flutter_hbb/FloatingWindowService.kt`

## 2. Menu do ícone flutuante

Os textos agora são fixos em pt-BR, independentemente do idioma do aparelho.

| Antes | Agora |
|---|---|
| Show QuinyxDesk | Abrir QuinyxDesk |
| Atualizar a área de transferência do cliente | Enviar área de transferência |
| Parar serviço | Encerrar acesso remoto |

"Enviar área de transferência" só aparece quando há um cliente conectado. Esse comportamento é o mesmo de antes.

## 3. Referências ao "RustDesk" que ainda apareciam

Os textos que passam por `translate()` já trocavam "RustDesk" por "QuinyxDesk" automaticamente (`src/lang.rs`). Os pontos abaixo não passavam por ali e foram corrigidos:

| Onde | Antes | Agora |
|---|---|---|
| Título da notificação do serviço | RustDesk | QuinyxDesk |
| Canal de notificação (configurações do Android) | RustDesk Service | Serviço QuinyxDesk |
| Toast ao iniciar com o aparelho | RustDesk is Open | QuinyxDesk iniciado |
| Descrição do serviço de acessibilidade | texto em inglês citando RustDesk | texto em pt-BR citando QuinyxDesk |
| Configurações › Sobre (mobile) | link "rustdesk.com" e "Privacy Statement" (site do RustDesk) | removidos |

- O ID interno do canal de notificação ficou igual. Assim o canal não é duplicado nos aparelhos que já têm o app.
- O diálogo `showAbout`, que não era usado, foi removido.

## 4. Aviso antes de iniciar o serviço

O cartão rosa em gradiente com o carimbo "SCAM" e o texto alarmista foi substituído por um diálogo no tema do app:

- escudo azul e o título "Compartilhar a tela deste aparelho";
- texto explicando que quem tiver o ID e a senha poderá ver a tela e, se permitido, controlar o aparelho;
- orientação para continuar só se a pessoa conhecer e confiar em quem vai acessar;
- "Não mostrar novamente" e os botões **Cancelar** e **Continuar**.

A lógica não mudou: mesma contagem regressiva, mesma opção `show-scam-warning` e mesmo `toggleService`.

- Arquivo: `flutter/lib/mobile/pages/server_page.dart`

## 5. Versão

| Item | Valor |
|---|---|
| Versão do app (`flutter/pubspec.yaml`) | **1.0.0+67** |
| Versão base do RustDesk (`Cargo.toml`) | **1.4.8** (não alterada) |
| `VERSION` nos workflows `meu-android/linux/windows.yml` | 1.0.0 (nomes dos artefatos) |

- O `versionCode` foi de 66 para 67 para que a atualização instale por cima da 1.4.8 que já está nas lousas.
- A versão do `Cargo.toml` não pode acompanhar a do app. Ela é enviada aos outros aparelhos na conexão, e eles ligam ou desligam recursos pelo número (área de transferência, transferência de arquivos, telas virtuais etc.).
- O `README.md` ganhou no topo uma tabela com as duas versões e essa explicação.
- Pendência: no Windows e no Linux, a tela "Sobre" ainda mostra 1.4.8, porque lá a versão vem do Rust.

## 6. Build e testes

**Como o APK de teste foi gerado (debug, x86_64):**

- A biblioteca nativa `librustdesk.so` foi reaproveitada do APK instalado na lousa (build de 22/10). O código Rust não mudou desde então.
- O binding Flutter ↔ Rust (`generated_bridge.dart`) foi gerado num container Docker, com as mesmas versões do CI (Rust 1.75, `flutter_rust_bridge_codegen` 1.80.1, Flutter 3.22.3).
- O APK foi compilado no Windows com o Flutter 3.24.5 (instalado em `D:\Programs\flutter-3.24.5`, com o patch do CI aplicado) e o JDK 17.
- Arquivo gerado: `flutter/build/app/outputs/flutter-apk/app-debug.apk`

**Limites desse APK:**

- é assinado com a chave de debug;
- usa o `REMOTE_SESSION_TOKEN` de desenvolvimento;
- só roda em x86_64.

**O APK de produção deve sair pelo workflow "Meu Build Android".**

**Verificado no emulador (Android 14):**

- o ícone fica recortado nas bordas esquerda e direita;
- arrastar e tocar funcionam;
- o menu aparece em pt-BR e "Abrir QuinyxDesk" abre o app;
- a notificação e o canal mostram "QuinyxDesk";
- o novo aviso aparece ao iniciar o serviço;
- a versão aparece como 1.0.0.

**Ambiente:**

- Foi criada a AVD `QuinyxDesk_Test` para não interferir na sessão que testava o Quinyx Office no `emulator-5554`.
- Depois, o APK final também foi instalado no `emulator-5554`, a pedido.
- Os arquivos locais de build (`jniLibs/x86_64` e `generated_bridge*.dart`) são ignorados pelo git.
- As configurações globais alteradas durante o build foram revertidas: `flutter config --jdk-dir` e `pubspec.lock`.

## Pendências e sugestões

- Gerar o APK de release pelo CI e validar em uma lousa real.
- Decidir se o desktop (Windows/Linux) também deve mostrar 1.0.0 na tela "Sobre".
- Definir o site e a política de privacidade da Quinyx para recolocar os links em "Sobre", se for o caso.
- Apagar a AVD `QuinyxDesk_Test` se ela não for mais usada.
