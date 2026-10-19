package dev.sms.wallet

import android.app.Activity
import android.app.AlertDialog
import android.content.Intent
import android.os.Bundle
import android.util.Log
import com.solana.mobilewalletadapter.common.ProtocolContract
import com.solana.mobilewalletadapter.walletlib.association.AssociationUri
import com.solana.mobilewalletadapter.walletlib.authorization.AuthIssuerConfig
import com.solana.mobilewalletadapter.walletlib.protocol.MobileWalletAdapterConfig
import com.solana.mobilewalletadapter.walletlib.scenario.AuthorizeRequest
import com.solana.mobilewalletadapter.walletlib.scenario.AuthorizedAccount
import com.solana.mobilewalletadapter.walletlib.scenario.DeauthorizedEvent
import com.solana.mobilewalletadapter.walletlib.scenario.LocalScenario
import com.solana.mobilewalletadapter.walletlib.scenario.ReauthorizeRequest
import com.solana.mobilewalletadapter.walletlib.scenario.Scenario
import com.solana.mobilewalletadapter.walletlib.scenario.SignAndSendTransactionsRequest
import com.solana.mobilewalletadapter.walletlib.scenario.SignMessagesRequest
import com.solana.mobilewalletadapter.walletlib.scenario.SignTransactionsRequest
import com.solanamobile.seedvault.SigningRequest
import com.solanamobile.seedvault.Wallet
import com.solanamobile.seedvault.WalletContractV1
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch

/**
 * The bridge: Mobile Wallet Adapter requests from a dapp come in here, and every key operation
 * goes to Seed Vault (which shows its own approval + PIN), like the Seeker's built-in wallet.
 */
class MwaActivity : Activity() {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
    private var scenario: Scenario? = null

    /** The request waiting for a Seed Vault result. */
    private var pending: Pending? = null

    private sealed interface Pending {
        data class Connect(val request: AuthorizeRequest) : Pending
        data class SignTx(val request: SignTransactionsRequest, val account: SeedVaultAccount) : Pending
        data class SignAndSend(val request: SignAndSendTransactionsRequest, val account: SeedVaultAccount) : Pending
        data class SignMsg(val request: SignMessagesRequest, val account: SeedVaultAccount) : Pending
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setFinishOnTouchOutside(false)
        if (!startSession(intent)) finish()
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        scenario?.close()
        if (!startSession(intent)) finish()
    }

    override fun onDestroy() {
        scenario?.close()
        scope.cancel()
        super.onDestroy()
    }

    private fun startSession(intent: Intent?): Boolean {
        val uri = intent?.data ?: return false.also { Log.e(TAG, "No association URI") }
        val association = AssociationUri.parse(uri) ?: return false.also { Log.e(TAG, "Unsupported association URI $uri") }
        val config = MobileWalletAdapterConfig(
            10, // max transactions per request
            10, // max messages per request
            arrayOf<Any>(MobileWalletAdapterConfig.LEGACY_TRANSACTION_VERSION, 0),
            3_000L,
            arrayOf(ProtocolContract.FEATURE_ID_SIGN_TRANSACTIONS),
        )
        scenario = association.createScenario(applicationContext, config, AuthIssuerConfig("sms-wallet"), Callbacks()).also {
            it.startAsync()
        }
        Log.i(TAG, "MWA session starting (policy=${Policy.get(this)})")
        return true
    }

    // ---- requests -------------------------------------------------------------------------------

    private fun onAuthorize(request: AuthorizeRequest) {
        val account = SeedVaultAccount.load(this)
        if (account == null) {
            // First use: ask Seed Vault to authorize a seed for this wallet, then continue.
            Log.i(TAG, "No Seed Vault account yet; asking Seed Vault to authorize a seed")
            pending = Pending.Connect(request)
            startActivityForResult(Wallet.authorizeSeed(this, WalletContractV1.PURPOSE_SIGN_SOLANA_TRANSACTION), REQ_AUTHORIZE_SEED)
            return
        }
        val name = request.identityName ?: "Unknown app"
        decide(
            "Connect $name?",
            "${request.identityUri ?: ""}\n\nAccount: ${account.address}\nNetwork: ${request.chain ?: request.cluster ?: "default"}",
            onApprove = {
                Log.i(TAG, "Authorized $name with ${account.address}")
                request.completeWithAuthorize(
                    arrayOf(AuthorizedAccount(account.publicKey, account.address, "base58", "SMS test account", null, null, null)),
                    null,
                    "sms-wallet".toByteArray(),
                    null,
                )
            },
            onDecline = {
                Log.i(TAG, "Declined authorize for $name")
                request.completeWithDecline()
            },
        )
    }

    private fun onSignTransactions(request: SignTransactionsRequest) = withAccount(request::completeWithAuthorizationNotValid) { account ->
        decide("Sign ${request.payloads.size} transaction(s)?", "Account: ${account.address}",
            onApprove = { startSeedVaultTxSigning(request.payloads, account, Pending.SignTx(request, account)) },
            onDecline = { request.completeWithDecline() })
    }

    private fun onSignAndSend(request: SignAndSendTransactionsRequest) = withAccount(request::completeWithAuthorizationNotValid) { account ->
        decide("Sign and send ${request.payloads.size} transaction(s)?",
            "Account: ${account.address}\nRPC: ${Rpc.urlForChain(request.chain ?: request.cluster)}",
            onApprove = { startSeedVaultTxSigning(request.payloads, account, Pending.SignAndSend(request, account)) },
            onDecline = { request.completeWithDecline() })
    }

    private fun onSignMessages(request: SignMessagesRequest) = withAccount(request::completeWithAuthorizationNotValid) { account ->
        if (request.addresses.none { it.contentEquals(account.publicKey) }) {
            request.completeWithAuthorizationNotValid()
            return@withAccount
        }
        decide("Sign ${request.payloads.size} message(s)?", "Account: ${account.address}",
            onApprove = {
                val signing = ArrayList(request.payloads.map { SigningRequest(it, arrayListOf(account.derivationPath)) })
                pending = Pending.SignMsg(request, account)
                startActivityForResult(Wallet.signMessages(this, account.authToken, signing), REQ_SIGN_MESSAGES)
            },
            onDecline = { request.completeWithDecline() })
    }

    private inline fun withAccount(onMissing: () -> Unit, block: (SeedVaultAccount) -> Unit) {
        val account = SeedVaultAccount.load(this)
        if (account == null) {
            Log.w(TAG, "Signing request but no Seed Vault account")
            onMissing()
        } else {
            block(account)
        }
    }

    /** Asks Seed Vault to sign each transaction's message with our account. */
    private fun startSeedVaultTxSigning(payloads: Array<ByteArray>, account: SeedVaultAccount, next: Pending) {
        val messages = payloads.map { tx ->
            runCatching {
                val parsed = SolanaTx.parse(tx)
                require(SolanaTx.signerIndex(parsed, account.publicKey) >= 0) { "account is not a signer" }
                SolanaTx.message(tx, parsed)
            }.getOrNull()
        }
        if (messages.any { it == null }) {
            val valid = messages.map { it != null }.toBooleanArray()
            Log.w(TAG, "Invalid transaction payload(s): ${valid.toList()}")
            when (next) {
                is Pending.SignTx -> next.request.completeWithInvalidPayloads(valid)
                is Pending.SignAndSend -> next.request.completeWithInvalidSignatures(valid)
                else -> Unit
            }
            return
        }
        val signing = ArrayList(messages.map { SigningRequest(it!!, arrayListOf(account.derivationPath)) })
        pending = next
        startActivityForResult(Wallet.signTransactions(this, account.authToken, signing), REQ_SIGN_TRANSACTIONS)
    }

    // ---- Seed Vault results ---------------------------------------------------------------------

    @Deprecated("Activity result API")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        val p = pending.also { pending = null } ?: return
        when (requestCode) {
            REQ_AUTHORIZE_SEED -> {
                val request = (p as Pending.Connect).request
                runCatching { Wallet.onAuthorizeSeedResult(resultCode, data) }
                    .onSuccess {
                        SeedVaultAccount.saveAuthToken(this, it)
                        onAuthorize(request)
                    }
                    .onFailure {
                        // NO_AVAILABLE_SEEDS can mean "already authorized"; try loading the account again.
                        if (SeedVaultAccount.load(this) != null) onAuthorize(request)
                        else {
                            Log.w(TAG, "Seed Vault did not authorize a seed", it)
                            request.completeWithDecline()
                        }
                    }
            }
            REQ_SIGN_TRANSACTIONS -> {
                val signatures = runCatching { Wallet.onSignTransactionsResult(resultCode, data).map { it.signatures[0] } }
                    .getOrElse { e -> return seedVaultFailed(p, resultCode, e) }
                when (p) {
                    is Pending.SignTx -> {
                        val signed = p.request.payloads.mapIndexed { i, tx -> SolanaTx.withSignature(tx, p.account.publicKey, signatures[i]) }
                        Log.i(TAG, "Signed ${signed.size} transaction(s) via Seed Vault: ${signatures.joinToString { Base58.encode(it) }}")
                        p.request.completeWithSignedPayloads(signed.toTypedArray())
                    }
                    is Pending.SignAndSend -> send(p, signatures)
                    else -> Unit
                }
            }
            REQ_SIGN_MESSAGES -> {
                val p2 = p as Pending.SignMsg
                val signatures = runCatching { Wallet.onSignMessagesResult(resultCode, data).map { it.signatures[0] } }
                    .getOrElse { e -> return seedVaultFailed(p, resultCode, e) }
                // MWA signed message payload = message bytes followed by the signature.
                val signed = p2.request.payloads.mapIndexed { i, msg -> msg + signatures[i] }
                Log.i(TAG, "Signed ${signed.size} message(s) via Seed Vault")
                p2.request.completeWithSignedPayloads(signed.toTypedArray())
            }
        }
    }

    private fun seedVaultFailed(p: Pending, resultCode: Int, e: Throwable) {
        Log.w(TAG, "Seed Vault signing failed (result=$resultCode)", e)
        when (p) {
            is Pending.SignTx -> p.request.completeWithDecline()
            is Pending.SignAndSend -> p.request.completeWithDecline()
            is Pending.SignMsg -> p.request.completeWithDecline()
            is Pending.Connect -> p.request.completeWithDecline()
        }
    }

    private fun send(p: Pending.SignAndSend, signatures: List<ByteArray>) {
        val req = p.request
        val url = Rpc.urlForChain(req.chain ?: req.cluster)
        val signed = req.payloads.mapIndexed { i, tx -> SolanaTx.withSignature(tx, p.account.publicKey, signatures[i]) }
        scope.launch(Dispatchers.IO) {
            val submitted = signed.map { tx ->
                runCatching { Rpc.sendTransaction(url, tx, req.skipPreflight, req.commitment, req.maxRetries, req.minContextSlot) }
                    .onSuccess { Log.i(TAG, "Sent transaction $it to $url") }
                    .onFailure { Log.w(TAG, "sendTransaction to $url failed", it) }
                    .isSuccess
            }
            if (submitted.all { it }) {
                req.completeWithSignatures(signatures.toTypedArray())
            } else {
                req.completeWithNotSubmitted(Array(signatures.size) { i -> if (submitted[i]) signatures[i] else null })
            }
        }
    }

    // ---- approval -------------------------------------------------------------------------------

    /** Applies the policy, or shows an Approve / Decline dialog. */
    private fun decide(title: String, message: String, onApprove: () -> Unit, onDecline: () -> Unit) {
        when (Policy.get(this)) {
            Policy.APPROVE -> onApprove()
            Policy.REJECT -> onDecline()
            Policy.ASK -> AlertDialog.Builder(this)
                .setTitle(title)
                .setMessage(message)
                .setCancelable(false)
                .setPositiveButton("Approve") { _, _ -> onApprove() }
                .setNegativeButton("Decline") { _, _ -> onDecline() }
                .show()
        }
    }

    private inner class Callbacks : LocalScenario.Callbacks {
        override fun onScenarioReady() = Unit
        override fun onScenarioServingClients() = Unit
        override fun onScenarioServingComplete() {
            runOnUiThread { scenario?.close() }
        }
        override fun onScenarioComplete() = Unit
        override fun onScenarioError() = Unit
        override fun onScenarioTeardownComplete() {
            Log.i(TAG, "MWA session ended")
            runOnUiThread { finish() }
        }

        override fun onAuthorizeRequest(request: AuthorizeRequest) = runOnUiThread { onAuthorize(request) }
        override fun onReauthorizeRequest(request: ReauthorizeRequest) = request.completeWithReauthorize()
        override fun onSignTransactionsRequest(request: SignTransactionsRequest) = runOnUiThread { onSignTransactions(request) }
        override fun onSignMessagesRequest(request: SignMessagesRequest) = runOnUiThread { onSignMessages(request) }
        override fun onSignAndSendTransactionsRequest(request: SignAndSendTransactionsRequest) = runOnUiThread { onSignAndSend(request) }
        override fun onDeauthorizedEvent(event: DeauthorizedEvent) = event.complete()
        override fun onLowPowerAndNoConnection() {
            Log.w(TAG, "Low power mode and no connection; the dapp's connection may have been blocked")
        }
    }

    companion object {
        private const val TAG = "SmsWallet"
        private const val REQ_AUTHORIZE_SEED = 10
        private const val REQ_SIGN_TRANSACTIONS = 11
        private const val REQ_SIGN_MESSAGES = 12
    }
}
