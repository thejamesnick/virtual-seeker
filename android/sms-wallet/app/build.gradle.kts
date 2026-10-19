plugins {
    id("com.android.application")
}

android {
    namespace = "dev.sms.wallet"
    compileSdk = 37

    defaultConfig {
        applicationId = "dev.sms.wallet"
        // Seed Vault requires API 31+; the virtual Seeker is API 36.
        minSdk = 31
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            // Test wallet: signed with the debug key so `sms setup` can install it anywhere.
            signingConfig = signingConfigs.getByName("debug")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

dependencies {
    // Mobile Wallet Adapter, wallet side (talks to dapps)
    implementation("com.solanamobile:mobile-wallet-adapter-walletlib:2.2.0")
    // Seed Vault Wallet API (keys + signing live in Seed Vault)
    implementation("com.solanamobile:seedvault-wallet-sdk:0.4.0")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.11.0")
}
