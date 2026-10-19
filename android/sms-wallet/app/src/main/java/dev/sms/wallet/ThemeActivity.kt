package dev.sms.wallet

import android.app.Activity
import android.app.WallpaperManager
import android.graphics.BitmapFactory
import android.os.Bundle
import android.util.Log

/**
 * Applies the virtual Seeker's wallpaper (home + lock screen), then closes. Used by `sms setup`:
 *   adb shell am start -W -n dev.sms.wallet/.ThemeActivity
 * Android has no shell command for setting a wallpaper, so an app has to do it.
 */
class ThemeActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        runCatching {
            val bitmap = BitmapFactory.decodeResource(resources, R.drawable.seeker_wallpaper)
            WallpaperManager.getInstance(this)
                .setBitmap(bitmap, null, true, WallpaperManager.FLAG_SYSTEM or WallpaperManager.FLAG_LOCK)
        }.onSuccess { Log.i(TAG, "Seeker wallpaper applied") }
            .onFailure { Log.e(TAG, "Could not set wallpaper", it) }
        finish()
    }

    companion object {
        private const val TAG = "SmsWallet"
    }
}
