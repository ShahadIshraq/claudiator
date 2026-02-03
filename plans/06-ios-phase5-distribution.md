# Phase 5 — Distribution

## Pre-Submission Checklist

- [ ] App Icon: 1024x1024 asset for App Store, plus all required sizes in asset catalog
- [ ] Privacy Policy URL: Required for push notifications — host a simple page explaining data collected (server URL, push token, no personal data)
- [ ] App functions correctly with a live server (Apple tests during review)
- [ ] Push notifications work end-to-end (server sends, device receives)
- [ ] iPad layout works correctly (NavigationSplitView)
- [ ] No crashes on first launch, no config, or bad server URL

## TestFlight (First)

1. In Xcode: Product → Archive
2. Upload to App Store Connect via Xcode Organizer
3. In App Store Connect → TestFlight:
   - Add build to a test group
   - Internal testing (up to 100 team members) — no review needed
   - External testing (up to 10,000) — brief review on first build
4. Testers install via TestFlight app on their devices
5. Push notifications work on TestFlight builds (use APNs sandbox endpoint: `api.sandbox.push.apple.com`)

**Important:** TestFlight builds use the **sandbox** APNs environment. The server must be configured with `CLAUDIATOR_APNS_SANDBOX=true` for TestFlight testers. Production App Store builds use the production APNs endpoint.

## App Store Submission

1. In App Store Connect → App Store tab:
   - Fill in app metadata (name, subtitle, description, keywords, category)
   - Upload screenshots: iPhone 6.7" and iPad 12.9" at minimum
   - Set pricing (Free)
   - Add privacy policy URL
   - Select the build from TestFlight
2. Submit for Review
3. Review notes: Explain that the app connects to a self-hosted server and requires a server URL + API key to function. Provide a demo server URL and API key for the reviewer.

## CI/CD (Optional, Later)

Xcode Cloud or GitHub Actions with `xcodebuild` + `altool`/`xcrun notarytool` for automated archive and upload. Not needed for initial release — manual Xcode archive is fine.

## APNs Environment Mapping

| Distribution Method | APNs Endpoint | Server Config |
|---|---|---|
| Xcode debug / Simulator | Sandbox (`api.sandbox.push.apple.com`) | `APNS_SANDBOX=true` |
| TestFlight | Sandbox | `APNS_SANDBOX=true` |
| App Store | Production (`api.push.apple.com`) | `APNS_SANDBOX=false` |

If you have both TestFlight and App Store users simultaneously, the server needs to track which environment each token belongs to and send to the correct endpoint. Simplest approach: add a `sandbox` boolean column to `push_tokens` and let the app send its environment during registration.
