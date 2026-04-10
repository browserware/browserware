//! Selector types for identifying and filtering browser contexts.
//!
//! This module provides [`ContextSelector`] for matching browser contexts
//! by family, browser ID, and profile, as well as [`AmbiguityPolicy`] for
//! controlling behaviour when a selector matches multiple contexts.

use serde::{Deserialize, Serialize};

use crate::{BrowserContext, BrowserFamily, Error, Result};

/// Policy for handling the case where a selector matches multiple contexts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AmbiguityPolicy {
    /// Use the first matching context; no warning is emitted.
    #[default]
    First,
    /// Use the first matching context and emit a [`tracing::warn!`] log.
    Warn,
    /// Return [`Err`] when more than one context matches.
    Error,
}

/// A selector for identifying one or more browser contexts.
///
/// Each field is optional; unspecified fields match any value.
/// All three fields may be combined to identify a single unique context.
///
/// # Examples
///
/// ```
/// use browserware_types::selector::ContextSelector;
///
/// let sel = ContextSelector::parse("chrome:work").unwrap();
/// assert_eq!(sel.browser, Some("chrome".to_string()));
/// assert_eq!(sel.profile, Some("work".to_string()));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ContextSelector {
    /// Filter by browser engine family (e.g. `"chromium"`, `"firefox"`, `"webkit"`).
    pub family: Option<String>,
    /// Filter by browser ID (e.g. `"chrome"`, `"firefox"`, `"safari"`).
    pub browser: Option<String>,
    /// Filter by profile ID (e.g. `"Profile 1"`, `"work"`).
    pub profile: Option<String>,
}

impl ContextSelector {
    /// Parse a selector from the canonical `"key=value,key=value"` format.
    ///
    /// Valid keys are `family`, `browser`, and `profile`. Empty segments
    /// (after trimming) are silently skipped. Returns [`Error::Other`] if
    /// any segment contains no `=` character or uses an unknown key.
    ///
    /// If the same key appears more than once, the last value wins.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Other`] on malformed or unknown key segments.
    #[must_use = "parsing a selector has no effect if the result is not used"]
    pub fn parse_canonical(s: &str) -> Result<Self> {
        let mut sel = Self::default();
        for raw_segment in s.split(',') {
            let segment = raw_segment.trim();
            if segment.is_empty() {
                continue;
            }
            let Some((key, value)) = segment.split_once('=') else {
                return Err(Error::Other(format!(
                    "invalid selector segment: {segment:?} (expected key=value)"
                )));
            };
            let key = key.trim();
            let value = value.trim();
            if key.is_empty() || value.is_empty() {
                return Err(Error::Other(format!(
                    "empty selector key/value in segment: {segment:?}"
                )));
            }
            let value = decode_selector_value(value)?;
            match key {
                "family" => sel.family = Some(value),
                "browser" => sel.browser = Some(value),
                "profile" => sel.profile = Some(value),
                unknown => {
                    return Err(Error::Other(format!(
                        "unknown selector key: {unknown:?} (valid keys: family, browser, profile)"
                    )));
                }
            }
        }
        Ok(sel)
    }

    /// Parse a selector from alias format: `"browser_id"` or `"browser_id:profile_id"`.
    ///
    /// Returns [`Error::Other`] if `s` is empty.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Other`] if the input is empty.
    #[must_use = "parsing a selector has no effect if the result is not used"]
    pub fn parse_alias(s: &str) -> Result<Self> {
        if s.trim().is_empty() {
            return Err(Error::Other("selector alias must not be empty".to_string()));
        }
        match s.split_once(':') {
            Some((browser, profile)) if browser.trim().is_empty() || profile.trim().is_empty() => {
                Err(Error::Other(format!(
                    "empty selector key/value in alias: {s:?}"
                )))
            }
            Some((browser, profile)) => Ok(Self {
                family: None,
                browser: Some(browser.to_string()),
                profile: Some(profile.to_string()),
            }),
            None => Ok(Self {
                family: None,
                browser: Some(s.to_string()),
                profile: None,
            }),
        }
    }

    /// Parse a selector, auto-dispatching to [`Self::parse_canonical`] when
    /// the input contains `=`, or to [`Self::parse_alias`] otherwise.
    ///
    /// # Errors
    ///
    /// Propagates errors from the dispatched parser.
    #[must_use = "parsing a selector has no effect if the result is not used"]
    pub fn parse(s: &str) -> Result<Self> {
        if s.contains('=') {
            Self::parse_canonical(s)
        } else {
            Self::parse_alias(s)
        }
    }

    /// Returns `true` if this selector matches the given [`BrowserContext`].
    ///
    /// Each `Some` field must equal the corresponding context field; `None`
    /// fields always match. If `self.profile` is `Some` but the context has
    /// no profile, the method returns `false`.
    #[must_use]
    pub fn matches(&self, ctx: &BrowserContext) -> bool {
        if let Some(family) = &self.family
            && family.as_str() != family_as_str(ctx.browser.family())
        {
            return false;
        }
        if let Some(browser) = &self.browser
            && *browser != ctx.browser.id.0
        {
            return false;
        }
        if let Some(profile) = &self.profile {
            match ctx.profile.as_ref() {
                Some(p) => {
                    if *profile != p.id {
                        return false;
                    }
                }
                None => return false,
            }
        }
        true
    }

    /// Find the best matching context from a slice, applying `policy` when
    /// more than one context matches.
    ///
    /// - 0 matches → `Ok(None)`
    /// - 1 match   → `Ok(Some(matched))`
    /// - 2+ matches with [`AmbiguityPolicy::First`] → `Ok(Some(first))`
    /// - 2+ matches with [`AmbiguityPolicy::Warn`]  → emits [`tracing::warn!`], then `Ok(Some(first))`
    /// - 2+ matches with [`AmbiguityPolicy::Error`] → `Err`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Other`] when `policy` is [`AmbiguityPolicy::Error`]
    /// and more than one context matches.
    #[must_use = "select result must be checked"]
    pub fn select<'a>(
        &self,
        contexts: &'a [BrowserContext],
        policy: AmbiguityPolicy,
    ) -> Result<Option<&'a BrowserContext>> {
        let mut iter = contexts.iter().filter(|ctx| self.matches(ctx));
        let first = iter.next();
        let second = iter.next();

        match first {
            None => Ok(None),
            Some(first) if second.is_none() => Ok(Some(first)),
            Some(first) => match policy {
                AmbiguityPolicy::First => Ok(Some(first)),
                AmbiguityPolicy::Warn => {
                    // count = first + second + remaining
                    let count = 2 + iter.count();
                    tracing::warn!(
                        count,
                        "ambiguous selector matches multiple contexts, using first"
                    );
                    Ok(Some(first))
                }
                AmbiguityPolicy::Error => {
                    // Rebuild the full match list for the error message (only in error branch)
                    let all: Vec<&BrowserContext> =
                        contexts.iter().filter(|ctx| self.matches(ctx)).collect();
                    let count = all.len();
                    let selectors: Vec<&str> = all.iter().map(|c| c.selector()).collect();
                    Err(Error::Other(format!(
                        "ambiguous selector matches {count} contexts: {}",
                        selectors.join(", ")
                    )))
                }
            },
        }
    }
}

fn decode_selector_value(value: &str) -> Result<String> {
    let mut out: Vec<u8> = Vec::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] != b'%' {
            out.push(bytes[index]);
            index += 1;
            continue;
        }

        if index + 2 >= bytes.len() {
            return Err(Error::Other(format!(
                "invalid percent-encoding in selector value: {value:?}"
            )));
        }

        let hex = &value[index + 1..index + 3];
        let decoded = u8::from_str_radix(hex, 16).map_err(|_| {
            Error::Other(format!(
                "invalid percent-encoding in selector value: {value:?}"
            ))
        })?;
        out.push(decoded);
        index += 3;
    }

    String::from_utf8(out).map_err(|_| {
        Error::Other(format!(
            "invalid UTF-8 in decoded selector value: {value:?}"
        ))
    })
}

/// Map a [`BrowserFamily`] to its canonical lowercase string without allocating.
const fn family_as_str(family: BrowserFamily) -> &'static str {
    match family {
        BrowserFamily::Chromium => "chromium",
        BrowserFamily::Firefox => "firefox",
        BrowserFamily::WebKit => "webkit",
        BrowserFamily::Other => "other",
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::{
        Browser, BrowserContext, BrowserVariant, ChromiumChannel, LaunchCapability, ProfileRef,
        WebKitChannel,
    };

    // ─── test helpers ─────────────────────────────────────────────────────────

    fn chrome_ctx(profile_id: &str, profile_name: &str) -> BrowserContext {
        BrowserContext::new(
            Browser::new("chrome", "Google Chrome", PathBuf::from("/usr/bin/chrome"))
                .with_variant(BrowserVariant::Chromium(ChromiumChannel::Stable)),
            Some(ProfileRef {
                id: profile_id.to_string(),
                display_name: profile_name.to_string(),
            }),
            LaunchCapability::full(),
        )
    }

    fn safari_ctx() -> BrowserContext {
        BrowserContext::new(
            Browser::new(
                "safari",
                "Safari",
                PathBuf::from("/Applications/Safari.app"),
            )
            .with_variant(BrowserVariant::WebKit(WebKitChannel::Stable)),
            None,
            LaunchCapability::launch_only("no profile support"),
        )
    }

    // ─── parse_canonical ──────────────────────────────────────────────────────

    #[test]
    fn parse_canonical_all_fields() {
        let sel = ContextSelector::parse_canonical("family=chromium,browser=chrome,profile=work")
            .unwrap();
        assert_eq!(sel.family, Some("chromium".to_string()));
        assert_eq!(sel.browser, Some("chrome".to_string()));
        assert_eq!(sel.profile, Some("work".to_string()));
    }

    #[test]
    fn parse_canonical_partial() {
        let sel = ContextSelector::parse_canonical("browser=chrome").unwrap();
        assert_eq!(sel.family, None);
        assert_eq!(sel.browser, Some("chrome".to_string()));
        assert_eq!(sel.profile, None);
    }

    #[test]
    fn parse_canonical_duplicate_key_last_wins() {
        let sel = ContextSelector::parse_canonical("browser=chrome,browser=firefox").unwrap();
        assert_eq!(sel.browser.as_deref(), Some("firefox"));
    }

    #[test]
    fn parse_canonical_rejects_no_equals() {
        let err = ContextSelector::parse_canonical("family=chromium,browser").unwrap_err();
        assert!(
            err.to_string().contains("invalid selector segment"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn parse_canonical_rejects_unknown_key() {
        let err = ContextSelector::parse_canonical("family=chromium,channel=stable").unwrap_err();
        assert!(
            err.to_string().contains("unknown selector key"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn parse_canonical_rejects_empty_value() {
        let err = ContextSelector::parse_canonical("browser=").unwrap_err();
        assert!(err.to_string().contains("empty selector key/value"));
    }

    #[test]
    fn parse_canonical_decodes_percent_escaped_values() {
        let sel = ContextSelector::parse_canonical("profile=work%2Calpha%3D1%25").unwrap();
        assert_eq!(sel.profile, Some("work,alpha=1%".to_string()));
    }

    // ─── parse_alias ──────────────────────────────────────────────────────────

    #[test]
    fn parse_alias_browser_and_profile() {
        let sel = ContextSelector::parse_alias("chrome:work").unwrap();
        assert_eq!(sel.browser, Some("chrome".to_string()));
        assert_eq!(sel.profile, Some("work".to_string()));
        assert_eq!(sel.family, None);
    }

    #[test]
    fn parse_alias_browser_only() {
        let sel = ContextSelector::parse_alias("chrome").unwrap();
        assert_eq!(sel.browser, Some("chrome".to_string()));
        assert_eq!(sel.profile, None);
        assert_eq!(sel.family, None);
    }

    #[test]
    fn parse_alias_rejects_empty() {
        assert!(ContextSelector::parse_alias("").is_err());
    }

    #[test]
    fn parse_alias_rejects_empty_profile() {
        let err = ContextSelector::parse_alias("chrome:").unwrap_err();
        assert!(err.to_string().contains("empty selector key/value"));
    }

    // ─── parse (auto-dispatch) ────────────────────────────────────────────────

    #[test]
    fn parse_dispatches_canonical() {
        let sel = ContextSelector::parse("family=chromium").unwrap();
        assert_eq!(sel.family, Some("chromium".to_string()));
        assert_eq!(sel.browser, None);
        assert_eq!(sel.profile, None);
    }

    #[test]
    fn parse_dispatches_alias() {
        let sel = ContextSelector::parse("chrome:work").unwrap();
        assert_eq!(sel.browser, Some("chrome".to_string()));
        assert_eq!(sel.profile, Some("work".to_string()));
    }

    // ─── matches ──────────────────────────────────────────────────────────────

    #[test]
    fn matches_exact_browser_and_profile() {
        let ctx = chrome_ctx("Profile 1", "Work");
        let sel = ContextSelector::parse("browser=chrome,profile=Profile 1").unwrap();
        assert!(sel.matches(&ctx));
    }

    #[test]
    fn parse_canonical_decoding_matches_selector_encoding() {
        let profile_id = "work,alpha=1%";
        let encoded = crate::context::encode_selector_value(profile_id);
        let sel = ContextSelector::parse_canonical(&format!("profile={encoded}")).unwrap();
        assert_eq!(sel.profile.as_deref(), Some(profile_id));
    }

    #[test]
    fn matches_family_only() {
        let ctx = chrome_ctx("Profile 1", "Work");
        let sel = ContextSelector::parse("family=chromium").unwrap();
        assert!(sel.matches(&ctx));
    }

    #[test]
    fn no_match_wrong_profile() {
        let ctx = chrome_ctx("Profile 1", "Work");
        let sel = ContextSelector::parse("browser=chrome,profile=Profile 2").unwrap();
        assert!(!sel.matches(&ctx));
    }

    #[test]
    fn no_match_profile_requested_but_context_has_none() {
        let ctx = safari_ctx();
        let sel = ContextSelector::parse("browser=safari,profile=work").unwrap();
        assert!(!sel.matches(&ctx));
    }

    // ─── select ───────────────────────────────────────────────────────────────

    #[test]
    fn select_exact_match() {
        let contexts = vec![
            chrome_ctx("Profile 1", "Work"),
            chrome_ctx("Profile 2", "Personal"),
            safari_ctx(),
        ];
        let sel = ContextSelector::parse("browser=chrome,profile=Profile 2").unwrap();
        let result = sel.select(&contexts, AmbiguityPolicy::First).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().profile.as_ref().unwrap().id, "Profile 2");
    }

    #[test]
    fn select_no_match_returns_none() {
        let contexts = vec![chrome_ctx("Profile 1", "Work"), safari_ctx()];
        let sel = ContextSelector::parse("browser=firefox").unwrap();
        let result = sel.select(&contexts, AmbiguityPolicy::First).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn select_ambiguous_policy_first() {
        let contexts = vec![
            chrome_ctx("Profile 1", "Work"),
            chrome_ctx("Profile 2", "Personal"),
        ];
        let sel = ContextSelector::parse("browser=chrome").unwrap();
        let result = sel.select(&contexts, AmbiguityPolicy::First).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().profile.as_ref().unwrap().id, "Profile 1");
    }

    #[test]
    fn select_ambiguous_policy_warn() {
        let contexts = vec![
            chrome_ctx("Profile 1", "Work"),
            chrome_ctx("Profile 2", "Personal"),
        ];
        let sel = ContextSelector::parse("browser=chrome").unwrap();
        let result = sel.select(&contexts, AmbiguityPolicy::Warn).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().profile.as_ref().unwrap().id, "Profile 1");
    }

    #[test]
    fn select_ambiguous_policy_error() {
        let contexts = vec![
            chrome_ctx("Profile 1", "Work"),
            chrome_ctx("Profile 2", "Personal"),
        ];
        let sel = ContextSelector::parse("browser=chrome").unwrap();
        let err = sel.select(&contexts, AmbiguityPolicy::Error).unwrap_err();
        assert!(
            err.to_string()
                .contains("ambiguous selector matches 2 contexts"),
            "unexpected error: {err}"
        );
    }
}
