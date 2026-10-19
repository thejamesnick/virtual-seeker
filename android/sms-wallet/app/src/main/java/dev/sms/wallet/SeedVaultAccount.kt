package dev.sms.wallet

import android.content.Context
import android.net.Uri
import com.solanamobile.seedvault.Wallet
import com.solanamobile.seedvault.WalletContractV1

/** The one account this test wallet uses: `m/44'/501'/0'` of the seed authorized in Seed Vault. */
data class SeedVaultAccount(
    val authToken: Long,
    val publicKey: ByteArray,
    val address: String,
    val derivationPath: Uri,
) {
    companion object {
        /** Same default account path as Seed Vault's pre-derived accounts. */
        val DERIVATION_PATH: Uri = Uri.parse("bip32:/m/44'/501'/0'")

        private const val PREFS = "sms_wallet"
        private const val KEY_AUTH_TOKEN = "auth_token"

        /** Loads the account if a seed is already authorized for this wallet, else null. */
        fun load(context: Context): SeedVaultAccount? {
            val token = savedAuthToken(context) ?: findAuthorizedSeed(context) ?: return null
            return accountFor(context, token)
        }

        fun saveAuthToken(context: Context, authToken: Long) {
            context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit()
                .putLong(KEY_AUTH_TOKEN, authToken).apply()
        }

        fun clear(context: Context) {
            context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit().remove(KEY_AUTH_TOKEN).apply()
        }

        private fun savedAuthToken(context: Context): Long? {
            val token = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).getLong(KEY_AUTH_TOKEN, -1)
            return token.takeIf { it != -1L }
        }

        /** An auth token Seed Vault already granted this app (e.g. after a reinstall of our prefs). */
        private fun findAuthorizedSeed(context: Context): Long? = runCatching {
            Wallet.getAuthorizedSeeds(
                context,
                arrayOf(WalletContractV1.AUTHORIZED_SEEDS_AUTH_TOKEN, WalletContractV1.AUTHORIZED_SEEDS_AUTH_PURPOSE),
            )?.use { c ->
                while (c.moveToNext()) {
                    if (c.getInt(1) == WalletContractV1.PURPOSE_SIGN_SOLANA_TRANSACTION) return@use c.getLong(0)
                }
                null
            }
        }.getOrNull()?.also { saveAuthToken(context, it) }

        private fun accountFor(context: Context, authToken: Long): SeedVaultAccount? = runCatching {
            Wallet.getAccounts(
                context,
                authToken,
                arrayOf(
                    WalletContractV1.ACCOUNTS_BIP32_DERIVATION_PATH,
                    WalletContractV1.ACCOUNTS_PUBLIC_KEY_RAW,
                    WalletContractV1.ACCOUNTS_PUBLIC_KEY_ENCODED,
                ),
            )?.use { c ->
                while (c.moveToNext()) {
                    if (c.getString(0) == DERIVATION_PATH.toString()) {
                        return@use SeedVaultAccount(authToken, c.getBlob(1), c.getString(2), DERIVATION_PATH)
                    }
                }
                null
            }
        }.getOrNull()
    }

    override fun equals(other: Any?) = other is SeedVaultAccount && other.authToken == authToken && other.publicKey.contentEquals(publicKey)
    override fun hashCode() = publicKey.contentHashCode()
}
