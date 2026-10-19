# Technical Building Blocks: Rust CLI Orchestrating an Android Emulator + Solana Networks (as of Oct 18, 2026)

Scope: the parts needed for an open-source "Solana Mobile Simulator" CLI (`sms doctor | dev | devices | install | logs | tx | stop`). macOS on Apple Silicon comes first; Linux, Windows and CI come later.
Rule used throughout: anything not confirmed by a fetched source is marked **[unverified, from general knowledge]** and appears only under Inferences or Gaps.

---

## 1. Android SDK command-line tools (sdkmanager, avdmanager, emulator, adb): detection, packages, AVD creation, headless launch, boot detection, snapshots

### Takeaway
The classic toolchain still works: `cmdline-tools/latest/bin/{sdkmanager,avdmanager}`, `emulator/emulator` and `platform-tools/adb`. One big 2026 change: Google's docs now mark **sdkmanager and avdmanager as deprecated** in favor of a new **Android CLI** (`android sdk ...`, `android emulator create|start|stop|list|remove`). `sms` should detect both and prefer the classic binaries, which are stable and scriptable, until the Android CLI's flags are documented and stable. On Apple Silicon, use `arm64-v8a` system images. Headless flags and snapshot flags are well documented.

### Cited Findings
**Paths and detection**
- Default SDK location on macOS (Android Studio install): `~/Library/Android/sdk/`. System images live at `~/Library/Android/sdk/system-images/android-<apiLevel>/<variant>/<arch>/`. AVD data lives at `~/.android/avd/<name>.avd/`. Example emulator binary: `/Users/<user>/Library/Android/sdk/emulator/emulator`. — [Android: Start the emulator from the command line](https://developer.android.com/studio/run/emulator-commandline)
- sdkmanager must sit at `android_sdk/cmdline-tools/latest/bin/sdkmanager`. The `latest` subdirectory is required because it holds `lib`, `bin`, `NOTICE.txt` and `source.properties`. — [Android: sdkmanager](https://developer.android.com/tools/sdkmanager)
- avdmanager ships in the Command-Line Tools package at `android_sdk/cmdline-tools/<version>/bin/avdmanager`. — [Android: avdmanager](https://developer.android.com/tools/avdmanager)
- The sdkmanager page links to an "Environment variables" reference (`/tools/variables`) for ANDROID_HOME and ANDROID_SDK_ROOT. That page was not fetched in this session. — [Android: sdkmanager](https://developer.android.com/tools/sdkmanager)

**Deprecation and the new Android CLI (new in 2026)**
- "sdkmanager is deprecated". Replacement: the Android CLI, `android sdk [install|list|update|remove]`. — [Android: sdkmanager](https://developer.android.com/tools/sdkmanager)
- "avdmanager is deprecated". Replacement: `android emulator [create|remove]`. Download link at `/tools/agents`. — [Android: avdmanager](https://developer.android.com/tools/avdmanager)
- The Android CLI command groups are `android emulator create|list|remove|start|stop` and `android sdk install|list|remove|update`. Also `android info`, which prints the SDK location and environment, and `android help <command>`. — [Android CLI commands](https://developer.android.com/tools/agents/android-cli/commands)
- Search snippets say `android emulator start` returns only once the emulator is fully started. Examples shown: `android emulator start medium_phone` and `android emulator list`. — [Android CLI (FR overview)](https://developer.android.com/tools/agents/android-cli?hl=fr) (via search snippet; not fully fetched)
- A third-party guide warns that `android sdk list` can misreport installed images and suggests checking the filesystem. It also notes newer tooling uses slash-separated package names (`system-images/android-<api>/google_apis_playstore/arm64-v8a`). — [arda.pw guide](https://arda.pw/posts/android-emulator-macos-without-android-studio/) (third-party, via search snippet)

**sdkmanager commands**
- `sdkmanager "platform-tools" "platforms;android-36"`, `sdkmanager --list`, `--list --channel=3` (0=stable, 1=beta, 2=dev, 3=canary), `--update`, `--licenses` (needed on CI and headless machines), `--sdk_root=path`, `--package_file=file`, and the `REPO_OS_OVERRIDE` env var. — [Android: sdkmanager](https://developer.android.com/tools/sdkmanager)

**System images on Apple Silicon**
- Apple Silicon needs `arm64-v8a` images; x86 and x86_64 images don't run natively. Example: `sdkmanager "system-images;android-34;google_apis_playstore;arm64-v8a"`. — [dev.to Homebrew guide](https://dev.to/mochafreddo/setting-up-and-managing-android-emulators-on-macos-with-homebrew-3fg0) and [arda.pw](https://arda.pw/posts/android-emulator-macos-without-android-studio/) (third-party)
- Play Store images are locked down: no `adb root` or `adb remount`. — [arda.pw](https://arda.pw/posts/android-emulator-macos-without-android-studio/) (third-party, search snippet)
- A forum report says Play Store app-compatibility checks failed on API 35–37 arm64 Play images. — [Shopify dev community](https://community.shopify.dev/t/shopify-pos-your-device-isnt-compatible-with-this-version-on-android-studio-emulator-api-35-36-37-google-play-images/36093) (anecdotal)
- The 2020 Apple Silicon emulator preview is historical and **outdated**. — [Android Studio blog 2020](https://androidstudio.googleblog.com/2020/12/android-emulator-apple-silicon-preview.html?m=1)
- The `-sdcard` option "will not work on Mac with Apple Silicon." — [Android: emulator command line](https://developer.android.com/studio/run/emulator-commandline)

**Creating an AVD non-interactively**
- `avdmanager create avd -n name -k "sdk_id" [-c {path|size}] [-f] [-p path]`. Example: `avdmanager create avd -n test -k "system-images;android-25;google_apis;x86"`. `-f` forces overwrite. `avdmanager list [target|device|avd] [-c]`. `delete avd -n name`. — [Android: avdmanager](https://developer.android.com/tools/avdmanager)
- The android-emulator-runner action exposes a `profile` input (for example `pixel_7_pro`), which maps to an avdmanager device definition. — [android-emulator-runner](https://github.com/ReactiveCircus/android-emulator-runner)

**Headless launch flags**
- `-no-window` disables the window; you then reach the device through adb or the console. `-no-boot-anim` speeds up boot. `-wipe-data` resets the device. `-gpu swiftshader_indirect` is the documented example. `-port <even>` sets console port N and adb port N+1. The range is 5554–5682, so up to 64 instances. An odd port can stay hidden from `adb devices`. — [Android: emulator command line](https://developer.android.com/studio/run/emulator-commandline)
- The CI-standard default set is `-no-window -gpu swiftshader_indirect -no-snapshot -noaudio -no-boot-anim`. — [android-emulator-runner](https://github.com/ReactiveCircus/android-emulator-runner)

**Snapshots for reproducibility**
- `-no-snapshot` turns off both auto-load and auto-save, giving a full cold boot. `-no-snapshot-load` and `-no-snapshot-save` control each half. `-snapshot <name>` loads and saves a named snapshot; the default is `default_boot`. `-snapshot-list` lists snapshots. `-no-snapshot-update-time` skips the clock fix after a restore. — [Android: emulator command line](https://developer.android.com/studio/run/emulator-commandline)

**gRPC and console**
- The page documents gRPC only as Android Studio's launch flags `-grpc-use-token -idle-grpc-timeout 300`. These are "only used to run the emulator window within Android Studio." — [Android: emulator command line](https://developer.android.com/studio/run/emulator-commandline)
- Console port = the even port. adb port = console + 1. — [Android: emulator command line](https://developer.android.com/studio/run/emulator-commandline)

### Inferences
- Concrete `sms dev` boot sequence for macOS arm64. Flags are cited above; the boot-wait and kill commands are **[unverified, from general knowledge]**:
  ```sh
  SDK="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Library/Android/sdk}}"
  yes | "$SDK/cmdline-tools/latest/bin/sdkmanager" --licenses
  "$SDK/cmdline-tools/latest/bin/sdkmanager" "platform-tools" "emulator" "system-images;android-35;google_apis;arm64-v8a"
  echo no | "$SDK/cmdline-tools/latest/bin/avdmanager" create avd -n sms_api35 -k "system-images;android-35;google_apis;arm64-v8a" -d pixel_7 -f
  "$SDK/emulator/emulator" -avd sms_api35 -port 5554 -no-window -no-boot-anim -noaudio -gpu swiftshader_indirect -no-snapshot-save &
  adb -s emulator-5554 wait-for-device
  until [ "$(adb -s emulator-5554 shell getprop sys.boot_completed | tr -d '\r')" = "1" ]; do sleep 1; done   # [unverified]
  adb -s emulator-5554 emu kill   # stop. [unverified]
  ```
  `-d pixel_7`, `wait-for-device`, `getprop sys.boot_completed` and `emu kill` are standard practice but were not confirmed on the pages fetched here. Also check `dev.bootcomplete` and `init.svc.bootanim=stopped` for robustness. **[unverified]**
- Prefer **`google_apis`** over `google_apis_playstore` for the dev loop, because Play images block `adb root` (cited above). Offer Play images as opt-in only, for testing wallets installed from the Play Store (for example Phantom or Solflare).
- On Apple Silicon, `-gpu host` (Metal/ANGLE) is likely faster than `swiftshader_indirect` when a window is shown. Use swiftshader for `--headless`/CI. **[unverified]**
- `sms doctor` detection order: `$ANDROID_HOME`, then `$ANDROID_SDK_ROOT`, then `~/Library/Android/sdk` (macOS), `~/Android/Sdk` (Linux) and `%LOCALAPPDATA%\Android\Sdk` (Windows). Then check `which adb`, `which android` (the new CLI) and `emulator -version`. The Linux and Windows defaults are **[unverified]**.
- A snapshot-based fast path: boot once with the wallet installed and funded, then save a named snapshot (`-snapshot sms_ready`). Later runs load it with `-no-snapshot-save`, so every session starts from a known state. This mirrors the CI caching recipe in §3.
- Emulator console: `telnet localhost 5554` needs the auth token from `~/.emulator_console_auth_token`. `adb emu <cmd>` proxies to the console. The emulator also exposes a gRPC service (`-grpc <port>`, with protos in the emulator package) for screenshots and input. **[unverified, from general knowledge]**. Use adb-only control in v1.

### Gaps
- JDK requirement for cmdline-tools: none of the fetched pages state it. From general knowledge, recent cmdline-tools need JDK 17+ **[unverified]**. Maestro needs Java 17+ (cited in §6), so `sms doctor` should check `java -version` ≥ 17 anyway.
- The Android CLI install method, flags (headless? port?), JDK needs and platform support weren't on the fetched pages. Check `/tools/agents/android-cli` and the per-subcommand pages before relying on it.
- No official Google page fetched confirms which API levels ship `arm64-v8a` `google_apis` images in Oct 2026. Run `sdkmanager --list | grep arm64-v8a` at runtime instead of hardcoding.
- Emulator gRPC API documentation wasn't found on developer.android.com.
- The ANDROID_HOME vs ANDROID_SDK_ROOT precedence page (`/tools/variables`) wasn't fetched. ANDROID_SDK_ROOT is generally considered deprecated in favor of ANDROID_HOME **[unverified]**.

---

## 2. ADB: install, logcat, forward/reverse, am start intents (MWA)

### Takeaway
Use `adb -s <serial>` for every command; `-s` overrides `$ANDROID_SERIAL`. `adb install -t`, `am start -a VIEW -d <uri>` and `adb forward` are documented. MWA wallets are launched through a `solana-wallet:/v1/associate/local?...` VIEW/BROWSABLE intent that `sms` can fire with `am start` to smoke-test wallet handling.

### Cited Findings
- `adb [-d | -e | -s serial] command`. "If you use both `-s` and `$ANDROID_SERIAL`, `-s` overrides `$ANDROID_SERIAL`." Example: `adb -s emulator-5555 install helloWorld.apk`. — [Android: adb](https://developer.android.com/tools/adb)
- `adb devices -l` reports states `offline`, `device` and `no device`, plus product, model and device fields. Example output includes `emulator-5554 device product:sdk_google_phone_x86 ...`. — [Android: adb](https://developer.android.com/tools/adb)
- `adb install path.apk`. Test APKs need `-t`. `pm install` options include `-r` (reinstall, keeping data) and `-g` (grant all manifest permissions). Use `install-multiple` for split APKs. — [Android: adb](https://developer.android.com/tools/adb)
- `adb forward tcp:6100 tcp:7100` forwards host → device. — [Android: adb](https://developer.android.com/tools/adb)
- `am start [options] intent` with `-a action` and `-d data_uri`. — [Android: adb](https://developer.android.com/tools/adb)
- Shell quoting changed: `adb shell setprop key 'two words'` is now an error, so quote twice (`"'two words'"`). This matters when passing URIs with `&`. — [Android: adb](https://developer.android.com/tools/adb)
- MWA local association URI: `solana-wallet:/v1/associate/local?association=<association_token>&port=<port_number>&v=<version>`. The port is random in 49152–65535. The intent action is `android.intent.action.VIEW` with category `android.intent.category.BROWSABLE`. Native dapps should use `startActivityForResult`. Remote form: `solana-wallet:/v1/associate/remote?association=...&reflector=<host>&id=...&v=...`. — [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)
- The MWA repo ships sample apps `android/fakewallet` and `android/fakedapp`. Library snippets use version `2.0.7`. — [solana-mobile/mobile-wallet-adapter](https://github.com/solana-mobile/mobile-wallet-adapter)
- On Android, `connect` calls `transact` with an empty callback. The result is `Success`, `NoWalletFound` or `Failure`. "No MWA compatible wallet app found on device" is the error a user sees when no wallet is installed. — [Solana Mobile docs: Using MWA](https://docs.solanamobile.com/android-native/using_mobile_wallet_adapter)

### Inferences
- Command set for `sms` (adb reverse and logcat flags are **[unverified, from general knowledge]**):
  ```sh
  adb -s emulator-5554 install -r -t -g app-debug.apk        # sms install. -r/-g are accepted by `adb install` in practice. [unverified]
  adb -s emulator-5554 reverse tcp:8899 tcp:8899              # device localhost:8899 -> host validator RPC. [unverified]
  adb -s emulator-5554 reverse tcp:8900 tcp:8900              # websocket
  adb -s emulator-5554 reverse tcp:8081 tcp:8081              # Metro / Expo dev server
  adb -s emulator-5554 logcat -v color --pid=$(adb -s emulator-5554 shell pidof -s com.example.app)   # sms logs. [unverified]
  adb -s emulator-5554 logcat '*:S' ReactNativeJS:V MobileWalletAdapter:V   # tag filter [unverified tag names]
  adb -s emulator-5554 shell am start -a android.intent.action.VIEW -c android.intent.category.BROWSABLE \
      -d "'solana-wallet:/v1/associate/local?association=<b64>&port=55555&v=1'"   # double-quoted per adb quoting rule
  adb -s emulator-5554 shell pm list packages | grep -i wallet   # doctor: is any MWA wallet installed?
  adb -s emulator-5554 shell pm query-activities -a android.intent.action.VIEW -d solana-wallet:/v1/associate/local  # [unverified flag syntax]
  ```
- `adb reverse` is better than `10.0.2.2` for RPC URLs. The app can then use `http://localhost:8899` both on the emulator and on a USB-connected physical device, so its config never changes. Cleartext HTTP to `localhost` or `10.0.2.2` may still need an Android network-security-config exception in debug builds **[unverified]**.
- Firing a raw association intent only checks that a wallet *responds*. A real session needs the dapp-side WebSocket server on `port`, so use it in `doctor` as a "wallet installed and handles scheme" check, not as a full test.
- Build `fakewallet` from the MWA repo as an "auto-approve" test wallet for CI. It's the only official wallet sample found.

### Gaps
- `adb reverse`, `wait-for-device`, `getprop`, `emu kill` and logcat filtering syntax weren't on the fetched adb page. Verify with `adb --help` or AOSP docs.
- Whether a prebuilt APK of `fakewallet` is published, or a prebuilt Seed Vault simulator / "mock MWA wallet" APK exists, wasn't confirmed. The Solana Mobile docs pages fetched didn't cover emulator testing.

---

## 3. CI: Android emulators on GitHub Actions

### Takeaway
Use **Linux runners with KVM** and `reactivecircus/android-emulator-runner@v2` with x86_64 images. GitHub-hosted **arm64 macOS runners can't run hardware-accelerated emulators**, because Apple's Virtualization Framework doesn't support nested virtualization. So macOS-first local development and Linux CI will use different ABIs, and `sms` must make the image ABI configurable per platform.

### Cited Findings
- KVM enable step for `ubuntu-latest`:
  ```yaml
  - name: Enable KVM
    run: |
      echo 'KERNEL=="kvm", GROUP="kvm", MODE="0666", OPTIONS+="static_node=kvm"' | sudo tee /etc/udev/rules.d/99-kvm4all.rules
      sudo udevadm control --reload-rules
      sudo udevadm trigger --name-match=kvm
  - uses: reactivecircus/android-emulator-runner@v2
    with: { api-level: 29, script: ./gradlew connectedCheck }
  ```
  — [android-emulator-runner](https://github.com/ReactiveCircus/android-emulator-runner)
- Inputs: `api-level` (required), `arch` (default `x86`; `x86_64`, `arm64-v8a`), `target` (`default`, `google_apis`, `google_apis_playstore`, ...), `emulator-options` (default `-no-window -gpu swiftshader_indirect -no-snapshot -noaudio -no-boot-anim`), `disable-animations` (true), `profile`, `cores` (2), `force-avd-creation` (true), and `emulator-boot-timeout` (600 s). — [android-emulator-runner](https://github.com/ReactiveCircus/android-emulator-runner)
- AVD snapshot caching: cache `~/.android/avd/*` and `~/.android/adb*` with `actions/cache`. On a cache miss, boot without `-no-snapshot` to produce the snapshot. On later runs use `-no-snapshot-save`. This "can significantly reduce emulator startup time." — [android-emulator-runner](https://github.com/ReactiveCircus/android-emulator-runner)
- Ubuntu runners are "2-3 times faster" than macOS and macOS is "a lot more expensive". On macOS the action uses Hypervisor.Framework. Nested virtualization must be supported by the host VM. — [android-emulator-runner](https://github.com/ReactiveCircus/android-emulator-runner)
- GitHub's arm64 macOS runners (`macos-latest`, `macos-14`, `macos-15`, `macos-26`, `xcode-27`) have 3 M1 CPUs and 7 GB RAM. Nested virtualization is "not supported due to the limitation of Apple's Virtualization Framework." Intel labels are `macos-15-intel` and `macos-26-intel`; `macos-13` is no longer listed, so older advice to "use macos-13" is **outdated**. Linux arm64 runners exist: `ubuntu-22.04-arm`, `ubuntu-24.04-arm`, `ubuntu-26.04-arm`. GitHub-hosted Linux runners support hardware acceleration for Android SDK tools. — [GitHub Docs: GitHub-hosted runners](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)
- An older commit moved a project from `macos-14` to `macos-13` for emulator support. This is **outdated** now that macos-13 is gone. A commenter claims nested virtualization works on macOS 15 + M3, but this is unverified for hosted runners. — [runner-images #9254](https://github.com/actions/runner-images/issues/9254), [cross-platform-actions #82](https://redirect.github.com/cross-platform-actions/action/issues/82)

### Inferences
- CI recipe for `sms`: `ubuntu-latest` + KVM + `arch: x86_64` + `target: google_apis`, then cached snapshots, then `sms dev --ci --headless` (or the action's `script:`), then Maestro flows. Intel macOS runners (`macos-15-intel`) may work with HVF but cost more.
- Ship an `sms ci github` template generator, and let `sms dev` detect `CI=true` to switch to swiftshader, no window and longer timeouts.
- Whether `ubuntu-24.04-arm` runners expose `/dev/kvm` for arm64 images is unknown. If they do, CI could use the same ABI as Apple Silicon dev machines. Worth testing.

### Gaps
- No source gave measured boot times. The action's timeout is 600 s. Anecdotally, a cold boot is ~1–3 min on KVM Linux runners and snapshot boots take seconds **[unverified]**.
- Whether arm64 Linux runners support KVM for Android emulation wasn't confirmed.

---

## 4. Solana network from the emulator: host loopback, local validators (solana-test-validator, Surfpool), devnet RPC, faucets, Agave install

### Takeaway
From the emulator, the host is `10.0.2.2` (or use `adb reverse` so `localhost` works). Run a local validator on the host with **Surfpool** (`surfpool start`, RPC 8899 / WS 8900, can fork mainnet or devnet just-in-time). Surfpool is the officially documented drop-in replacement for `solana-test-validator`. Public devnet is heavily rate-limited (100 req/10 s per IP; faucet 2 requests/8 h), so default `sms dev` to a local Surfpool.

### Cited Findings
**Emulator networking**
- `10.0.2.0/24` virtual router: `10.0.2.1` gateway, **`10.0.2.2` = host loopback (127.0.0.1)**, `10.0.2.3` DNS, `10.0.2.15` guest (Ethernet), `10.0.2.16` guest (Wi-Fi), and `127.0.0.1` = the emulator's own loopback. After emulator 36.5, multiple emulators on Wi-Fi get .16, .17, and so on. Before 36.5, each instance had its own isolated router. — [Android: Emulator network addresses](https://developer.android.com/studio/run/emulator-networking-address)
- Network capabilities: Ethernet (API 15+), Wi-Fi (25+; P2P after emulator 36.5), Cellular (30+), Bluetooth (31+), UWB (36+). — [Android: Emulator networking](https://developer.android.com/studio/run/emulator-networking)

**Surfpool**
- Surfpool is a "drop-in replacement for `solana-test-validator`". It fetches mainnet accounts just in time, offers Infrastructure-as-Code for Anchor and Pinocchio, a Kit plugin with a pre-funded payer and typed cheatcodes, a TUI, and Rust and TypeScript SDKs. It is now hosted on solana.com docs; `docs.surfpool.run` redirects there. — [Solana docs: Surfpool](https://solana.com/docs/tools/surfpool)
- `surfpool start` flags: `-p/--port` (8899), `-w/--ws-port` (8900), `-u/--rpc-url` (fork source; env `SURFPOOL_DATASOURCE_RPC_URL`), `-n/--network mainnet|devnet|testnet`, `--offline`, `--no-tui`, `-a/--airdrop <PUBKEY>` (repeatable), `-k/--airdrop-keypair-path` (default `~/.config/solana/id.json`), `-q/--airdrop-amount` (default 10,000,000,000,000 lamports = 10,000 SOL). Studio runs on port 18488, host defaults to `127.0.0.1`, metrics on `127.0.0.1:9000`. Examples: `surfpool start --network devnet`, `surfpool start --no-tui --offline`. — [Solana docs: Surfpool CLI](https://solana.com/docs/tools/surfpool/toolchain/cli)

**Agave CLI / solana-test-validator**
- Install: `sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"`. You can use a version tag or the `stable`/`beta`/`edge` channel; the page shows `v4.4.0-beta.1`. Homebrew may lag. The prebuilt macOS tarball listed is x86_64 only. — [Anza docs: Install](https://docs.anza.xyz/cli/install)
- A local validator gives unlimited SOL: `solana-test-validator` then `solana airdrop 100 <ADDR> --url localhost`. — [Solana Faucet](https://faucet.solana.com)

**Public RPC and faucets**
- Endpoints: `https://api.mainnet.solana.com` (the page lists this host; elsewhere it's commonly `api.mainnet-beta.solana.com`), `https://api.devnet.solana.com`, `https://api.testnet.solana.com`. Per-IP limits: 100 req/10 s overall, 40 req/10 s per method, 40 concurrent connections, 40 new connections/10 s, and 100 MB/30 s. HTTP 429 means honor `Retry-After`; 403 means blocked. Not for production. — [Solana docs: Clusters](https://solana.com/docs/references/clusters)
- faucet.solana.com: "Maximum of 2 requests every 8 hours". GitHub login raises the limit. It covers devnet and testnet. Alternatives: `solana airdrop 2 <ADDR> --url devnet`, a PoW faucet (`cargo install devnet-pow`, `devnet-pow mine -d 3 --reward 0.02 --no-infer -t 5000000000`), and a local validator. The page also tells AI agents not to use the web faucet. — [Solana Faucet](https://faucet.solana.com)

### Inferences
- With `sms dev`, the host runs `surfpool start --no-tui --network <net> --airdrop <wallet-pubkey>` and adb reverses 8899 and 8900. The app config then points at `http://localhost:8899` (or `http://10.0.2.2:8899` with no reverse). Surfpool binds `127.0.0.1` by default, which is enough for both paths because the emulator's NAT and adb reverse both terminate on host loopback.
- Profiles: `localnet` (Surfpool `--offline` or `solana-test-validator`), `fork` (Surfpool forking mainnet, which gets real program state such as Jupiter or Token-2022 without deploying), and `devnet` (public RPC; warn about 429s and suggest a provider key via env).
- `sms doctor` should check `solana --version` (Agave 4.x is stable as of Oct 2026), `surfpool --version`, and that ports 8899 and 8900 are free.

### Gaps
- Surfpool's install command wasn't on the CLI page; it's at `/docs/tools/surfpool/toolchain/getting-started`, which wasn't fetched. Probably `curl -sL https://run.surfpool.run/ | bash` or `brew install txtx/taps/surfpool` **[unverified]**. Cheatcode names (for example `surfnet_setAccount`, `surfnet_timeTravel`) also weren't confirmed.
- The exact SOL amounts per devnet airdrop in 2026 weren't stated on the faucet page.
- The current Agave stable version number wasn't stated explicitly. crates.io shows 4.3.2 stable and 4.4.0-beta.1 (see §5).

---

## 5. Rust crates: ADB clients, process management, CLI/config, Solana RPC and transaction inspection

### Takeaway
`adb_client` (cocool97) is now mature and actively maintained (v3.2.3, Aug 2026, ~134k downloads) and can talk to the adb server, TCP or USB. Still, shelling out to the real `adb` binary through `tokio::process` is the most robust v1 choice, with `adb_client` as an optional pure-Rust path. For Solana, Anza's `solana-rpc-client` 4.3.x is current. Dependency weight is a real concern, so prefer the modular `solana-*` crates over the `solana-sdk` umbrella, or use raw JSON-RPC over `reqwest` for the lightest build.

### Cited Findings
- `adb_client`: newest 3.2.3 (2026-08-02). Major releases 3.0.0 (2025-12-29) and 3.1/3.2 throughout 2026. 134,181 downloads. Repo [cocool97/adb_client](https://github.com/cocool97/adb_client). — [crates.io API: adb_client](https://crates.io/api/v1/crates/adb_client)
- adb_client is a pure-Rust ADB implementation supporting the ADB server, USB and TCP/IP. It ships an `adb_cli` binary and a Python wrapper `pyadb_client`. Older lib.rs listings (v1.0.x, 2024) said USB was missing; these are **outdated**. — [lib.rs adb_client](https://www.lib.rs/crates/adb_client), [rust-digger](https://rust-digger.code-maven.com/crates/adb_client)
- Alternatives: `adb_client_tokio` (async, Tokio) — [docs.rs](https://docs.rs/adb-client-tokio). `radb_client` is a separate crate — [docs.rs](https://docs.rs/radb_client).
- ADB protocol internals reference: [Synacktiv: Diving into ADB protocol internals](https://synacktiv.com/en/node/1042)
- `solana-rpc-client`: newest stable 4.3.2 (2026-10-15). 4.4.0-beta.1 (2026-10-13) and 4.5.0-alpha.2 exist. Repo anza-xyz/agave. — [crates.io API: solana-rpc-client](https://crates.io/api/v1/crates/solana-rpc-client)
- Solana's Rust crates are published under the `solana-` prefix on crates.io and docs.rs. — [Solana docs: Rust client](https://solana.com/docs/clients/rust)
- `solana-kite` is a testing helper for LiteSVM (wallets, mints, ATAs, `deploy_program`, `send_transaction_from_instructions`, PDA helpers, Token Extensions). It is **not** an RPC client, so it doesn't fit `sms tx`. — [docs.rs solana-kite](https://docs.rs/solana-kite)

### Inferences
- Recommended crate stack (versions to pin at implementation time):
  - CLI: `clap` (derive) and `clap_complete`. Config: `serde` + `toml` (`sms.toml`: avd name, api level, abi, network profile, wallet APK, ports). Dirs: `directories`/`dirs`. Errors: `anyhow`/`miette`. TUI output: `indicatif`, `console`/`owo-colors`.
  - Processes: `tokio::process::Command` with `kill_on_drop(true)` to spawn the emulator and Surfpool. Stream stdout and stderr with `BufReader::lines()`. Write PID files under `~/.sms/run/` so `sms stop` works across invocations. Handle Ctrl-C with `tokio::signal`. Use `which` for binary lookup. **[general Rust knowledge; crate docs not fetched]**
  - ADB: v1 shells out to `adb`, which matches Flutter, Expo and RN behavior and avoids adb-server version conflicts. Optional `adb_client` feature for `devices` and `logcat` streaming without parsing text.
  - Solana `sms tx <sig>`: lightest option is `reqwest` + `serde_json` calling `getTransaction` (`encoding: jsonParsed`, `maxSupportedTransactionVersion: 0`) and `getSignatureStatuses`, then pretty-printing logs and balance deltas. Heavier option is `solana-rpc-client` + `solana-transaction-status-client-types` + `solana-signature`/`solana-pubkey`. The modular split allows this without the whole `solana-sdk`, but compile time is still large, so put it behind a Cargo feature. **[unverified; compile-time numbers not measured]**
  - Websockets for `sms logs --solana`: `logsSubscribe` via `tokio-tungstenite`, or `solana-pubsub-client`. **[unverified]**
- `mozdevice` and `forensic-adb` (Mozilla's async adb crates) weren't researched in this session; see Gaps.

### Gaps
- Maturity data for `mozdevice` and `forensic-adb` wasn't gathered.
- No measured compile times or dependency counts for `solana-rpc-client` 4.x vs raw `reqwest`. Benchmark with `cargo build --timings`.
- Whether `adb_client` 3.x supports `reverse` and `logcat` streaming natively wasn't checked in its docs.

---

## 6. Prior art (CLI emulator orchestration, doctor patterns) and UI automation for wallet approval

### Takeaway
Maestro is the best fit for automating wallet-approval taps. It uses YAML flows, needs Java 17+, installs with a one-line curl, and works on Android emulators. A `flutter doctor`-style checklist (✓/✗ plus a fix hint per check) is the pattern to copy for `sms doctor`. The new Google Android CLI (`android emulator start` blocks until ready) is itself prior art and possibly a future backend.

### Cited Findings
- Maestro is a framework for automating mobile and web UI tests with YAML flows. — [Maestro docs](https://docs.maestro.dev/)
- Install: `curl -fsSL "https://get.maestro.mobile.dev" | bash`, or `brew tap mobile-dev-inc/tap && brew install mobile-dev-inc/tap/maestro`. Windows uses a zip from GitHub releases. Needs Java 17+ with `JAVA_HOME` set. Supports `--host` to point at a remote adb server, as in the WSL setup. — [Maestro: Install CLI](https://docs.maestro.dev/maestro-cli/how-to-install-maestro-cli)
- Google's Android CLI `android emulator start` returns once the emulator is fully started, and `android info` prints SDK and environment. That is the same "doctor + blocking boot" pattern `sms` needs. — [Android CLI commands](https://developer.android.com/tools/agents/android-cli/commands)

### Inferences
- `sms doctor` checks: SDK root found; cmdline-tools/latest present; platform-tools/adb version; emulator binary and version; an arm64 image installed (on Apple Silicon); AVD exists; licenses accepted; Java ≥ 17; Hypervisor available (macOS) or `/dev/kvm` writable (Linux); `solana` and `surfpool` on PATH; ports 5554/5555/8899/8900 free; an MWA wallet installed on the device. Print a fix command per failure, as `flutter doctor -v` does. **[pattern from general knowledge of Flutter; Flutter docs not fetched]**
- Patterns from other tools **[general knowledge, not fetched]**: `flutter emulators --launch <id>` / `--create`; Expo CLI's `a` key auto-boots the first AVD and runs `adb reverse` for Metro; React Native CLI `run-android` does `adb reverse tcp:8081`; Detox `avdName` config with `-no-window` in CI; Maestro `maestro start-device --platform android` creates and boots a predefined emulator. Copy: auto-pick a device, `adb reverse` on connect, `--headless` for CI, and a "first run creates the AVD for you" flow.
- Wallet-approval automation: a Maestro flow targeting the wallet's package (for example fakewallet, or a real wallet with a test seed) with `tapOn: "Approve"` / `tapOn: "Sign"`, run via `maestro test flows/approve.yaml` while the dapp under test triggers MWA. UIAutomator (`adb shell uiautomator dump`) can serve as a zero-dependency fallback to find and tap buttons via `adb shell input tap x y`. Appium is heavier and needs a Node server, so it's not recommended. **[unverified specifics]**

### Gaps
- Maestro's flow syntax, `start-device` command and CI docs weren't fetched. The main docs page contained embedded agent instructions, which were ignored.
- Flutter, Expo, Detox and RN CLI docs weren't fetched in this session, so the specific behaviors listed above are unverified.
- No source confirmed an official Solana Mobile "test wallet" APK suitable for CI auto-approval beyond the `fakewallet` sample source.
