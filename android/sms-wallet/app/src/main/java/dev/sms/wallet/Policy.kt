package dev.sms.wallet

import android.content.Context

/**
 * How the wallet answers dapp requests (connect / sign). Seed Vault still asks for its own PIN,
 * exactly like on a real Seeker.
 *
 * Set from the app, or from a script:
 *   adb shell am start -n dev.sms.wallet/.MainActivity --es policy approve
 */
enum class Policy(val label: String) {
    ASK("Ask me"),
    APPROVE("Always approve"),
    REJECT("Always reject");

    companion object {
        private const val PREFS = "sms_wallet"
        private const val KEY = "policy"

        fun get(context: Context): Policy =
            context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).getString(KEY, null)
                ?.let { name -> entries.firstOrNull { it.name.equals(name, ignoreCase = true) } } ?: ASK

        fun set(context: Context, policy: Policy) {
            context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit().putString(KEY, policy.name).apply()
        }

        fun parse(value: String?): Policy? = entries.firstOrNull { it.name.equals(value, ignoreCase = true) }
    }
}
