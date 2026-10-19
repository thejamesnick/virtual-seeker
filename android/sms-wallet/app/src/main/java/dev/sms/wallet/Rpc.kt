package dev.sms.wallet

import android.util.Base64
import org.json.JSONArray
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL

/** Sends signed transactions with Solana JSON-RPC `sendTransaction`. */
object Rpc {
    /** MWA chain id (or legacy cluster name) → RPC URL. `localnet` reaches the host via `adb reverse`. */
    fun urlForChain(chain: String?): String = when (chain) {
        "solana:mainnet", "mainnet-beta" -> "https://api.mainnet-beta.solana.com"
        "solana:testnet", "testnet" -> "https://api.testnet.solana.com"
        "solana:localnet", "localnet" -> "http://localhost:8899"
        else -> "https://api.devnet.solana.com" // solana:devnet, devnet, or unspecified
    }

    class RpcException(message: String) : Exception(message)

    /** Returns the base58 signature the node reports. Throws [RpcException] on an RPC error. */
    fun sendTransaction(
        url: String,
        signedTx: ByteArray,
        skipPreflight: Boolean?,
        commitment: String?,
        maxRetries: Int?,
        minContextSlot: Int?,
    ): String {
        val config = JSONObject().put("encoding", "base64")
        skipPreflight?.let { config.put("skipPreflight", it) }
        commitment?.let { config.put("preflightCommitment", it) }
        maxRetries?.let { config.put("maxRetries", it) }
        minContextSlot?.let { config.put("minContextSlot", it) }
        val body = JSONObject()
            .put("jsonrpc", "2.0")
            .put("id", 1)
            .put("method", "sendTransaction")
            .put("params", JSONArray().put(Base64.encodeToString(signedTx, Base64.NO_WRAP)).put(config))

        val conn = (URL(url).openConnection() as HttpURLConnection).apply {
            requestMethod = "POST"
            doOutput = true
            connectTimeout = 10_000
            readTimeout = 30_000
            setRequestProperty("Content-Type", "application/json")
        }
        try {
            conn.outputStream.use { it.write(body.toString().toByteArray()) }
            val text = (if (conn.responseCode in 200..299) conn.inputStream else conn.errorStream)
                ?.bufferedReader()?.use { it.readText() }.orEmpty()
            val json = runCatching { JSONObject(text) }.getOrElse { throw RpcException("HTTP ${conn.responseCode}: $text") }
            json.optJSONObject("error")?.let { throw RpcException(it.optString("message", it.toString())) }
            return json.optString("result").ifEmpty { throw RpcException("no result: $text") }
        } finally {
            conn.disconnect()
        }
    }
}
