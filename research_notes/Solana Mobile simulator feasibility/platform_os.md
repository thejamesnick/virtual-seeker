# Solana Mobile Platform / OS (Saga and Seeker): What Is Custom vs Stock Android, and What a Simulator Must Reproduce

Research date: 2026-10-18. Items dated before 2026 are marked [older]. Where a page's dates looked inconsistent, that is noted.

## 1. What OS do Saga and Seeker run, and which Solana Mobile Stack (SMS) parts are system-level vs ordinary APKs?

### Takeaway
Both phones run Android, not a fork. Seeker launched on Android 15 and moved to Android 16 in the MR7 update (around March 2026). Saga's last version was Android 14, and it reached end of support in October 2025. Solana Mobile's customization is a thin layer on top: the Seed Vault system service, the built-in Seed Vault Wallet, the built-in Solana dApp Store, and device-identity assets (Seeker Genesis Token, .skr domain). Ordinary dApps use standard Android APIs plus the Mobile Wallet Adapter (MWA) protocol.

### Cited Findings
- Seeker is described as Solana Mobile's flagship Android device, "designed from the ground up for crypto," and as the Saga's successor. Its parts are Seed Vault, Genesis Token, .skr Domain and the dApp Store. The docs say "testing on any Android device is sufficient," so a Seeker is not needed for development. — [Solana Mobile Docs: Seeker](https://docs.solanamobile.com/solana-mobile-stack/seeker.md)
- Seeker MR7 system update: "Updated the system to Android 16", with March 2026 Android and vendor security patches. Size is about 1.65 GB (Global) or 1.66 GB (EEA). Builds are `Seeker.260327.001` (Global) and `Seeker.260328.001` (EEA). It also "Updated the built-in Solana dApp Store and Seed Vault Wallet apps to their latest versions." — [Seeker Release Notes MR7](https://docs.solanamobile.com/seeker/release-notes/mr7.md)
- Seeker MR8, the latest release note found, is build `Seeker.260612.001` (Global, serials starting SM02G) and `Seeker.260616.001` (EEA, serials starting SM02E). It carries April and May 2026 security patches, "Seed Vault: Miscellaneous bug fixes", and an 80 MB update. — [Seeker Release Notes MR8](https://docs.solanamobile.com/seeker/release-notes/mr8.md)
- Earlier release notes: MR6 fixed a bug that could rename stored seeds, and MR5 fixed seeds not showing after setup. These Seed Vault fixes ship through OTA system updates, which suggests Seed Vault is part of the system image. — [MR6](https://docs.solanamobile.com/seeker/release-notes/mr6.md), [MR5](https://docs.solanamobile.com/seeker/release-notes/mr5)
- Seeker launched on Android 15. This comes from retail listings and press rather than an official spec sheet. Hardware: MediaTek Dimensity 7300, 8 GB RAM, 128 GB storage, 6.36" 120 Hz AMOLED, 4500 mAh battery, fingerprint sensor, Nano-SIM plus eSIM. — [finn.no listing](https://www.finn.no/recommerce/forsale/item/449195337); the Dimensity 7300 chip and 192 g weight are confirmed by [Blockworks review](https://blockworks.com/news/seeker-review-solana-mobile) [older, 2025]
- Seeker's React Native `Platform.constants` (from `android.os.Build`) report `Model: "Seeker"`, `Brand: "solanamobile"`, `Manufacturer: "Solana Mobile Inc."` and a `Fingerprint` string. — [Detecting Seeker Users](https://docs.solanamobile.com/recipes/general/detecting-seeker-users)
- Saga was "last upgraded to Android 14", with a final security patch from November 2024. [older] — [Decrypt, 2025-10-21](https://decrypt.co/345262/solana-saga-loses-security-updates-support-seeker-phone-rollout)
- Seed Vault is "a system service providing secure key custody to Wallet apps." The Seed Vault Wallet is "the built-in wallet app on Seeker that interfaces directly with Seed Vault", created during device onboarding. — [Seed Vault docs](https://docs.solanamobile.com/solana-mobile-stack/seed-vault.md)
- The dApp Store is "the crypto-friendly app store for Solana Mobile devices" and supports native Android apps and wrapped web apps (PWA to APK). — [dApp Store intro](https://docs.solanamobile.com/dapp-store/intro.md)
- Seeker system apps (from the October 2026 announcement): "Seeker app" will merge the Solana dApp Store and the wallet into one app. It is announced for Ledger Atom. Whether Seeker itself will consolidate the same way is not stated. — [PR Newswire, 2026-10-15](https://www.prnewswire.com/news-releases/solana-mobile-announces-ledger-as-its-first-hardware-expansion-partner-302909677.html)
- SeekerClaw is "an open-source AI agent that lives on your Seeker 24/7" that can "monitor your wallet, swap on Jupiter, control your camera, launch apps". It is distributed through the dApp Store, so it is a regular APK, not a system component. — [Solana Mobile blog: SMS goes global](https://solanamobile.com/blog/sms-goes-global-and-season-2-wrapup) (the page header shows Jul 14, 2026 and its metadata shows Oct 15, 2026, so the date is unclear)

### Inferences
- For a simulator, an emulator on a stock AOSP or Google APIs image at API 35 or 36 (Android 15 or 16) matches Seeker's OS baseline. Nothing in the sources points to custom framework APIs beyond Seed Vault's content provider and intents. To mimic Seeker identity, the simulator can override `ro.product.model=Seeker`, `ro.product.brand=solanamobile` and `ro.product.manufacturer=Solana Mobile Inc.` in a custom AVD or system image. This makes `Build.MODEL` checks pass, and the docs openly call those checks spoofable.
- Seed Vault (system service), Seed Vault Wallet and the dApp Store are updated through OTA system images (MR7 explicitly "built-in"). So on real devices they are privileged or system apps, not user-installed APKs. A simulator can still install equivalents as normal APKs, because the public Seed Vault simulator runs as an ordinary app on API 31+ (see section 2).

### Gaps
- No official Seeker spec sheet giving the exact launch Android version was found. "Android 15 at launch" rests on listings and press. The MR7 Android 16 upgrade is official.
- The release date of MR8 is not on the page. It is inferred from the build string (June 2026). No MR9 or later notes were found as of October 2026.
- Package names, the signature or privileged-permission allowlist, and the SELinux policy for Seed Vault on production devices were not found in public docs.
- Saga hardware details (commonly cited as Snapdragon 8+ Gen 1, launched 2023) were not verified against a source in this session.

## 2. Seed Vault on the device: integration, TEE or secure element, wallets that use it

### Takeaway
Seed Vault is an Android system service that holds BIP-39 seeds and signs inside the device's secure execution environment, with keys held by the Android Keystore. Every signature needs user authorization through system UI using biometrics or PIN. Wallets reach it through a public Wallet API made of a content provider and intents. Solana Mobile ships an official Seed Vault simulator and a fake wallet that run on any Android API 31+ device. That simulator is the obvious base for an open-source Solana Mobile Simulator. Real hardware-backed security cannot be reproduced and should not be claimed.

### Cited Findings
- "Your keys, seeds, and secrets never leave the secure execution environment." Seed Vault uses "secure processor modes and/or auxiliary coprocessors" and the Android Keystore backed by that environment. "Every signing operation requires explicit user approval via secure biometric fingerprint or PIN code entry", using Android-built UI components. — [Seed Vault docs](https://docs.solanamobile.com/solana-mobile-stack/seed-vault.md)
- dApp developers use Seed Vault indirectly via MWA. The Seed Vault SDK is aimed at wallet developers and "also includes a simulator for testing on standard Android devices." — [Seed Vault docs](https://docs.solanamobile.com/solana-mobile-stack/seed-vault.md)
- The SDK repo contains an integration guide, the Wallet API, a simulator implementing the Seed Vault Wallet interfaces, and a fake wallet. Maven artifact: `com.solanamobile:seedvault-wallet-sdk:0.4.0` (the version in the README example; the latest release was not confirmed). Warning: "this is a simulator, and makes zero guarantees about security." — [GitHub: solana-mobile/seed-vault-sdk](https://github.com/solana-mobile/seed-vault-sdk)
- From the integration guide:
  - The simulator needs Android API 31+ and gives a UI for creating seeds from BIP-39 phrases plus a reference Wallet API implementation.
  - Derivation uses BIP-32 and SLIP-0010, with common paths `m'/44'/501'/X'` and `m/44'/501'/X'/0'` pre-derived when a seed is authorized. Non-standard paths need password or biometric authorization on first use.
  - Signing flow: `Wallet.signTransaction(...)` or `ACTION_SIGN_TRANSACTION`, then a Seed Vault approval screen with password or biometrics, then `onSignTransactionResult`.
  - Implementations store at least 8 seeds, and limits can be queried through an `implementationlimits` table.
  - The guide (written before Saga shipped) says the simulator could be used until Saga was available. [older]
  — [Seed Vault integration guide](https://raw.githubusercontent.com/solana-mobile/seed-vault-sdk/main/docs/integration_guide.md)
- Wallets: on Seeker, the built-in Seed Vault Wallet holds the user's Seeker Genesis Token. — [Seed Vault docs](https://docs.solanamobile.com/solana-mobile-stack/seed-vault.md). Saga seeds follow the "same seed standard" and can be restored elsewhere. [older] — [Decrypt](https://decrypt.co/345262/solana-saga-loses-security-updates-support-seeker-phone-rollout)
- OEM version of SMS (MWC, March 2026): SMS includes TEE support through a Trustonic partnership. MediaTek opened its development platform so SMS is production-ready on Dimensity chipsets, and Qualcomm chipsets are also supported. The stack is designed not to affect payments certification, Google Mobile Services (GMS) or security approvals. These details come from search-result summaries of the company blog and press; the primary blog page returned 404 when fetched. — [Solana Mobile blog: Introducing SMS for OEMs](https://blog.solanamobile.com/post/introducing-the-solana-mobile-stack-for-oems-and-hardware-manufacturers); [eGamers summary](https://egamers.io/solana-mobile-opens-stack-access-to-all-android-oems-with-seekerclaw-ai-integration-in-seeker-phone/)
- The SMS for OEMs bundles "Seed Vault, Seeker Wallet, and dApp Store", "now available as a tech stack for any Android phone manufacturer." — [Solana Mobile blog: SMS goes global](https://solanamobile.com/blog/sms-goes-global-and-season-2-wrapup)

### Inferences
- The simulator can reuse or wrap the official Seed Vault simulator (Apache-licensed SDK repo) so that wallets written against the Wallet API work unchanged. A software keystore or Keystore emulation stands in for the TEE. It should be clearly labeled insecure and test-only, in line with the upstream warning.
- Biometric or PIN prompts can be reproduced with the emulator's fingerprint simulation (`adb emu finger touch`) or a PIN, but they will not have hardware-backed attestation.
- On real devices the Trustonic TEE (Kinibi) on MediaTek parts suggests Seed Vault depends on vendor trusted apps. This cannot be emulated, and fidelity to TEE behavior should be explicitly out of scope.

### Gaps
- The exact permission names (for example `com.solanamobile.seedvault.ACCESS_SEED_VAULT` and any privileged variant) were not confirmed in the pages fetched.
- Whether the Seeker Seed Vault runs in a Trustonic trusted app versus Keystore/StrongBox was not confirmed for Seeker specifically. The Trustonic statement concerns the OEM SMS offering.
- No public list of third-party wallets (Phantom, Solflare, etc.) integrating Seed Vault on Seeker was found in this session.

## 3. The Solana dApp Store: how it works, publishing, and use on an emulator

### Takeaway
The dApp Store is a Solana Mobile-built, system-preinstalled Android store for native APKs and wrapped web apps. Publishing is on-chain: an "App NFT" is minted, and releases are submitted through the Publisher Portal (publish.solanamobile.com) or the `@solana-mobile/dapp-store-cli` with a Solana keypair and portal API key. The docs say developers need no Solana device. No official way to install the dApp Store client itself on an emulator was found, so a simulator should mock or omit it and keep only the publishing flow (CLI and portal) available against real services.

### Cited Findings
- Publishing CLI: package `@solana-mobile/dapp-store-cli`, command `dapp-store`, flags `--apk-file`/`--apk-url`, `--keypair` (required), `--whats-new`, and an API key via `DAPP_STORE_API_KEY`. The CLI reads the package name from the APK and matches it to a portal app. Requirements: an app created in the portal with its App NFT minted, a release-signed APK, a Solana keypair and a portal API key. The Publisher Portal is at publish.solanamobile.com. — [Publishing CLI docs](https://docs.solanamobile.com/dapp-store/publishing-cli.md)
- Debug builds are not accepted, and submissions must be signed release builds. — [dApp Store checklist (via search)](https://docs.solanamobile.com/dapp-store/checklist)
- You don't need a Solana Mobile device to build dApp Store apps, and a setup guide covers Android devices and emulators. — [dApp Store intro](https://docs.solanamobile.com/dapp-store/intro.md)
- Store scale: "over 1,500 apps" (October 2026 press release) versus "500+ dApps" (MWC blog) and "over 225 dApps" (Breakpoint, December 2025 [older]). The figures differ by source and date. — [PR Newswire](https://www.prnewswire.com/news-releases/solana-mobile-announces-ledger-as-its-first-hardware-expansion-partner-302909677.html); [SMS goes global](https://solanamobile.com/blog/sms-goes-global-and-season-2-wrapup); [Solana Compass Breakpoint 25 recap](https://solanacompass.com/learn/breakpoint-25/update-from-solana-mobile-the-seeker-economy)
- Additional doc pages exist for build-and-sign, publishing from Google Play, submitting updates, linking to listings and the publisher policy (not fetched in detail). — [docs index](https://docs.solanamobile.com/llms.txt)

### Inferences
- Publishing is mainnet-oriented (App NFT, portal API key). A simulator would not reproduce publishing. At most it might offer a "local dApp Store" catalog UI that installs APKs from a local registry for testing deep links and discovery.
- Because the dApp Store client ships as a built-in system app and has no documented sideload path, distributing its APK in an open-source simulator is likely not allowed (license unknown) and should be avoided.

### Gaps
- Whether the dApp Store client APK runs on non-Solana devices or emulators: no source found.
- Details of the NFT model (Publisher, App and Release NFTs, Arweave storage) were not confirmed in the pages fetched this session.
- Fees and review SLAs were not found.

## 4. Seeker-specific features: Genesis Token, .skr / Seeker ID, TEEPIN, SKR, attestation, and access from non-Seeker devices or emulators

### Takeaway
Seeker-specific identity is anchored in mainnet on-chain assets minted once per device into the Seed Vault Wallet: the Seeker Genesis Token (SGT), a Token-2022 NFT in group `GT22s89n...`, and a .skr domain on AllDomains. Any device or server can read them through the `@solana-mobile/skr-genesis-token` and `@solana-mobile/skr-domain` npm packages, but they can only be minted on a real Seeker. TEEPIN is a described three-layer architecture (hardware TEE attestation, platform on-chain verification, Guardian network). No public developer-facing device attestation API was found. SKR is a mainnet SPL token with a staking program. An emulator can read all these on-chain assets but cannot legitimately obtain an SGT or .skr domain.

### Cited Findings
- **SGT:** "can only be minted once per device", minted into the primary account of the Seed Vault Wallet. It uses Token Extensions (Metadata Pointer, Token Group Member and Pointer). The group and metadata address is `GT22s89nU4iWFkNXj1Bw6uYhJJWDRPpShHt4Bk8f99Te`. It is transferable only on a permissioned basis between the user's own Seed Vault Wallet accounts, and the mint address persists. Verification uses `@solana-mobile/skr-genesis-token` (`getSeekerGenesisTokens`, `hasSeekerGenesisToken`, `isSeekerGenesisToken`) against a mainnet RPC. — [SGT docs](https://docs.solanamobile.com/solana-mobile-stack/seeker-genesis-token.md)
- The recommended anti-sybil check combines SIWS, using MWA `authorize` with `sign_in_payload` and `chain: "solana:mainnet"` verified server-side with `verifySignIn` and domain and nonce binding, with SGT ownership and uniqueness tracked by mint address. The `Platform.constants.Model === "Seeker"` check "can be spoofed." — [Detecting Seeker Users](https://docs.solanamobile.com/recipes/general/detecting-seeker-users)
- **.skr domain:** every Seeker owner receives one, minted once per device and built on AllDomains. Package `@solana-mobile/skr-domain` provides `resolveSkrDomain`, `getPrimarySkrDomain` and `getSkrDomains` (the last uses `getProgramAccounts`, so a backend is recommended). React Native needs a WebCrypto polyfill. The docs page does not use the term "Seeker ID". — [.skr Domain docs](https://docs.solanamobile.com/solana-mobile-stack/skr-domain.md)
- **SKR:**
  - Mint `SKRbvo6Gf7GondiT3BbTfuRDPqLWei4j2Qy2NPGZhW3` (SPL Token).
  - Holders stake and delegate to Guardians. Inflation starts at 10%, decays 25% per year and settles at 2% terminal. Epochs last 2 days, with a 48-hour withdrawal cooldown.
  - Staking Program `SKRskrmtL83pcL4YqLWt6iPefDqwXQWHSw9S9vz94BZ`, Stake Config `4HQy82s9CHTv1GsYKnANHMiHfhcqesYkK6sB3RDSYyqw`, Stake Vault `8isViKbwhuhFhsv2t8vaFL74pKCqaFPQXo1KkeQwZbB8`, Solana Mobile Guardian Pool `DPJ58trLsF9yPrBa2pk6UaRkvqW8hWUYjawe788WBuqr`.
  — [SKR docs](https://docs.solanamobile.com/solana-mobile-stack/skr.md)
- SKR launch: announced for January 2026 with a 10 billion supply, 30% for airdrops and 25% for growth and partnerships. One secondary source reports about 2 billion SKR airdropped to 100,908 Seeker users and 188 developers on January 21, 2026. That figure is unconfirmed and comes from a lower-reliability exchange news aggregator. — [MEXC news](https://www.mexc.com/news/541008); see also [The Block on SKR plans](https://www.theblock.co/post/355238/solana-mobile-skr-token-seeker-shipping-date) [older, 2025]
- **TEEPIN** (Trusted Execution Environment Platform Infrastructure Network) has three layers:
  - Hardware layer: device TEEs attest identity, boot status and software authenticity through cryptographic proofs.
  - Platform layer: devices "prove their legitimacy by validating their credentials against blockchain records" at boot, and apps follow similar verification paths.
  - Network layer: a Guardian Network validates devices, verifies apps and governs policy.
  - The page specifies no developer API, attestation format or keys.
  — [Solana Mobile blog: The Web3 Mobile Evolution](https://solanamobile.com/blog/solana-mobile-the-web3-mobile-evolution) (originally published in 2025 [older]; the current page shows conflicting header and metadata dates of Jul 14, 2026 and Oct 15, 2026); also [The Defiant](https://thedefiant.io/news/nfts-and-web3/solana-seeker-web3-phone-to-launch-in-august-with-native-token) [older]
- Breakpoint 2025 named Helius and Jito as Guardians and mentioned the MediaTek partnership. [older, December 2025] — [Solana Compass recap](https://solanacompass.com/learn/breakpoint-25/update-from-solana-mobile-the-seeker-economy)

### Inferences
- Simulator approach:
  - **SGT and .skr:** reproducing them on mainnet is impossible and undesirable, because they are anti-sybil proofs. The simulator can deploy look-alike devnet or localnet mints (a Token-2022 group plus member NFT, and a mock domain registry) and give developers a configuration switch so their verification code points at the mock group address. The `skr-genesis-token` package hard-codes the mainnet group, so the simulator would need a shim or a fork, or developers would need to inject the group address.
  - **SKR:** the staking program could be cloned to localnet using a validator's `--clone` from mainnet if the program is upgradeable or dumpable. This should be checked.
  - **TEEPIN:** device attestation cannot be faithfully emulated. The simulator should expose a clearly fake "attestation" stub, or nothing, since no public attestation API was found.
- A non-Seeker Android phone or emulator can call every read API (SGT check, .skr resolution, SKR balances) because they are plain RPC reads. It cannot mint an SGT or .skr domain.

### Gaps
- No public "Seeker device attestation API" (Android Key Attestation wrapper, TEEPIN SDK, Guardian verification endpoint) was found in docs.solanamobile.com or in press.
- No devnet deployments of the SGT group, .skr registry or SKR staking were documented.
- How the SGT mint is gated on-device (which attestation the mint authority checks) is not public.
- No official source defines "Seeker ID" as distinct from the .skr domain.

## 5. Hardware-dependent APIs that cannot be emulated faithfully

### Takeaway
The parts that cannot be faithfully emulated are those rooted in device hardware:
- TEE/Trustonic-backed Seed Vault key custody and secure-UI approval
- hardware-backed Keystore/StrongBox and key attestation
- TEEPIN device attestation (boot state, software authenticity)
- once-per-device minting of the SGT and .skr domain (gated by that attestation)
- fingerprint biometrics with hardware trust

Everything a typical dApp touches can be emulated with high fidelity: MWA, SIWS, signing, RPC, deep links and `Build` properties. Solana Mobile's own tooling already treats an emulator with a fake wallet as the supported development path.

### Cited Findings
- Seed Vault keys never leave the secure execution environment, and signing requires biometric or PIN approval via system UI. — [Seed Vault docs](https://docs.solanamobile.com/solana-mobile-stack/seed-vault.md)
- The official simulator "makes zero guarantees about security." — [seed-vault-sdk](https://github.com/solana-mobile/seed-vault-sdk)
- TEEPIN's hardware layer relies on TEE-generated proofs of identity, boot status and software authenticity. — [Solana Mobile blog](https://solanamobile.com/blog/solana-mobile-the-web3-mobile-evolution)
- The `Build.MODEL` check is spoofable, and only SGT plus SIWS is described as a guarantee. — [Detecting Seeker Users](https://docs.solanamobile.com/recipes/general/detecting-seeker-users)
- **Existing official tooling overlaps with the planned simulator.** Solana Mobile ships a `solana-mobile` CLI (`npx solana-mobile@latest`), which matters for scoping and avoiding duplicate work. Commands:
  - `emulator create/start/stop/list/status/images/tune` manage AVDs and system images without Android Studio.
  - `device install` installs from a catalog that includes "the Mobile Wallet Adapter test wallet (`fakewallet`)", with options `--grant` and `--list`.
  - `device tune` disables animations, autofill, the lockscreen and similar for automation.
  - `playground` serves a bundled MWA test page through `adb reverse` on localhost:4747 to exercise connect, SIWS, sign message, sign transaction, and sign and send on devnet (default), testnet, localnet (via the CLI's `localnet` command, with a 1 SOL airdrop button) or mainnet (custom `--url` required).
  — [CLI emulator](https://docs.solanamobile.com/cli/emulator.md); [CLI device](https://docs.solanamobile.com/cli/device.md); [CLI playground](https://docs.solanamobile.com/cli/playground.md)
- The CLI docs pages fetched do not mention a Seed Vault simulator, a dApp Store, or Seeker identity spoofing (SGT, model properties). — [CLI device](https://docs.solanamobile.com/cli/device.md); [CLI emulator](https://docs.solanamobile.com/cli/emulator.md)
- Ledger Atom (announced October 2026) uses AOSP plus an EAL6+ Secure Element running Ledger OS, with display and touch physically isolated from Android in a "Secure Mode" for signing, and anti-tamper features. This adds another hardware trust path SMS must support on partner devices. — [PR Newswire](https://www.prnewswire.com/news-releases/solana-mobile-announces-ledger-as-its-first-hardware-expansion-partner-302909677.html); [The Block](https://www.theblock.co/news/business/2026-10-15-ledger-atom-solana-mobile-418644)

### Inferences
- The proposed open-source simulator's differentiation over the official `solana-mobile` CLI would be:
  - a Seed Vault simulator integrated as the wallet backend
  - Seeker-identity spoofing (Build props, mock SGT and .skr on devnet or localnet)
  - a mock dApp Store catalog
  - a one-command bundle

  Emulator management, fakewallet install and the MWA playground already exist upstream and could be reused rather than rebuilt.
- Out of scope, and the simulator should say so: TEE, Secure Element, attestation, secure-UI/trusted display (Ledger Atom), real Seed Vault package and privileged-permission behavior, OTA updates, and genuine SGT or .skr minting.

### Gaps
- Whether the `solana-mobile` CLI catalog includes the Seed Vault simulator APK (`--list` output not retrieved).
- Whether a Play Integrity or Key Attestation check is used by any Solana Mobile service (for example the SGT mint or the dApp Store) was not found.

## 6. Status as of October 2026: Seeker shipping, Saga end-of-life, third devices and OEM licensing

### Takeaway
Seeker has been shipping since August 2025 and is still supported. The latest OTA found is MR8 (June 2026 builds, Android 16 since MR7). Saga reached end of support on 2025-10-21. No third Solana-branded phone has been announced. Instead, in March 2026 at MWC, Solana Mobile opened SMS to any Android OEM. On 2026-10-15 it named Ledger the first hardware expansion partner: the Ledger Atom ($849 per press, shipping summer 2027) will preinstall a "Seeker app" that merges the dApp Store and wallet.

### Cited Findings
- Seeker began shipping on August 4, 2025, to over 50 countries. [older] — [The Block, 2025-08-05](https://www.theblock.co/news/business/2025-08-05-solana-mobile-seeker-crypto-smartphone-365600); [Cointelegraph](https://cointelegraph.com/news/solana-mobile-device-seeker-ships-50-countries)
- Seeker sold "over 200,000 devices", with more than $13B in transaction volume and 154 million transactions (company claims, October 2026). — [PR Newswire, 2026-10-15](https://www.prnewswire.com/news-releases/solana-mobile-announces-ledger-as-its-first-hardware-expansion-partner-302909677.html). Earlier figures were 150,000 devices at Breakpoint, December 2025 [older] — [Solana Compass](https://solanacompass.com/learn/breakpoint-25/update-from-solana-mobile-the-seeker-economy). The MWC blog says "200K+ devices, $5B+ onchain volume". — [SMS goes global](https://solanamobile.com/blog/sms-goes-global-and-season-2-wrapup)
- Saga end of support: "The Solana Mobile Saga has reached the end of its support lifecycle". There are no more software or security updates, and the page says Seeker "will continue to get software updates and security patches." The page is dated 2025-10-21. — [Solana Mobile Support](https://support.solanamobile.com/en/articles/12600995-saga-end-of-support); [DL News](https://dlnews.com/articles/web3/solana-quietly-ends-support-for-saga-mobile-phone-just-two-years-after-launch/)
- OEM licensing: in March 2026 at MWC Barcelona, Solana Mobile opened the Solana Mobile Stack to Android manufacturers. Before this, SMS powered only Saga and Seeker. OEMs can configure the wallet, curate apps and tailor onboarding by market. — [PR Newswire](https://www.prnewswire.com/news-releases/solana-mobile-announces-ledger-as-its-first-hardware-expansion-partner-302909677.html); [Solana Mobile blog](https://blog.solanamobile.com/post/introducing-the-solana-mobile-stack-for-oems-and-hardware-manufacturers) (via search summary)
- Ledger Atom: announced 2026-10-15 at Ledger Op3n Paris, with the Seeker app preinstalled and on the home screen. It is AOSP-based with an EAL6+ Secure Element, ships summer 2027, and reservations are at ledger.com/seeker. — [PR Newswire](https://www.prnewswire.com/news-releases/solana-mobile-announces-ledger-as-its-first-hardware-expansion-partner-302909677.html); [Solana Compass news](https://solanacompass.com/news/solana-mobile-names-ledger-its-first-hardware-expansion-partner-preinstalling-the-seeker-app-on-ledger-atom). The $849 price comes from [AltcoinBuzz](https://www.altcoinbuzz.io/ledger-unveils-849-atom-with-solana-s-seeker-app-preinstalled) (secondary).

### Inferences
- The ecosystem's direction is from "two phones" to "SMS on many Android devices plus a consolidated Seeker app". That makes a hardware-agnostic simulator more relevant: dApps will increasingly run on varied OEM hardware, and Seed Vault backends will vary (Trustonic TEE, Ledger SE).
- Simulator profiles worth offering: "Seeker (Android 16, API 36)", "Saga (Android 14, API 34, EOL)", and possibly "generic SMS OEM".

### Gaps
- No announced Seeker successor or third Solana-branded phone was found as of 2026-10-18.
- No named OEM other than Ledger has publicly shipped SMS. MediaTek is a chipset or platform partner, not a device OEM.
- No 2026 Seeker unit-shipment breakdown beyond the company's cumulative "200,000+ sold".
