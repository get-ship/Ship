# Installer signing

The release pipeline is **signing-ready** — every signing step in
`.github/workflows/release-cli.yml` reads from GitHub Actions secrets
and turns into a no-op when the secret is unset. To enable signing,
drop the secrets below. **No workflow or code change needed.**

## Recommended stack (cheapest, no cloud HSM)

| Platform | Vendor | Cost | Key storage |
|----------|--------|------|-------------|
| macOS    | Apple Developer Program | $99/yr | n/a (Apple-managed) |
| Windows  | SSL.com OV code signing | ~$180/yr | `.pfx` file in GitHub Secrets |

**Total: ~$280/yr.** No Azure, no AWS, no hardware tokens, no cloud
HSM subscription. The Windows OV cert means users see a SmartScreen
"Don't run" warning for the first few weeks until reputation builds
with Microsoft; macOS is clean from day one.

When cash flow allows, upgrade Windows to **Certum EV** (~$300/yr)
which clears SmartScreen immediately and bundles cloud signing at no
extra cost. No vendor lock-in — same `WINDOWS_SIGNING_*` secrets.

## macOS — Apple Developer Program

Required secrets:

| Secret | Source |
|--------|--------|
| `APPLE_DEVELOPER_ID` | Cert common name, e.g. `Developer ID Application: Ship Inc. (TEAMID12)` |
| `APPLE_ID` | Apple ID email enrolled in the Developer Program |
| `APPLE_APP_SPECIFIC_PASSWORD` | App-specific password from appleid.apple.com |
| `APPLE_TEAM_ID` | 10-char team identifier from developer.apple.com |

Enrollment:

1. developer.apple.com → enroll ($99/yr).
   - Individual: ~24h, cert says your personal name.
   - Organization: needs a D-U-N-S number (free from dnb.com, 1–5 days)
     + Apple phone verification (3–7 business days).
2. Xcode → Settings → Accounts → Manage Certificates → create a
   **Developer ID Application** cert and a **Developer ID Installer**
   cert. Export both from Keychain Access as `.p12`.
3. appleid.apple.com → Sign-In and Security → App-Specific Passwords →
   generate one labelled "ship-ci".
4. Set the four secrets. `notarytool` submits every `.pkg` for
   notarization and staples the ticket to the artifact. Fail-open:
   notarization failure logs but does not block the release.

Turnaround per pkg: ~5 minutes.

## Windows — SSL.com OV (v1 recommendation)

Required secrets:

| Secret | Source |
|--------|--------|
| `WINDOWS_SIGNING_CERT` | Base64-encoded `.pfx` |
| `WINDOWS_SIGNING_PASSWORD` | PFX export password |

Procurement:

1. ssl.com → Code Signing → OV Code Signing Certificate (~$180/yr).
2. Org validation: phone call + business records review. 1–5 days.
3. SSL.com issues the cert through their portal. Download as `.pfx`.
4. `base64 -w0 ship-codesign.pfx > cert.b64` — paste the contents into
   the `WINDOWS_SIGNING_CERT` GitHub secret. Put the export password
   in `WINDOWS_SIGNING_PASSWORD`.
5. The MSI build step runs `signtool sign /f cert.pfx /p $PASSWORD` on
   every artifact.

SmartScreen will warn users until the cert accrues a few thousand
installs. Document the "click More info → Run anyway" path in the
release notes until signing reputation clears (weeks, not months).

## Windows — Certum EV (upgrade path)

When you're ready for a clean install experience from day one:

1. certum.eu → Code Signing → Open Source EV (~$300/yr) — yes, despite
   the name they sell it for proprietary use too; it's their lowest
   EV tier.
2. Identity verification + phone call to a listed org number. 5–10 days.
3. **Certum's Cloud Signing Service is included** — no USB token, no
   Azure Key Vault, no separate HSM. Sign-in credentials + an OTP
   device they ship.
4. Same two secrets (`WINDOWS_SIGNING_CERT`, `WINDOWS_SIGNING_PASSWORD`)
   — Certum's signing tool reads them the same way.
5. EV certs trust SmartScreen immediately.

Alternatives with the same "cert bundles HSM" property if Certum
isn't available: **DigiCert KeyLocker** (~$500/yr), **SSL.com eSigner**
(~$300/yr). All avoid standalone HSM subscriptions.

## Linux packages

`.deb` and `.rpm` install without signing — GitHub Releases is the
canonical source and users verify via the download URL + checksums.
APT/YUM repository signing is a separate follow-up (publishing to a
hosted repo) unrelated to installer signing.

## Rollout checklist

- [ ] Apple Developer Program enrollment complete
- [ ] Developer ID Application + Installer certs exported + base64-encoded
- [ ] App-specific password generated
- [ ] Four `APPLE_*` secrets set on the GitHub repo
- [ ] Windows code signing cert procured (SSL.com OV to start)
- [ ] Two `WINDOWS_SIGNING_*` secrets set on the GitHub repo
- [ ] Tag push triggers pipeline; artifacts verified with `codesign -dvv`
      and `signtool verify /pa`
- [ ] "Unsigned build" warning removed from the release notes template
- [ ] (later) Upgrade Windows to Certum EV; swap the `.pfx` in the secret
