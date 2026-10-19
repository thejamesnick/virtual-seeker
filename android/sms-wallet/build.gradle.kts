buildscript {
    dependencies {
        // AGP 9's built-in Kotlin is 2.2; our dependencies need a 2.4 compiler (same as seed-vault-sdk).
        classpath("org.jetbrains.kotlin:kotlin-gradle-plugin:2.4.20")
    }
}

plugins {
    id("com.android.application") version "9.1.1" apply false
}
