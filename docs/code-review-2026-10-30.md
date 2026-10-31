# Revisão de Código — QuinyxDesk

**Data:** 2026-10-30
**Escopo revisado:** todas as customizações do fork sobre o RustDesk 1.4.8 (base `36e812e55`), num total de 45 commits. Revisei o servidor, a chave e a senha embutidos via CI, a senha mestra, a ponte com o Launcher Quinyx, o ícone flutuante, o aviso de compartilhamento, o rebranding e os workflows `meu-android/linux/windows.yml`. O `libs/hbb_common` vendorizado foi comparado com o upstream (`387603f`): só `config.rs` e `build.rs` mudaram. Também revisei os pontos do RustDesk original que o rebranding ou o uso de servidor próprio afetam: atualizador, API/heartbeat, nomes de serviço no Linux e empacotamento no Windows.
**Fora do escopo:** o restante do código original do RustDesk, sem alterações (protocolo, captura, input e transferência de arquivos); o Launcher Quinyx; e o próprio servidor (hbbs/hbbr), que não está neste repositório.
**Stack:** Rust 1.75 (núcleo), Flutter 3.24 / Dart (UI), Kotlin (Android), GitHub Actions (build).

## Resumo executivo

O núcleo das customizações está bem feito. O servidor, a chave e o método de autenticação ficam travados em `OVERWRITE_SETTINGS`, o que impede que o usuário os altere. A senha mestra reaproveita o caminho de validação do upstream, sem criar um atalho novo. O token da ponte com o launcher faz o build de release falhar se não estiver configurado. No Android, que é a prioridade, não encontrei nenhum bug que impeça a conexão com o servidor próprio.

Os riscos principais aparecem em três lugares:

- **No build:** se o secret `RENDEZVOUS_SERVER` estiver ausente, o app passa a usar, sem nenhum aviso, o servidor público do RustDesk, mas mantém a senha mestra embutida.
- **No Linux e no Windows:** o rebranding ficou pela metade. No Linux, o código procura o serviço e os processos como `quinyxdesk`, mas o `.deb` os instala como `rustdesk`. No Windows, o `.exe` autoextraível aponta para `rustdesk.exe`, um arquivo que não existe mais.
- **No atualizador do Windows:** se alguém marcar "Auto update", o app baixa e instala o **RustDesk oficial**, que usa o servidor público.

A recomendação principal é fazer o CI falhar quando os secrets do servidor estiverem ausentes e alinhar os nomes no Linux e no Windows antes de distribuir essas duas versões. Também é preciso confirmar se o servidor em uso é o **open source** ou o **Pro**, porque isso define a correção do M-01.

## Panorama por severidade

| Severidade | Quantidade |
|------------|-----------|
| 🔴 Crítico | 0 |
| 🟠 Alto    | 5 |
| 🟡 Médio   | 3 |
| 🟢 Baixo   | 4 |

## Achados

### 🟠 [A-01] Sem o secret do servidor, o build usa o servidor público do RustDesk e mantém a senha mestra

- **Local:** `libs/hbb_common/src/config.rs:181-188`, `libs/hbb_common/src/config.rs:100-121`, `.github/workflows/meu-android.yml:28-31` (o mesmo vale para `meu-linux.yml:24-27` e `meu-windows.yml:32-35`)
- **Categoria:** Confiabilidade / Segurança
- **Problema:** `RENDEZVOUS_SERVERS` e `RS_PUB_KEY` usam `rs-ny.rustdesk.com` e a chave pública oficial quando o secret está vazio. `baked_env` trata um valor vazio como ausente. Os workflows repassam `${{ secrets.X }}`, que vira `""` quando o secret não existe (por exemplo, num fork, num repositório novo ou depois de um secret ser renomeado), e o build termina sem erro.
- **Impacto:** o APK, o MSI ou o DEB gerado registra os aparelhos no servidor público do RustDesk, e o `PRESET_PASSWORD` continua embutido como senha mestra. As lousas somem do servidor próprio sem nenhum erro aparente, ficam acessíveis pela rede pública e com uma senha conhecida.
- **Correção sugerida:** fazer o build falhar, do mesmo jeito que o `build.gradle` já faz com o `REMOTE_SESSION_TOKEN`. Adicionar um passo no início de cada workflow:
  ```yaml
  - name: Validar secrets do servidor
    shell: bash
    run: |
      for v in RENDEZVOUS_SERVER RS_PUB_KEY; do
        [ -n "${!v}" ] || { echo "::error::$v não definido"; exit 1; }
      done
  ```
  Se quiser uma proteção também no código, troque o fallback de `config.rs:181-188` por `compile_error!` quando a variável estiver vazia. Isso só vale para builds de release, para não quebrar o build local de desenvolvimento.

### 🟠 [A-02] Linux: serviço e processos procurados como `quinyxdesk`, mas o `.deb` instala `rustdesk`

- **Local:** `src/platform/linux.rs:698-716` (`stop_rustdesk_servers`, `stop_subprocess`), `src/platform/linux.rs:2059-2105` (`install_service`, `uninstall_service`, `check_if_stop_service`), `src/platform/linux_desktop_manager.rs:739-802`, `res/DEBIAN/postinst:8-25`, `.github/workflows/meu-linux.yml:179-181`
- **Categoria:** Bug
- **Problema:** `APP_NAME` agora é `QuinyxDesk`, e o código Linux monta os nomes com `get_app_name().to_lowercase()`. O resultado são comandos como `systemctl enable/stop quinyxdesk`, `grep -E 'quinyxdesk +--server'`, `/etc/quinyxdesk/xorg.conf` e `/etc/pam.d/quinyxdesk`. Mas o `build.py` e o `res/DEBIAN/postinst` continuam gerando o pacote `rustdesk`, o binário `/usr/bin/rustdesk` e o `rustdesk.service`.
- **Impacto:** a instalação funciona, porque é o `postinst` que habilita o `rustdesk.service`. Depois disso, porém:
  - os botões de iniciar e parar o serviço na interface falham (`Unit quinyxdesk.service not found`);
  - processos `--server` antigos não são encerrados ao trocar de sessão, de usuário ou de display, o que pode deixar servidores duplicados;
  - a sessão headless (`xorg.conf`/`startwm.sh`) não encontra os arquivos.

  O pacote também conflita com o RustDesk oficial, se ele estiver instalado na mesma máquina.
- **Correção sugerida:** escolher um nome único. A opção mais limpa é renomear o pacote Linux para `quinyxdesk` (nome do pacote, binário, `quinyxdesk.service`, `/usr/share/quinyxdesk` e `/etc/quinyxdesk`), ajustando o `build.py`, o `res/DEBIAN/*`, o `res/rustdesk.service` e os `.desktop`. Se o Linux não for usado em produção agora, registrar isso como pendência e não distribuir o `.deb`.

### 🟠 [A-03] Windows: o `.exe` autoextraível tenta iniciar `rustdesk.exe`, que não existe

- **Local:** `.github/workflows/meu-windows.yml:171`, `flutter/windows/CMakeLists.txt:7`, `libs/portable/src/main.rs:98`
- **Categoria:** Bug
- **Problema:** o binário passou a se chamar `QuinyxDesk.exe` (`BINARY_NAME`), mas o empacotador ainda recebe `-e ../../rustdesk/rustdesk.exe`. O `generate.py` não confere se o arquivo existe e grava `./rustdesk.exe` nos metadados. Na execução, o `main.rs:98` faz `dir.join(reader.exe)`, e o `Command::new` falha.
- **Impacto:** o artefato `rustdesk-1.0.0-x86_64.exe` extrai os arquivos e não abre nada. O `.msi` não é afetado, porque o `preprocess.py` já usa `--app-name QuinyxDesk`.
- **Correção sugerida:** em `meu-windows.yml:171`, trocar para `-e ../../rustdesk/QuinyxDesk.exe`. O `build.py:450` já usa esse nome no fluxo local.

### 🟠 [A-04] Windows: "Auto update" instala o RustDesk oficial no lugar do QuinyxDesk

- **Local:** `flutter/lib/desktop/pages/desktop_setting_page.dart:485` (`showAutoUpdate`), `src/rendezvous_mediator.rs:114-116`, `src/updater.rs:120-147`, `src/common.rs:955-990`
- **Categoria:** Bug / Segurança
- **Problema:** com o app instalado no Windows, a caixa "Auto update" aparece em Configurações › Geral. Ela não depende de `isCustomClient`, ao contrário de "Check for software update", que fica oculta. Quando marcada, o `check_update` consulta `api.rustdesk.com` com a versão `1.4.8` (a do `Cargo.toml`) e, se houver uma versão mais nova do RustDesk, baixa `github.com/rustdesk/rustdesk/releases/download/<v>/rustdesk-<v>-x86_64.exe` e o executa com `--update`.
- **Impacto:** quem tiver acesso às configurações pode, sem perceber, trocar o QuinyxDesk pelo RustDesk oficial, que usa o servidor público, não tem a senha mestra e não tem a marca. O padrão é desligado (`allow-` exige `"Y"`), então só acontece se alguém marcar a opção.
- **Correção sugerida:** desligar o atualizador para este cliente customizado nas duas camadas:
  ```dart
  final showAutoUpdate = isWindows && bind.mainIsInstalled() && !bind.isCustomClient();
  ```
  ```rust
  // src/updater.rs, início de check_update
  if crate::is_custom_client() {
      return Ok(());
  }
  ```

### 🟠 [A-05] A senha mestra pode ser extraída do binário e vale para todos os aparelhos

- **Local:** `libs/hbb_common/src/config.rs:123-131`, `src/server/connection.rs:2208-2218`, `src/server/connection.rs:2078-2087`
- **Categoria:** Segurança (risco de projeto)
- **Problema:** o `PRESET_PASSWORD` é compilado em texto puro (`option_env!`) e aceito como senha mestra mesmo quando o aparelho tem uma senha local. Qualquer pessoa com o APK, o MSI ou o DEB consegue recuperá-lo com `strings`. Embutir um hash (`"00" + base64(sha256(senha+salt))` mais um `salt`, formato que `permanent_password.rs:124-137` já aceita) esconde o texto, mas **não protege o acesso**. No protocolo do RustDesk, o cliente prova que conhece a senha com `sha256(h1 + challenge)` (`connection.rs:2078-2087`), então ter o `h1` já basta para entrar (*pass-the-hash*).
- **Impacto:** quem tiver um instalador e souber o ID de um aparelho (que aparece na tela e no launcher) consegue controlá-lo remotamente, em qualquer aparelho da frota, sem aprovação local. O risco é maior se os instaladores circularem fora da equipe.
- **Correção sugerida:** é uma decisão de produto, não uma correção pontual. As opções, da mais simples para a mais robusta:
  1. Tratar os instaladores como segredo: não publicá-los, distribuir só por canal interno e trocar o `PRESET_PASSWORD` sempre que um instalador vazar. Isso exige reinstalar em todos os aparelhos.
  2. Embutir o hash em vez do texto puro. Evita que a senha seja reaproveitada em outros sistemas, mas não resolve o acesso.
  3. Ter uma senha por aparelho, provisionada pelo launcher ou pelo backend, em vez de uma senha global.

  Sem mudar o código, vale ao menos registrar o risco como aceito.

### 🟡 [M-01] Sem `API_SERVER`, o app tenta a API em `http://<servidor>:21114` (a verificar)

- **Local:** `src/common.rs:1083-1103`, `src/hbbs_http/sync.rs:276-285`, `src/server/connection.rs:1344-1352`, `libs/hbb_common/src/config.rs:1234-1240`
- **Categoria:** Confiabilidade / Compatibilidade com o servidor
- **Problema:** quando `api-server` está vazio, o `get_api_server_` monta `http://<host do RENDEZVOUS_SERVER>:21114`. A partir disso, o app envia heartbeat e sysinfo a cada 3 a 15 segundos e um registro de auditoria a cada conexão e transferência de arquivo. A porta 21114 só existe no **RustDesk Server Pro**. No hbbs/hbbr open source, todas essas requisições falham.
- **Impacto:** se o servidor for open source, não há falha funcional, porque o acesso remoto continua funcionando. O efeito são tentativas de conexão contínuas numa porta fechada, ruído no log e, se um firewall descartar os pacotes em silêncio, threads esperando o timeout. Se o servidor for Pro, o `API_SERVER` precisa estar preenchido, e o login, que acabou de ser ocultado, era necessário para o catálogo de endereços.
- **Correção sugerida:** confirmar o tipo do servidor. Se for open source, desligar todas as chamadas de API quando nenhum `API_SERVER` for embutido, em `baked_builtin_settings`:
  ```rust
  if baked_env(option_env!("API_SERVER")).is_none() {
      m.insert("register-device".to_owned(), "N".to_owned());
  }
  ```
  Com `register-device=N`, o `get_api_server` retorna vazio e o heartbeat, o sysinfo e a auditoria são desligados.

### 🟡 [M-02] Windows: renomear o executável troca o servidor travado

- **Local:** `src/platform/windows.rs:2126-2133`, `src/platform/windows.rs:2165`, `libs/hbb_common/src/config.rs:980-984`, `src/common.rs:691-696`
- **Categoria:** Segurança
- **Problema:** o upstream lê o servidor e a chave do nome do executável (por exemplo, `QuinyxDesk-host=x.y.z,key=....exe`) e grava o resultado em `EXE_RENDEZVOUS_SERVER`, que tem **prioridade** sobre o `custom-rendezvous-server` travado em `OVERWRITE_SETTINGS`.
- **Impacto:** quem tiver o instalador pode gerar uma cópia que aponta para outro servidor, contornando a trava que o fork criou. O ataque exige ação local, mas anula a garantia de que o usuário não consegue mudar o servidor.
- **Correção sugerida:** quando houver `RENDEZVOUS_SERVER` embutido, ignorar a licença lida do nome do exe. A forma mais simples é fazer `get_license_from_exe_name` retornar `CustomServer::default()` nesse caso.

### 🟡 [M-03] O ID só chega ao launcher com a tela do QuinyxDesk aberta e é reenviado a cada 3 s (a verificar)

- **Local:** `flutter/lib/mobile/pages/server_page.dart:186-189`, `flutter/lib/models/server_model.dart:474-485`
- **Categoria:** Confiabilidade / Integração
- **Problema:** o `report_access_id` só é chamado pelo `fetchID()`, que roda num timer de 3 segundos enquanto o `ServerPage` está montado. Quando o serviço inicia pelo boot (`BootReceiver` → `MainService`, sem Activity), nenhum ID é enviado. Com a tela aberta, o app manda 3 broadcasts a cada 3 segundos, mesmo sem mudança no ID.
- **Impacto:** se o launcher depende só desse broadcast e o QuinyxDesk subiu pelo boot, o launcher fica sem ID até alguém abrir o app. A verificar: depende de o launcher abrir o QuinyxDesk ou usar outra fonte para o ID.
- **Correção sugerida:** enviar só quando o ID mudar (guardar o último ID enviado) e também a partir do lado nativo, quando o `MainService` iniciar. Se o launcher já abre o app, basta o primeiro ajuste.

### 🟢 [B-01] Comentários desatualizados sobre o sinal de sessão remota (FLAG_SECURE)

- **Local:** `flutter/android/app/src/main/AndroidManifest.xml:23-24`, `flutter/android/app/build.gradle:35-48`
- **Categoria:** Manutenibilidade
- **Problema:** os comentários citam `notifyLauncherRemoteSession em MainService` e "o sinal que desliga o FLAG_SECURE". Esse sinal foi removido no commit `aaf557ce6`, e hoje o token protege só o broadcast de ID.
- **Correção sugerida:** atualizar os comentários para refletir que o token autentica o `REMOTE_ID`.

### 🟢 [B-02] O botão de escanear QR nas configurações do Android não tem mais utilidade

- **Local:** `flutter/lib/mobile/pages/settings_page.dart:31`, `flutter/lib/mobile/pages/scan_page.dart:159`, `flutter/lib/mobile/widgets/dialog.dart:73-78`
- **Categoria:** Bug (UX)
- **Problema:** o `ScanButton` continua na AppBar das configurações. Ao ler um QR de servidor, ele abre o diálogo, que agora nunca preenche os campos e exige um campo API. Mesmo que o usuário preencha tudo, o `is_option_can_save` recusa as chaves travadas, e o app ainda mostra "Successful".
- **Correção sugerida:** ocultar o `ScanButton` (a lista `appBarActions` vazia) do mesmo jeito que o item "ID/Relay Server".

### 🟢 [B-03] O deep link `rustdesk://` no Android e no Linux conflita com o RustDesk oficial

- **Local:** `src/common.rs:1013-1034`, `flutter/android/app/src/main/AndroidManifest.xml` (`android:scheme`)
- **Categoria:** Manutenibilidade
- **Problema:** fora do Windows, o esquema continua `rustdesk://`. Num aparelho com os dois apps instalados, o sistema pergunta qual deve abrir o link, ou abre o errado.
- **Correção sugerida:** só faz diferença se os deep links forem usados. Nesse caso, migrar para `quinyxdesk://` no manifest, no `.desktop` do Linux e no `get_uri_prefix`, tudo de uma vez.

### 🟢 [B-04] Os artefatos de build ainda se chamam `rustdesk-*`

- **Local:** `.github/workflows/meu-android.yml:236-244`, `.github/workflows/meu-windows.yml:174,188,193`
- **Categoria:** Manutenibilidade
- **Problema:** APK, EXE e MSI saem como `rustdesk-1.0.0-...`. Não afeta o funcionamento, mas causa confusão na distribuição e com o RustDesk oficial.
- **Correção sugerida:** renomear para `quinyxdesk-...`. No Windows, cuide para que o nome do `.exe` não contenha `-host=` nem `-qs` (ver M-02).

## Observações positivas

- **A trava de configuração usa o mecanismo do próprio upstream.** Servidor, chave e método de verificação ficam em `OVERWRITE_SETTINGS` e são recusados por `is_option_can_save` (`config.rs:1299`), sem atalhos espalhados pelo código. O `build.rs` com `rerun-if-env-changed` evita que o cache de build mantenha secrets antigos.
- **A senha mestra reaproveita a validação do upstream**, incluindo a checagem de armazenamento utilizável e a comparação em tempo constante (`connection.rs:2197-2218`). A mudança é pequena e bem comentada.
- **O build de release falha sem o `REMOTE_SESSION_TOKEN`** (`build.gradle:41-49`), e os broadcasts usam `setPackage`, o que impede que outros apps os interceptem. É o modelo que o A-01 deveria seguir.
- **O ícone flutuante foi refeito corretamente.** O recorte via `Matrix` não depende de a janela sair da tela, e o tamanho em dp acompanha a densidade. Não encontrei nenhum problema no `FloatingWindowService.kt`.

## Recomendações priorizadas

1. **A-01:** adicionar a validação dos secrets `RENDEZVOUS_SERVER` e `RS_PUB_KEY` nos três workflows. É barato e evita o pior cenário.
2. **Confirmar o tipo do servidor (open source ou Pro)** e aplicar o M-01: `register-device=N` se for open source, ou preencher `API_SERVER` se for Pro.
3. **A-03:** corrigir o `-e QuinyxDesk.exe` em `meu-windows.yml:171`. É uma linha.
4. **A-04:** ocultar "Auto update" e bloquear o `check_update` para o cliente customizado.
5. **A-05:** decidir sobre a senha mestra. No mínimo, registrar o risco como aceito e definir como os instaladores serão distribuídos.
6. **A-02:** antes de distribuir o `.deb`, alinhar o nome `quinyxdesk` no empacotamento Linux, ou suspender a distribuição Linux.
7. **M-02, M-03 e os itens B:** fazer junto com a próxima rodada de ajustes.

**Checklist do servidor em uso**, a conferir fora do código:

- Secrets `RENDEZVOUS_SERVER` (host, ou `host:porta` se a porta não for 21116) e `RS_PUB_KEY` iguais ao conteúdo de `id_ed25519.pub` do hbbs. Uma chave diferente gera "Key mismatch" em todas as conexões.
- Portas abertas: TCP 21115 a 21117 e UDP 21116. Também TCP 21118 e 21119, se for usar WebSocket.
- Relay: se o hbbs não roda com `-r <host>`, o cliente usa `<host do rendezvous>:21117`, que precisa ser o mesmo host do hbbr.
- Depois do próximo build pelo CI, confirmar numa lousa que a tela inicial mostra "Pronto" (registrado), e não "Sem conexão".
