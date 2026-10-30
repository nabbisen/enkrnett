# RFC 017 — Operator UI

**Status.** Accepted (2026-10-31)
**Tracks.** The management UI as the operator experiences it: screens, words in English and
Japanese, behaviour. RFC 001 open question 1 (Japanese terms); RFC 011, 012, 015 open questions
on wording; RFC 009 table C as a hard rule; review 13 suggestions S-1 to S-4; gap analysis
GAP-070 (no UI design existed).
**Touches.** `ui/` (Svelte 5 + TypeScript); the two language catalogues; the word check in CI;
the browser harness; the guides (which reuse these sentences); RFC 001 §4 (display forms, see
Consequences); RFC 011 §8 (one sentence).

Terms are defined in [RFC 001](./001-scope-people-vocabulary.md) as amended by
[RFC 009](./009-vocabulary-amendment-operator-words.md) and
[RFC 016](./016-amendments-review-001-015.md). The API the UI speaks is
[RFC 015](./015-management-api.md). Delivery and request security:
[RFC 003](./003-management-channel-and-ui-delivery.md). The claim:
[RFC 004](./004-ownership-credentials-authorization.md). Changes:
[RFC 011](./011-configuration-model-and-transactions.md). Guest modes and the notice:
[RFC 012](./012-owe-modes-and-status.md). Update:
[RFC 013](./013-build-release-update.md).

## Summary

Fourteen screens, two languages, one catalogue of sentences per language, and nothing on a screen
that a decision does not need. Every word the operator reads is in this RFC, in English and in
Japanese, with the API value or RFC sentence it renders. Every state and error code of RFC 015
maps to exactly one sentence. The operator never meets a standard's name, a status that
summarises protection as good or bad, or two words for one thing. The phone's own Wi-Fi screen
and Enkrnett's screen use the same words for the same things, because the operator moves between
them during setup and after every change of the management Wi-Fi.

## Motivation

| Fact | Source |
|---|---|
| The operator is not an IT specialist and installs, sets up, and runs the device alone. | RFC 001 §3; D-02 |
| The accepted RFCs fix the English sentences for changes (RFC 011 §8), guest status (RFC 012 §3, §4), and the channel (RFC 003 §5), and leave Japanese and the remaining screens to this RFC. | Those RFCs |
| A page served over HTTP from a private address has no service worker, no persistent storage it can rely on, no camera, no clipboard API. | RFC 003 ENK-MGMT-003 |
| The whole UI bundle, both languages included, must fit in 150 KiB of squashfs. | RFC 010 ENK-FOOT-002 |
| Phones call the thing you type to join a Wi-Fi network a *password*. The operator types the generated management key into that field. | iOS and Android join dialogs |
| 初期化 (Erase) and 初期設定 (a common rendering of "initial setup") share their first two characters; a stressed operator could confuse them. | Japanese usage |
| The guest notice must be printable and must say the same as the status screen. | RFC 012 ENK-OWE-018, ENK-GUEST-001 |
| Non-experts have not yet used any of this; M-7 is blocked by hardware and the legal answer. | ROADMAP |

## Design

### 1. Principles of the screens

| # | Rule |
|---|---|
| 1 | **The status screen is the first screen** after login and the one the operator returns to. It answers "is my guest Wi-Fi working, and for whom?" with no tap (RFC 012 ENK-OWE-016). |
| 2 | **One decision per screen** in setup; one action per screen for everything that disturbs the Wi-Fi or cannot be undone. The consequence is stated beside the button, in one sentence. |
| 3 | **Words come from §2.** The phone's Wi-Fi screen and Enkrnett's screen call the same thing by the same name. |
| 4 | **A number is explained before it is typed.** Name limits and password rules appear above the field, not after an error. |
| 5 | **Colour means running or not running**, nothing else (RFC 012 ENK-OWE-017). |
| 6 | **The screen never waits silently.** A countdown shows the seconds the device gave; a poll that fails shows the "not connected" screen and continues by itself. |
| 7 | **No technical names** except the two allowed in the guest notice (RFC 009 table C). |

### 2. Words: display forms and Japanese terms

RFC 001 §4 defines the terms; this section fixes what the operator sees. A display form differs
from the term only where the operator's other screens (the phone's Wi-Fi settings) use a
different word. Identifiers and RFC text keep the terms.

| Term (RFC 001/016) | English on screen | Japanese on screen | Note |
|---|---|---|---|
| Guest network | guest Wi-Fi | ゲストWi-Fi | Phones say "Wi-Fi"; "network" is abstract |
| Management network | management Wi-Fi | 設定用Wi-Fi | Japanese: "the Wi-Fi for settings"; avoids 管理 twice (see owner password) |
| Setup network | setup Wi-Fi | セットアップ用Wi-Fi | Not 初期設定用: too close to 初期化 (Erase) |
| Uplink | Internet connection | インターネット接続 | The cable; the operator never hears "uplink" |
| Site network | the shop's or venue's own network | お店や施設のネットワーク | In the notice and the separation text |
| Device | this device | 本体 | |
| Owner | owner | 管理者 | 所有者 sounds legal; 管理者 is what Japanese router screens say for the person who manages |
| Owner password | owner password | 管理者パスワード | |
| Management key | management Wi-Fi password | 設定用Wi-Fiのパスワード | The phone's join dialog asks for a *password*; the operator must recognise it as the same thing |
| Management UI | settings screen | 設定画面 | |
| Settings (RFC 016) | Settings | 設定 | |
| Change (RFC 016) | change | 変更 | |
| Status (RFC 016) | Status | 状態 | |
| Details (RFC 016) | Details | 詳細 | |
| Install | install | インストール | |
| Set up / claim | set up | セットアップ | "claim" never appears |
| Update (and upgrade) | Update | アップデート | One word (RFC 013 ENK-UPD-007) |
| Erase | Erase | 初期化 | RFC 009 |
| Restart | Restart | 再起動 | RFC 009 |
| Country | country or region | 国・地域 | As phones say |
| Network name | name | 名前 | In context: "name of the guest Wi-Fi" |
| Guest's device | device | 端末 | Counts: "12 devices" / 12台 |
| Encrypted / not encrypted | encrypted / not encrypted | 暗号化あり / 暗号化なし | |
| Running / not running | running / stopped | 稼働中 / 停止中 | Colour only here |
| Off by the operator | off | オフ | `settings.guest.enabled = false` |
| Open to every device | Open to every device | すべての端末が接続できる | RFC 012 |
| Encrypted only | Encrypted only | 暗号化できる端末のみ | RFC 012 |
| Save / saved / undone | Save / Saved / Undone | 保存 / 保存しました / 元に戻しました | RFC 009 table C |
| Address | address | アドレス | The management UI's address |
| QR code | QR code | QRコード | |
| Light / button | light / button | ランプ / ボタン | |
| Log in / log out | Log in / Log out | ログイン / ログアウト | |

| ID | Requirement | Verification |
|---|---|---|
| ENK-UI-008 | Operator-facing text shall use only the display forms of this table for the things it names, in both languages, and shall contain none of the words of RFC 009 table C nor their Japanese equivalents (§8). | Word check in the UI build (ENK-WORD-001); review |
| ENK-UI-009 | Each language shall be one catalogue of complete sentences with named placeholders. No sentence shall be assembled from fragments at run time. Counts use the language's own rule (English 1 / other; Japanese none). | Inspection of the catalogue; test with 0, 1, 2 |

### 3. Languages

| ID | Requirement | Verification |
|---|---|---|
| ENK-UI-010 | The first release ships English and Japanese. The UI shall start in the browser's language when it is one of them, otherwise in English, and shall offer the switch on every screen's header. The choice may be remembered in the browser as a convenience; the UI shall work when nothing is remembered (RFC 003 ENK-MGMT-003). | Test with each browser language; test with storage blocked |
| ENK-UI-011 | Both catalogues shall be reviewed by a native reader of the language before a release, against this RFC's tables. | Release checklist |

### 4. Screens

Fourteen screens. "Reads" and "Acts" name RFC 015 resources.

| # | Screen | State | Reads | Acts | Content |
|---|---|---|---|---|---|
| S-1 | Start | `SETUP` | greeting | `claim/start` | "Set up this device. About five minutes." The connection line (§5.4). One button: *Start*. If another client holds the claim: the `setup_in_progress_elsewhere` sentence. |
| S-2 | Is this the device? | `SETUP` | `claim` | `claim/identify` | "Press *Blink* and look at the device's light." Buttons *Blink*, *I saw it blink*. Without a light: "This device has no light Enkrnett can control. Check that the setup Wi-Fi's name ends in {suffix}, like the label." |
| S-3 | Country | `SETUP`, only when `country.fixed` is null | `claim` | — | "Wi-Fi rules differ by country. Where is this device used?" A list. |
| S-4 | Guest Wi-Fi | `SETUP` | `claim` | — | Name field with the room counter (§5.2); the two modes, the default shown as chosen, one line of consequence each (RFC 012 ENK-OWE-008). |
| S-5 | Owner password | `SETUP` | `claim` | `claim/check` | Rules above the field (§5.3). "You enter it whenever you change anything. Keep it somewhere safe: if it is lost, the only way back is Erase." |
| S-6 | Keep these | `SETUP` | — | `claim/confirm` | The proposal: management Wi-Fi name, management Wi-Fi password, a Wi-Fi QR code, the address and its QR code. "Keep these. You need them in a moment. This phone cannot scan its own screen: write them down or take a screenshot." One button: *Set up*. |
| S-7 | Join the management Wi-Fi | after confirm | — | — | Numbered: 1 Open your phone's Wi-Fi settings. 2 Join {name} with the password above. 3 Come back here, or open {address}. "You have 10 minutes." The countdown. The screen keeps polling the greeting and moves to S-8 by itself when it answers `PENDING`. |
| S-8 | Log in | `PENDING`, `CLAIMED` | greeting | `login` | Owner password. "Forgot it? The only way back is Erase: hold the small button on the back for five seconds until the light changes." The `try_later` sentence with its countdown when it applies. |
| S-9 | Status | `CLAIMED` | `status` | — | §5.1. The first screen. Header: language switch, menu. |
| S-10 | Settings | `CLAIMED` | `status` | `settings`, `settings/restore` | Guest Wi-Fi: on/off, name with counter, mode (switching to *Encrypted only* shows the count that will lose the network, RFC 012 ENK-OWE-011). Management Wi-Fi name. One *Save*. Drift banner with *Restore my settings* when `status.drift`. The country is shown, not editable: "Changed only by Erase." At the bottom: a link to S-13 Erase. |
| S-11 | Change in progress | `CLAIMED`, while `change.state` is not `none`/`settled` | `change` | — | The sentence of §5.5 for the state; for `waiting_confirmation` also the new name, the password, the Wi-Fi QR code, and the countdown. Returns to S-9 or S-10 by itself when settled or undone, showing the result sentence once. |
| S-12 | Management Wi-Fi | `CLAIMED` | `management-key` | `management-key` | Name, password (shown on request), Wi-Fi QR code, address and QR code; *Replace the password* on its own confirmation step: "Every device on the management Wi-Fi, including this phone, must join again with the new password. You will have 10 minutes." |
| S-13 | Erase | `CLAIMED` | — | `erase` | §5.7. Owner password field. One button. Not reachable from S-14. |
| S-14 | Details | `CLAIMED` | `device`, `update` | `update/check`, `update/install`, `password`, `restart`, `logout` | Version line, update block (§5.6), model and identifier, Internet connection, hardening report as a list of met / not met lines, *Change owner password* (own step), *Restart* (own step, §5.7), *Log out*. Erase is not here (RFC 009 ENK-WORD-003). |
| S-15 | What this protects / Guest notice | `CLAIMED` | `status` | — | The text of RFC 012 §4 as amended by RFC 016, with the guest Wi-Fi's name; a *Print* view (§5.8). Reached from S-9. |

S-15 is the fifteenth screen only in numbering; S-11 is a state of S-9/S-10 rather than a place
the operator navigates to.

| ID | Requirement | Verification |
|---|---|---|
| ENK-UI-012 | Setup shall be S-1 to S-7 in this order, one decision per screen, with "step n of m" shown, and shall take a non-expert at most ten minutes excluding the time to join the Wi-Fi networks. | M-7 |
| ENK-UI-013 | Every action that stops the Wi-Fi, cannot be undone, or ends sessions (Erase, Restart, Update, replacing the management Wi-Fi password, changing the owner password, switching to *Encrypted only*) shall have its own confirmation step that states the consequence in one sentence and offers one primary button. Erase and Restart shall never appear on one screen (ENK-WORD-003); Erase shall require the owner password (ENK-OWN-017). | Inspection |
| ENK-UI-014 | The UI shall offer only what the greeting's state and the device profile allow (RFC 005 ENK-TIER-003): no identify step without a light, no country choice when fixed, no update block when `checks_enabled` is false except the switch to enable it. | Test per profile |

### 5. Sentences

Every sentence the operator reads, grouped by screen or by the API value that selects it.
`{…}` are placeholders filled from the API. Sentences fixed by an accepted RFC are quoted; the
Japanese is fixed here.

#### 5.1 Status (S-9)

```text
Guest Wi-Fi  "Cafe Example"        ● running
Open to every device
12 devices connected: 10 encrypted · 2 not encrypted
What this protects ›

Management Wi-Fi  "Enkrnett-3F2A"  ● running   1 device
Internet connection                ● connected

Enkrnett 0.2 on OpenWrt 25.12      Up to date
```

| Element | English | Japanese |
|---|---|---|
| Guest line | Guest Wi-Fi "{name}" | ゲストWi-Fi「{name}」 |
| Running / stopped / off | running / stopped / off | 稼働中 / 停止中 / オフ |
| Mode | Open to every device / Encrypted only | すべての端末が接続できる / 暗号化できる端末のみ |
| Counts, open mode | {n} devices connected: {e} encrypted · {u} not encrypted | 接続中の端末 {n}台：暗号化あり {e}台・暗号化なし {u}台 |
| Counts, encrypted only | {n} devices connected, all encrypted | 接続中の端末 {n}台：すべて暗号化あり |
| No devices | No devices connected | 接続中の端末はありません |
| Link | What this protects | このWi-Fiが守るもの |
| Management line | Management Wi-Fi "{name}" · {n} devices | 設定用Wi-Fi「{name}」・{n}台 |
| Internet | Internet connection: connected / not connected | インターネット接続：接続中 / 未接続 |
| Version (RFC 013 ENK-RLS-003) | Enkrnett {v} on OpenWrt {b} | Enkrnett {v}（OpenWrt {b}） |
| Update summary | Up to date / Update available: Enkrnett {v} / Check for updates | 最新の状態です / アップデートがあります：Enkrnett {v} / アップデートを確認 |
| Drift banner | Settings were changed outside Enkrnett. Restore my settings? | Enkrnett以外の方法で設定が変更されています。Enkrnettの設定に戻しますか？ |
| Hardening not met | Some protections of this device are not in place. See Details. | 本体の保護の一部が有効になっていません。詳細をご確認ください。 |
| Development build (ENK-HARD-007) | Development build. Not for use in a venue. | 開発用ビルドです。店舗や施設では使用しないでください。 |

"Stopped" is `up: false` with `enabled: true`; "off" is `enabled: false`. The device never shows
"stopped" without a change in progress or a drift banner that explains it; if it would, the
hardening or health line says what the device knows.

#### 5.2 Names (S-4, S-10)

| Element | English | Japanese |
|---|---|---|
| Before the field | Guests see this name on their phones. Up to {max} characters. Japanese and other full-width characters count as 3 each, because a Wi-Fi name has a fixed size. | ゲストの端末に表示される名前です。最大{max}文字分。日本語などの全角文字は1文字で3文字分です（Wi-Fiの名前には大きさの上限があるため）。 |
| Counter | {n} left | あと{n} |
| Over the limit (mode switch, ENK-CFG-018) | In "Open to every device" the name can use at most 29. Shorten it by {k} first. | 「すべての端末が接続できる」では名前は29文字分までです。先に{k}文字分短くしてください。 |
| Management Wi-Fi name | Your phone shows this name when you join the management Wi-Fi. Up to 32 characters, counted the same way. | 設定用Wi-Fiに接続するときに表示される名前です。最大32文字分（数え方は同じ）。 |

| ID | Requirement | Verification |
|---|---|---|
| ENK-UI-015 | A name field shall normalise its value to Unicode NFC, count the UTF-8 bytes of the normalised value against `limits` from the API, show the room left while typing, and refuse to send a value over the limit (RFC 011 ENK-CFG-016 to 018). The same component serves setup and Settings. | Test with 9 and 10 Japanese characters, with combining marks, with 29/30/32/33 bytes |

#### 5.3 Passwords (S-5, S-8, S-13, S-14)

| Element | English | Japanese |
|---|---|---|
| Rules (above the field) | 10 to 64 characters: letters, numbers, and symbols you can type on any keyboard. Not a Wi-Fi name, not the management Wi-Fi password, not a common password. | 半角の英字・数字・記号で10〜64文字。Wi-Fiの名前、設定用Wi-Fiのパスワード、よく使われるパスワードは使えません。 |
| Keep it | Keep it somewhere safe. If it is lost, the only way back is Erase, and you set up again. | 安全な場所に控えてください。忘れた場合は初期化して最初からセットアップし直すしかありません。 |
| Show / hide | Show / Hide | 表示 / 隠す |
| Change owner password (S-14) | Enter the current password and the new one. Every phone, including this one, is logged out. | 現在のパスワードと新しいパスワードを入力してください。この端末を含むすべての端末がログアウトされます。 |

Password fields use the browser's `new-password` / `current-password` hints so that a phone's
password manager can offer to save the owner password (M-1 row 12 observes whether it does).

#### 5.4 The connection line (`connection.security`, RFC 016)

| Value | Where | English | Japanese |
|---|---|---|---|
| `owe` | S-1 | Your phone joined the setup Wi-Fi with encryption. What you enter here is protected on the radio link. | お使いの端末は暗号化ありでセットアップ用Wi-Fiに接続しています。ここで入力する内容は電波上で保護されます。 |
| `none` | S-1 | Your phone joined the setup Wi-Fi without encryption. Someone within range could read what you enter here. If other people are nearby, set up with a newer phone or at a quieter time. | お使いの端末は暗号化なしでセットアップ用Wi-Fiに接続しています。近くにいる人に入力内容を読み取られる可能性があります。周りに人がいる場合は、新しい端末を使うか、人のいない時間にセットアップしてください。 |
| `null` | S-1 | Could not tell whether your phone's connection is encrypted. | お使いの端末の接続が暗号化されているか確認できませんでした。 |
| `wpa2` | S-9, one line | Your phone uses the older protection on the management Wi-Fi. Someone who knows the management Wi-Fi password could read this session. Keep that password private. Newer phones do not have this limit. | お使いの端末は設定用Wi-Fiに古い方式で接続しています。設定用Wi-Fiのパスワードを知っている人には、この通信を読まれる可能性があります。パスワードは他の人に教えないでください。新しい端末にはこの制限はありません。 |
| `wpa3` | — | (nothing) | （表示なし） |

#### 5.5 Changes (`change`, RFC 011 §8 as amended)

| `state` / `reason` | English | Japanese |
|---|---|---|
| `validating` | Checking… | 確認しています… |
| `applying` | Saving… The Wi-Fi restarts for a few seconds. | 保存しています… Wi-Fiが数秒間つながらなくなります。 |
| `waiting_confirmation` | Saved. Join the new management Wi-Fi within 10 minutes, or the change is undone. | 保存しました。10分以内に新しい設定用Wi-Fiに接続してください。接続がないと、変更は元に戻ります。 |
| `settled` | Saved. | 保存しました。 |
| `undone` / `health_check_failed` | Not saved: the device could not start the Wi-Fi this way. Your previous settings are back. {detail} | 保存できませんでした。この設定ではWi-Fiを開始できませんでした。以前の設定に戻しました。{detail} |
| `undone` / `not_confirmed` | Undone: the new management Wi-Fi was not joined in time. Your previous settings are back. | 元に戻しました。時間内に新しい設定用Wi-Fiへの接続がなかったため、以前の設定に戻しました。 |
| `undone` / `restart` | A change from before the restart was not finished and has been undone. | 再起動前の変更が完了していなかったため、元に戻しました。 |
| Countdown | {m}:{ss} left | 残り {m}:{ss} |
| Mode switch warning (ENK-OWE-011) | {u} devices connected now are not encrypted. They lose the guest Wi-Fi when you save. | 現在接続中の端末のうち{u}台は暗号化なしです。保存するとこれらの端末はゲストWi-Fiに接続できなくなります。 |
| Mode switch note (ENK-OWE-012) | Encrypted devices may take a few minutes to come back. | 暗号化ありの端末は、再接続に数分かかることがあります。 |

| `detail` | English | Japanese |
|---|---|---|
| `guest_down` | The guest Wi-Fi did not start. | ゲストWi-Fiが開始しませんでした。 |
| `management_down` | The management Wi-Fi did not start. | 設定用Wi-Fiが開始しませんでした。 |
| `isolation_missing` | Devices could not be kept apart from each other. | 端末どうしを分離できませんでした。 |
| `dhcp_missing` | Devices would not have received an address. | 端末にアドレスを配れませんでした。 |
| `listener_exposed` | The settings screen would have been reachable from the guest Wi-Fi. | 設定画面がゲストWi-Fiから見える状態でした。 |
| `pmf_missing` | The encrypted connection could not be set up with the required protection. | 暗号化接続を必要な保護つきで設定できませんでした。 |
| `mode_differs` | The Wi-Fi did not run in the chosen mode. | Wi-Fiが選んだモードで動作しませんでした。 |

The `not_confirmed` sentence replaces RFC 011 §8's "you did not confirm in time", because there
is nothing to confirm since RFC 015 (Consequences).

#### 5.6 Update (S-14, RFC 013 §3)

| Element | English | Japanese |
|---|---|---|
| Check | Check for updates | アップデートを確認 |
| `last_check.result` = `unreachable` | Could not reach the update server. Check the Internet connection. | アップデートサーバーに接続できませんでした。インターネット接続を確認してください。 |
| `bad_signature`, `bad_index` | The update information could not be verified and was not used. | アップデート情報を検証できなかったため、使用しませんでした。 |
| `stale_index` | The update information received is older than what this device already knows and was not used. | 受け取ったアップデート情報は本体が知っているものより古いため、使用しませんでした。 |
| Available | Update available: Enkrnett {v}. Installing takes about {m} minutes. The Wi-Fi stops meanwhile. This cannot be undone from here. | アップデートがあります：Enkrnett {v}。インストールには約{m}分かかり、その間Wi-Fiは止まります。この操作は本体からは取り消せません。 |
| `changes_baseline` (ENK-UPD-007) | This one takes longer and restarts the Wi-Fi twice. | 今回は時間が長くかかり、Wi-Fiが2回再起動します。 |
| Button | Update | アップデート |
| `downloading` / `verifying` | Downloading… / Verifying… | ダウンロード中… / 検証中… |
| `installing` | Installing. The Wi-Fi stops for about two minutes. Come back to this address afterwards. | インストール中です。Wi-Fiが約2分間止まります。終わったらこのアドレスを開き直してください。 |
| `last_result.result` = `done` | Updated to Enkrnett {v}. | Enkrnett {v} にアップデートしました。 |
| `failed` / `health_check_failed` | The update was installed, but the device could not start your Wi-Fi with it and is using your previous settings. {reason} | アップデートはインストールされましたが、新しい版ではWi-Fiを開始できなかったため、以前の設定で動作しています。 |
| `cannot_update` / `bad_signature` | The update file could not be verified. Nothing was changed. | アップデートファイルを検証できませんでした。何も変更していません。 |
| `wrong_device` | This update is not for this device. | このアップデートはこの本体用ではありません。 |
| `wrong_baseline` | This update cannot follow the installed version directly. | このアップデートは現在のバージョンから直接は適用できません。 |
| `not_enough_memory` | Not enough free memory. Restart the device and try again. | 空きメモリが足りません。本体を再起動してからお試しください。 |
| `download_failed` | The update could not be downloaded. Check the Internet connection and try again. | アップデートをダウンロードできませんでした。インターネット接続を確認して、もう一度お試しください。 |
| `checks_enabled` false (ENK-UPD-008) | This device was not installed from a published image, so it does not look for updates. Turn on update checks? | この本体は公開イメージからインストールされていないため、アップデートを確認しません。アップデートの確認をオンにしますか？ |

#### 5.7 Erase and Restart

| Screen | English | Japanese |
|---|---|---|
| S-13 Erase | Erase removes everything you set up: the guest Wi-Fi, the management Wi-Fi, and the owner password. The device returns to how it was after installation and restarts. You then set it up again from the beginning. Enter the owner password to erase. | 初期化すると、ゲストWi-Fi、設定用Wi-Fi、管理者パスワードなど、設定した内容がすべて消えます。本体はインストール直後の状態に戻って再起動します。そのあと、最初からセットアップし直します。初期化するには管理者パスワードを入力してください。 |
| Button | Erase this device | この本体を初期化 |
| After | Erasing. The device restarts and opens its setup Wi-Fi in about two minutes. | 初期化しています。本体は再起動し、約2分後にセットアップ用Wi-Fiを開始します。 |
| S-14 Restart step | Restart turns the device off and on. The Wi-Fi stops for about a minute. Nothing changes. | 再起動すると本体の電源が入れ直されます。Wi-Fiが約1分間止まります。設定は変わりません。 |
| Button | Restart | 再起動 |
| After | Restarting. Come back to this address in about a minute. | 再起動しています。約1分後にこのアドレスを開き直してください。 |
| Guide sentence for the button (RFC 009 D) | Hold the small button on the back for five seconds. The light starts blinking. Let go. | 背面の小さなボタンを5秒間押し続けてください。ランプが点滅し始めたら指を離します。 |

#### 5.8 What this protects, and the guest notice (S-15; RFC 012 §4 as amended by RFC 016)

English: the text of RFC 012 §4 with the last paragraph of RFC 016 §8. Japanese:

> **このWi-Fiは、お使いの端末が対応している場合、端末とアクセスポイントの間の通信を暗号化します**
> （2020年ごろ以降のほとんどのスマートフォンやパソコンが対応しています。規格の名前は
> 「Enhanced Open」または「OWE」です）。古い端末は暗号化なしで接続します。
>
> 暗号化されるのは電波の区間だけです。このWi-Fiを運営している相手を確認するものではなく、
> 同じ名前の偽のWi-Fiから守るものでもなく、このアクセスポイントの先で起こることを守るもの
> でもありません。大切な通信には、通信の全体を暗号化するウェブサイトやアプリ（ブラウザーの
> 鍵マーク）をお使いください。
>
> このWi-Fiに接続した端末どうしの間、およびお店や施設のネットワークとの間は、この
> アクセスポイントが分離しています。これにより、ほかの利用者にできることは減りますが、
> なくなるわけではありません。
>
> このアクセスポイントは、接続した端末の記録を残しません。

| Element | English | Japanese |
|---|---|---|
| Notice title | Wi-Fi "{name}" | Wi-Fi「{name}」 |
| Print view | A4 and Letter, black on white, the name in large type, the text, no other element. One page. | 同じ |

| ID | Requirement | Verification |
|---|---|---|
| ENK-UI-016 | S-15 shall render the same catalogue entry for the screen and for the print view, with the guest Wi-Fi's effective name, in the UI's language, as one page with print styles and no colour (RFC 012 ENK-GUEST-001). | Print to PDF in both languages |

#### 5.9 Channel and connection (RFC 003 §5; RFC 015 §9)

| Situation | English | Japanese |
|---|---|---|
| Browser shows "Not secure" (S-8 footer, S-14) | This page is delivered inside your management Wi-Fi without a web certificate. Your management Wi-Fi encrypts it. Keep the management Wi-Fi password private. | このページはウェブ証明書を使わずに、設定用Wi-Fiの中だけで表示されています。設定用Wi-Fiが通信を暗号化しています。設定用Wi-Fiのパスワードは他の人に教えないでください。 |
| A poll fails | Not connected to the device. Join the management Wi-Fi "{name}" and this screen continues by itself. | 本体に接続できません。設定用Wi-Fi「{name}」に接続すると、この画面は自動的に続きます。 |
| Session ended (`no_session`) | You were logged out. Log in again. | ログアウトされました。もう一度ログインしてください。 |
| `wrong_state` | The device's state has changed. Updating the screen. | 本体の状態が変わりました。画面を更新します。 |
| `try_later` | Too many attempts. Try again in {s} seconds. | 試行回数が多すぎます。{s}秒後にもう一度お試しください。 |
| `wrong_password` | The password is not correct. | パスワードが違います。 |
| `setup_in_progress_elsewhere` | Setup is in progress on another phone. If that is not you, unplug the device, wait ten seconds, and plug it in again. | 別の端末でセットアップが進行中です。心当たりがない場合は、本体の電源を抜き、10秒待ってから差し直してください。 |
| `version_mismatch` | The settings changed since you opened this screen. Showing the current ones. | この画面を開いた後に設定が変わりました。最新の設定を表示します。 |
| `change_pending` | A change is still in progress. Wait for it to finish. | 変更を処理中です。完了までお待ちください。 |
| `update_in_progress` | An update is in progress. Nothing can be changed until it finishes. | アップデート中です。完了するまで変更できません。 |
| `nothing_to_install` | No update to install. | インストールするアップデートはありません。 |
| `not_available` | Not available on this device. | この本体では使えません。 |
| `invalid` field codes | `required` Enter a value. · `too_short` Too short. · `too_long` Too long. · `forbidden_characters` Contains characters that cannot be used. · `common_password` This is a commonly used password. · `equals_network_name` Must differ from the Wi-Fi name. · `equals_key` Must differ from the management Wi-Fi password. · `equals_device_id` Must differ from the device identifier. · `not_in_choices` Not one of the choices. · `not_allowed` Not allowed in this mode. | 入力してください。 · 短すぎます。 · 長すぎます。 · 使えない文字が含まれています。 · よく使われるパスワードです。 · Wi-Fiの名前と同じにはできません。 · 設定用Wi-Fiのパスワードと同じにはできません。 · 本体の識別番号と同じにはできません。 · 選べない値です。 · このモードでは使えません。 |
| `internal`, and any code this RFC does not list | Something went wrong on the device (reference {ref}). Try again. If it repeats, restart the device. | 本体で問題が発生しました（参照番号 {ref}）。もう一度お試しください。繰り返す場合は本体を再起動してください。 |

| ID | Requirement | Verification |
|---|---|---|
| ENK-UI-017 | Every enumeration value and error code of RFC 015 shall map to exactly one entry of §5; an unknown value shall map to the last row of §5.9 and shall be logged to the browser console with the raw value. | Test that walks every value of the RFC 015 shapes against the catalogue |
| ENK-UI-018 | Countdowns shall be computed from the `*_in` value of the latest response and a monotonic clock in the browser, never from the date, and shall be refreshed from each poll. | Test with a wrong phone date |
| ENK-UI-019 | When a poll fails, the UI shall show the "not connected" sentence with the management Wi-Fi's current name, keep polling at the same interval, and resume the screen it was on when a poll succeeds. During S-7 and S-11 this is the expected path and is shown without alarm. | Test by dropping the network |

### 6. Behaviour and form

| ID | Requirement | Verification |
|---|---|---|
| ENK-UI-020 | Every screen shall work at 360 CSS pixels of width without horizontal scrolling, with touch targets of at least 44 × 44 CSS pixels, and with text sized in `rem` so that the phone's text size setting applies. | Browser harness at 360 and 430 px; iOS larger text |
| ENK-UI-021 | Colour shall distinguish only running from not running; every other difference is made by words. Contrast shall meet WCAG 2.2 AA; dark appearance follows the system. | Inspection; contrast check in CI |
| ENK-UI-022 | Every value that comes from the API shall be rendered as text (RFC 007 ENK-SEC-013); catalogue strings shall contain no markup. | Test with hostile names |
| ENK-UI-023 | The session token shall be held in memory and in the tab's session storage only; nothing else the UI needs shall depend on browser storage (RFC 003 ENK-MGMT-003). | Test with storage blocked |
| ENK-UI-024 | The UI shall load nothing from outside the device (RFC 003 ENK-MGMT-002): system fonts only, at most twelve inline SVG icons, no icon font, no analytics. Both languages inside the 150 KiB budget (RFC 010 ENK-FOOT-002), measured in CI. | Network capture; CI size check |
| ENK-UI-025 | The QR codes (the address, RFC 003 ENK-MGMT-009; the management Wi-Fi as `WIFI:T:WPA;S:{name};P:{password};;`) shall be generated in the UI without any network request, beside the sentence that this phone cannot scan its own screen. | Inspection; scan test from a second phone |
| ENK-UI-026 | Every field shall have a visible label; focus order follows reading order; status changes are announced to assistive technology; nothing is conveyed by colour alone. | Accessibility audit in the harness |
| ENK-UI-027 | The development-build banner (RFC 002 ENK-HARD-007) shall appear at the top of every screen, including S-1 and S-8, when the greeting says `development: true`. | Test |

### 7. Navigation

```text
S-9 Status ──► S-15 What this protects / Guest notice
   │  menu ──► S-10 Settings ──► (bottom) S-13 Erase
   │        ──► S-12 Management Wi-Fi
   │        ──► S-14 Details ──► Change owner password · Restart · Log out · Update
   └── S-11 Change in progress (shown over S-9 / S-10 while a change runs)
```

The menu has three entries. Back always returns to Status.

### 8. The word check (ENK-WORD-001)

The UI build fails when a catalogue string contains any of:

| English | Japanese |
|---|---|
| secure, insecure, security (as a label), Transition Mode, transition, OWE*, Enhanced Open*, WPA2, WPA3, PMF, SAE, SSID, BSS, BSSID, reset (as an action), apply, commit, rollback, pairing, pair, uplink, claim, management key (display form is "management Wi-Fi password"), network (in "guest network", "management network", "setup network") | セキュア, セキュリティ, 安全（単独のラベルとして）, トランジション, 移行モード, OWE*, Enhanced Open*, WPA2, WPA3, PMF, SAE, SSID, BSS, BSSID, リセット, 適用, コミット, ロールバック, ペアリング, ペア, 管理用, 初期設定, 管理キー, ネットワーク（「ゲストネットワーク」など） |

`*` allowed once, in the guest notice entry only (RFC 012 §4). The check lists the offending
string and entry.

## Consequences

| Affected | Change |
|---|---|
| RFC 001 §4.1 and §4.3 | Gain display forms: "… Wi-Fi" for the three networks; "management Wi-Fi password" for the management key; "Internet connection" for the uplink. Terms unchanged. Pointer notes at acceptance (ENK-SCOPE-006). |
| RFC 001 open question 1 | Closed by §2. |
| RFC 011 §8 row `not_confirmed` | Sentence replaced (§5.5). |
| RFC 011, 012, 015 open questions on wording | Closed by §5. |
| RFC 012 ENK-OWE-019 (compatibility text) | Carried as a catalogue entry with the ledger's values; updated from M-4a and M-4 (open question 1). |
| Review 13 suggestions S-1 to S-4 | Adopted (ENK-UI-025, 015, §5.4, §2). |
| Gap analysis GAP-070 | Closed. |
| Draft ENK-UI-001 to 007 | Carried or withdrawn in the consolidation; their numbers are not reused. |
| Guides | Reuse the sentences of §5 verbatim where they describe the same step. |
| Browser harness (M1) | One test per screen, both languages, against the mock of RFC 015. |

## Alternatives considered

| Alternative | Why not |
|---|---|
| "Guest network" on screen, as in RFC 001 | The phone's own screen says Wi-Fi; a non-expert matches words across the two screens. |
| 管理用Wi-Fi for the management network | 管理者パスワード and 管理用Wi-Fiのパスワード differ by one character in the position the eye skips. |
| 初期設定 for "set up" | Shares 初期 with 初期化 (Erase), the one action that must never be taken by mistake. |
| 所有者 for owner | Reads as legal ownership; café staff who set the device up are not its legal owners. |
| Show `connection.security` = `wpa3` as reassurance | A reassurance badge is what RFC 012 ENK-OWE-017 forbids; silence is the honest form of "nothing to say". |
| A bottom tab bar | Three menu entries do not need one; a tab bar costs height on the status screen. |
| Web fonts for Japanese | Megabytes; system fonts render Japanese on every phone. |
| Server-side rendering or a lighter framework than Svelte | RFC 013 fixed Svelte 5; its compiled output fits the budget (to be measured in M1's first build). |

## Open questions

| # | Question | Closed by |
|---|---|---|
| 1 | The client-compatibility sentence (RFC 012 ENK-OWE-019) after M-4a and M-4 | M-4a report; M-4 |
| 2 | Whether the phones' "stay connected to this Wi-Fi without Internet?" dialogs need a sentence in S-7 and the guides, and in which words per OS version | M-1 rows 13 to 15 |
| 3 | Whether the address or also a name is shown in S-6 and S-12 (RFC 003 ENK-MGMT-008) | M-1 |
| 4 | Wording changes found with non-experts | M-7 |
| 5 | Native-reader review of the Japanese catalogue before the first release (ENK-UI-011) | Release checklist |
