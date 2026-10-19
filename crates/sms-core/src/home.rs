//! Seeker-style home screen layout (Pixel launcher on the google_apis image).
//!
//! Modeled on a real Seeker's home screen (hands-on review photo): 3 rows of 4 apps above the search
//! bar with the dApp Store at row 1 / column 3 and the Wallet at row 3 / column 4, and a dock of
//! Phone · Messages · Chrome · Gmail. Apps a real Seeker has but the emulator image doesn't (dApp
//! Store, Play Store, third-party apps) are replaced with the closest available ones.

use anyhow::{Context, Result};

use crate::adb::Adb;

const LAUNCHER: &str = "com.google.android.apps.nexuslauncher";
const LAUNCHER_DB: &str = "/data/data/com.google.android.apps.nexuslauncher/databases/launcher_4_by_5.db";
const CONTAINER_DESKTOP: i32 = -100;
const CONTAINER_HOTSEAT: i32 = -101;

/// (container, cellX, cellY, title, component). Hotseat items use cellX as their slot.
const LAYOUT: &[(i32, u32, u32, &str, &str)] = &[
    // Row 1 — real: Bluesky | Camera | dApp Store | Instagram
    (CONTAINER_DESKTOP, 0, 2, "Clock", "com.google.android.deskclock/com.android.deskclock.DeskClock"),
    (CONTAINER_DESKTOP, 1, 2, "Camera", "com.android.camera2/com.android.camera.CameraLauncher"),
    (CONTAINER_DESKTOP, 2, 2, "dApps", "dev.sms.wallet/dev.sms.wallet.DappsActivity"),
    (CONTAINER_DESKTOP, 3, 2, "Calendar", "com.google.android.calendar/com.android.calendar.AllInOneActivity"),
    // Row 2 — real: Maps | Apple Music | Photos | Play Store
    (CONTAINER_DESKTOP, 0, 3, "Maps", "com.google.android.apps.maps/com.google.android.maps.MapsActivity"),
    (CONTAINER_DESKTOP, 1, 3, "YT Music", "com.google.android.apps.youtube.music/com.google.android.apps.youtube.music.activities.MusicActivity"),
    (CONTAINER_DESKTOP, 2, 3, "Photos", "com.google.android.apps.photos/com.google.android.apps.photos.home.HomeActivity"),
    (CONTAINER_DESKTOP, 3, 3, "Settings", "com.android.settings/com.android.settings.Settings"),
    // Row 3 — real: Pocket Casts | TikTok | YouTube | Wallet
    (CONTAINER_DESKTOP, 0, 4, "Fake dApp", "com.solana.mobilewalletadapter.fakedapp/com.solana.mobilewalletadapter.fakedapp.MainActivity"),
    (CONTAINER_DESKTOP, 1, 4, "Contacts", "com.google.android.contacts/com.android.contacts.activities.PeopleActivity"),
    (CONTAINER_DESKTOP, 2, 4, "YouTube", "com.google.android.youtube/com.google.android.youtube.app.honeycomb.Shell$HomeActivity"),
    (CONTAINER_DESKTOP, 3, 4, "Wallet", "dev.sms.wallet/dev.sms.wallet.MainActivity"),
    // Dock — same as a real Seeker
    (CONTAINER_HOTSEAT, 0, 0, "Phone", "com.google.android.dialer/com.google.android.dialer.extensions.GoogleDialtactsActivity"),
    (CONTAINER_HOTSEAT, 1, 0, "Messages", "com.google.android.apps.messaging/com.google.android.apps.messaging.ui.ConversationListActivity"),
    (CONTAINER_HOTSEAT, 2, 0, "Chrome", "com.android.chrome/com.google.android.apps.chrome.Main"),
    (CONTAINER_HOTSEAT, 3, 0, "Gmail", "com.google.android.gm/com.google.android.gm.ConversationListActivityGmail"),
];

/// Replaces the home screen apps and dock with the Seeker layout. Skips apps that aren't installed.
/// Returns how many icons were placed. Needs root (`adb root`, google_apis images).
pub fn apply_seeker_home(adb: &Adb) -> Result<usize> {
    adb.root()?;
    let installed = adb
        .shell("cmd package query-activities --brief -a android.intent.action.MAIN -c android.intent.category.LAUNCHER")
        .unwrap_or_default();
    let is_installed = |component: &str| {
        let (pkg, cls) = component.split_once('/').unwrap_or((component, ""));
        let short = cls.strip_prefix(pkg).map(|rest| rest.to_string()).unwrap_or_else(|| cls.to_string());
        installed.contains(&format!("{pkg}/{cls}")) || installed.contains(&format!("{pkg}/{short}"))
    };

    // Make sure the launcher has created its database (it does on first start).
    if adb.shell(&format!("[ -f {LAUNCHER_DB} ] && echo yes")).unwrap_or_default().trim() != "yes" {
        adb.shell("input keyevent KEYCODE_HOME")?;
        std::thread::sleep(std::time::Duration::from_secs(3));
    }

    let mut sql = String::from("BEGIN;\nDELETE FROM favorites WHERE itemType IN (0, 1, 6) AND container IN (-100, -101);\n");
    let mut placed = 0;
    for &(container, x, y, title, component) in LAYOUT {
        if !is_installed(component) {
            continue;
        }
        let intent = format!(
            "#Intent;action=android.intent.action.MAIN;category=android.intent.category.LAUNCHER;launchFlags=0x10200000;component={component};end"
        );
        let screen = if container == CONTAINER_HOTSEAT { x } else { 0 };
        sql.push_str(&format!(
            "INSERT INTO favorites (title, intent, container, screen, cellX, cellY, spanX, spanY, itemType, profileId, rank) \
             VALUES ('{}', '{}', {container}, {screen}, {x}, {y}, 1, 1, 0, 0, 0);\n",
            title.replace('\'', "''"),
            intent.replace('\'', "''"),
        ));
        placed += 1;
    }
    sql.push_str("COMMIT;\n");

    let tmp = std::env::temp_dir().join(format!("sms-home-{}.sql", std::process::id()));
    std::fs::write(&tmp, &sql)?;
    let pushed = adb.run(&["push", &tmp.to_string_lossy(), "/data/local/tmp/sms_home.sql"]);
    let _ = std::fs::remove_file(&tmp);
    pushed?;

    adb.shell(&format!("am force-stop {LAUNCHER}"))?;
    adb.shell(&format!("sqlite3 {LAUNCHER_DB} < /data/local/tmp/sms_home.sql && rm /data/local/tmp/sms_home.sql"))
        .context("writing the launcher layout")?;
    adb.shell("input keyevent KEYCODE_HOME")?;
    Ok(placed)
}

/// True if the Wallet sits in the Seeker spot (row 3, column 4).
pub fn has_seeker_home(adb: &Adb) -> bool {
    adb.shell(&format!(
        "sqlite3 {LAUNCHER_DB} \"select count(*) from favorites where container=-100 and cellX=3 and cellY=4 and intent like '%dev.sms.wallet/dev.sms.wallet.MainActivity%'\""
    ))
    .map(|o| o.trim() == "1")
    .unwrap_or(false)
}
