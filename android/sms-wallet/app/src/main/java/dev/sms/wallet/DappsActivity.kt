package dev.sms.wallet

import android.app.Activity
import android.content.Intent
import android.content.pm.ApplicationInfo
import android.os.Bundle
import android.view.Gravity
import android.widget.ImageView
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView

/**
 * Sits where the Solana dApp Store is on a real Seeker's home screen. The real dApp Store only runs on
 * Solana Mobile devices, so this lists the apps *you* installed (e.g. with `sms install app.apk`) and
 * opens them.
 */
class DappsActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        title = "dApps"
    }

    override fun onResume() {
        super.onResume()
        val d = resources.displayMetrics.density
        val pad = (16 * d).toInt()
        val list = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(pad, pad, pad, pad)
        }
        list.addView(TextView(this).apply { text = "dApps"; textSize = 28f })
        list.addView(TextView(this).apply {
            text = "On a real Seeker this spot is the Solana dApp Store. In the virtual Seeker, install your app from your computer:\n\n    sms install your-app.apk\n\nInstalled apps:"
            setPadding(0, pad / 2, 0, pad)
        })

        val pm = packageManager
        val launchable = pm.queryIntentActivities(Intent(Intent.ACTION_MAIN).addCategory(Intent.CATEGORY_LAUNCHER), 0)
            .filter { it.activityInfo.applicationInfo.flags and ApplicationInfo.FLAG_SYSTEM == 0 }
            .filter { it.activityInfo.packageName != packageName }
            .sortedBy { it.loadLabel(pm).toString().lowercase() }

        if (launchable.isEmpty()) {
            list.addView(TextView(this).apply { text = "None yet." })
        }
        for (info in launchable) {
            val row = LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                setPadding(0, pad / 2, 0, pad / 2)
                isClickable = true
                setOnClickListener {
                    startActivity(Intent(Intent.ACTION_MAIN).addCategory(Intent.CATEGORY_LAUNCHER)
                        .setClassName(info.activityInfo.packageName, info.activityInfo.name)
                        .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
                }
            }
            row.addView(ImageView(this).apply {
                setImageDrawable(info.loadIcon(pm))
                layoutParams = LinearLayout.LayoutParams((48 * d).toInt(), (48 * d).toInt())
            })
            row.addView(TextView(this).apply {
                text = info.loadLabel(pm)
                textSize = 18f
                setPadding(pad, 0, 0, 0)
            })
            list.addView(row)
        }
        // Android 15+ draws apps edge-to-edge; keep content clear of the status bar and camera.
        setContentView(ScrollView(this).apply { fitsSystemWindows = true; addView(list) })
        MainActivity.darkStatusBarIcons(this)
    }
}
