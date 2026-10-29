package com.carriez.flutter_hbb

import android.content.Context
import android.content.Intent
import android.util.Log

/**
 * Ponte com o Launcher Quinyx (br.com.quinyx.launcher, com.quinyx.launcher.board e
 * com.quinyx.launcher.desk). Centraliza o contrato de broadcasts que
 * integram o QuinyxDesk ao launcher. Todos os valores aqui DEVEM bater com QuinyxDeskBridgeReceiver
 * no launcher.
 *
 *  - [reportAccessId]: empurra o ID de acesso ao launcher em tempo real, substituindo (como
 *    caminho preferencial) o arquivo gravado em Downloads.
 *
 * Tudo e best-effort: qualquer falha (launcher ausente, broadcast bloqueado) e apenas logada.
 */
object LauncherBridge {
    private const val TAG = "LauncherBridge"

    // Mesinha, Lousa e Mesa Digital, com o mesmo contrato. Broadcast para pacote
    // ausente não falha, então mandar para todos dispensa saber qual está instalado.
    private val LAUNCHER_PACKAGES = listOf("br.com.quinyx.launcher", "com.quinyx.launcher.board", "com.quinyx.launcher.desk")
    private const val ACTION_REMOTE_ID = "br.com.quinyx.launcher.action.REMOTE_ID"

    // Segredo compartilhado: prova de origem do sinal. Provisionado no build via
    // REMOTE_SESSION_TOKEN (ambiente/local.properties; ver build.gradle) e DEVE ser identico ao
    // token do launcher (QuinyxDeskBridgeReceiver).
    private val TOKEN = BuildConfig.REMOTE_SESSION_TOKEN

    fun reportAccessId(context: Context, id: String) {
        if (id.isBlank() || id == "NA") {
            return
        }
        send(context, ACTION_REMOTE_ID) { it.putExtra("id", id) }
    }

    private inline fun send(context: Context, action: String, extras: (Intent) -> Unit) {
        for (launcher in LAUNCHER_PACKAGES) {
            runCatching {
                val intent = Intent(action).apply {
                    setPackage(launcher)
                    putExtra("token", TOKEN)
                    putExtra("sender_package", context.packageName)
                    extras(this)
                }
                context.sendBroadcast(intent)
            }.onFailure { Log.w(TAG, "Falha ao enviar '$action' ao $launcher", it) }
        }
    }
}
