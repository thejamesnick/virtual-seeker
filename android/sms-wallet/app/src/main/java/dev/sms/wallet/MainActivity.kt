package dev.sms.wallet

import android.app.Activity
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Bundle
import android.util.Log
import android.view.Gravity
import android.widget.Button
import android.widget.LinearLayout
import android.widget.TextView
import com.solanamobile.seedvault.SeedVault
import com.solanamobile.seedvault.Wallet
import com.solanamobile.seedvault.WalletContractV1

/** Status screen: Seed Vault connection, account, and the approval policy. */
class MainActivity : Activity() {
    private lateinit var status: TextView

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        applyPolicyExtra(intent)

        val pad = (16 * resources.displayMetrics.density).toInt()
        val root = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(pad, pad, pad, pad)
        }
        root.addView(TextView(this).apply {
            text = "SMS Test Wallet"
            textSize = 24f
        })
        root.addView(TextView(this).apply {
            text = "Test wallet for the virtual Seeker. Talks Mobile Wallet Adapter to apps; keys and signing stay in Seed Vault. Test funds only."
            setPadding(0, pad / 2, 0, pad)
        })
        status = TextView(this).apply { setPadding(0, 0, 0, pad) }
        root.addView(status)
        root.addView(Button(this).apply {
            text = "Connect Seed Vault"
            setOnClickListener { connectSeedVault() }
        })
        Policy.entries.forEach { p ->
            root.addView(Button(this).apply {
                text = "Policy: ${p.label}"
                setOnClickListener {
                    Policy.set(this@MainActivity, p)
                    refresh()
                }
            })
        }
        root.gravity = Gravity.TOP
        // Android 15+ is edge-to-edge: the wrapper takes the system-bar insets, root keeps its margins.
        setContentView(android.widget.ScrollView(this).apply { fitsSystemWindows = true; addView(root) })
        darkStatusBarIcons(this)

        if (checkSelfPermission(WalletContractV1.PERMISSION_ACCESS_SEED_VAULT) != PackageManager.PERMISSION_GRANTED) {
            requestPermissions(arrayOf(WalletContractV1.PERMISSION_ACCESS_SEED_VAULT), REQUEST_PERMISSION)
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        applyPolicyExtra(intent)
        refresh()
    }

    /** `adb shell am start -n dev.sms.wallet/.MainActivity --es policy approve|reject|ask` */
    private fun applyPolicyExtra(intent: Intent?) {
        Policy.parse(intent?.getStringExtra("policy"))?.let {
            Policy.set(this, it)
            Log.i(TAG, "Policy set to $it")
        }
    }

    override fun onResume() {
        super.onResume()
        refresh()
    }

    override fun onRequestPermissionsResult(requestCode: Int, permissions: Array<out String>, grantResults: IntArray) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        refresh()
    }

    private fun refresh() {
        val vault = SeedVault.isAvailable(this, true)
        val permitted = checkSelfPermission(WalletContractV1.PERMISSION_ACCESS_SEED_VAULT) == PackageManager.PERMISSION_GRANTED
        val account = if (vault && permitted) SeedVaultAccount.load(this) else null
        status.text = buildString {
            appendLine("Seed Vault: ${if (vault) "available" else "NOT FOUND"}")
            appendLine("Permission: ${if (permitted) "granted" else "not granted"}")
            appendLine("Account: ${account?.address ?: "not connected"}")
            account?.let { appendLine("Path: ${it.derivationPath}") }
            append("Policy: ${Policy.get(this@MainActivity).label}")
        }
    }

    private fun connectSeedVault() {
        startActivityForResult(Wallet.authorizeSeed(this, WalletContractV1.PURPOSE_SIGN_SOLANA_TRANSACTION), REQUEST_AUTHORIZE)
    }

    @Deprecated("Activity result API")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (requestCode == REQUEST_AUTHORIZE) {
            runCatching { Wallet.onAuthorizeSeedResult(resultCode, data) }
                .onSuccess { SeedVaultAccount.saveAuthToken(this, it) }
                .onFailure { Log.w(TAG, "Seed authorization not granted", it) }
            refresh()
        }
    }

    companion object {
        /** Light screens need dark status-bar icons. */
        fun darkStatusBarIcons(activity: Activity) {
            activity.window.insetsController?.setSystemBarsAppearance(
                android.view.WindowInsetsController.APPEARANCE_LIGHT_STATUS_BARS,
                android.view.WindowInsetsController.APPEARANCE_LIGHT_STATUS_BARS,
            )
        }

        const val TAG = "SmsWallet"
        private const val REQUEST_PERMISSION = 1
        private const val REQUEST_AUTHORIZE = 2
    }
}
