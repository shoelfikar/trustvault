//! The Have I Been Pwned range client — R-25, and the only socket in this application.
//!
//! `trustvault-core` cannot reach here and this file cannot reach the vault. What crosses between
//! them is [`trustvault_core::BreachQuery`]: a 5-character prefix that is permitted to leave the
//! machine, a suffix with no accessor, and the ids the value belongs to. This module never sees a
//! password and has no way to ask for one.
//!
//! # What leaves, and what R-25's packet capture will find
//!
//! `GET /range/{prefix}` with `Add-Padding: true`, over TLS, to `api.pwnedpasswords.com` and
//! nowhere else. No full hash, no password, no item title, no vault or item identifier, no query
//! string, no cookie, no body. The whole request is five hex characters and a header — k-anonymity
//! is the design, not the disclaimer.
//!
//! # The configuration is the security decision, not the crate
//!
//! D-83 chose ureq after a survey; the part of that decision with teeth is [`RangeClient::agent`],
//! because every default it overrides is a default that would send the prefix somewhere R-25 does
//! not look:
//!
//! * **No proxy.** `ureq::Config::default` calls `Proxy::try_from_env`, so `HTTPS_PROXY` and
//!   `ALL_PROXY` are honoured unless refused. A password manager that silently routes through
//!   whatever the environment says is a password manager whose egress claim is a claim about the
//!   user's shell.
//! * **HTTPS only**, where the default is false. The prefix is small, but the *response* tells an
//!   observer which range was asked for, and a plaintext hop hands that over for free.
//! * **No redirects**, where the default follows ten. A 302 from this endpoint is either an
//!   incident or an interception; following one sends the prefix to a host the capture never
//!   recorded. This is the setting that most deserves its test below.
//! * **A bounded body.** ureq's `read_to_string` stops at 10 MB by default; a padded range is
//!   ~100 kB, and the bound here is 1 MB so a hostile response cannot make a thousand-request
//!   scan a memory problem.
//!
//! # What this module does not do
//!
//! It does not decide whether it may run. `Settings.breach_check_enabled` is read in the command
//! layer (§6.9: *the setting owns the egress, not the caller*), and the reason the check lives
//! there rather than here is that a client which refuses politely is a client somebody can call
//! anyway. S-10 is a command nobody invoked, not a branch nobody took.

use std::io::Read;
use std::time::Duration;

use trustvault_core::BreachQuery;

/// Where the range API lives. R-25 names it, and the packet capture is against this host.
pub const ENDPOINT: &str = "https://api.pwnedpasswords.com";

/// How long any single range request may take, end to end.
///
/// Generous rather than tight: §6.9 measured 1.87 req/s serially against this service, so the
/// budget here is about a request that has *stopped* rather than one that is slow. A check of a
/// thousand values is minutes long by arithmetic (S-07b), and a short timeout would convert a
/// congested link into a report full of `offline`.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// The largest range response this client will read — see the module docs.
const BODY_LIMIT: u64 = 1024 * 1024;

/// How many times a request is retried before its value is reported unchecked.
///
/// Three, and the backoff between them is what makes a thousand-item vault a polite caller: the
/// service is free, unauthenticated, and run by somebody else.
const MAX_ATTEMPTS: u32 = 3;

/// The base of the exponential backoff — 1 s, then 2 s, then 4 s.
const BACKOFF_BASE: Duration = Duration::from_secs(1);

/// Why a value could not be checked — §6.9's `unchecked.reason`, minus the `off` case.
///
/// `off` is not here on purpose: it is produced by the command that declined to call this module
/// at all, and a client that can report "the setting was off" is a client that ran.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RangeError {
    /// The request never reached the service — DNS, connect, TLS, timeout, or a refused redirect.
    ///
    /// **Deliberately coarse.** These are distinguishable to the code and are not distinguished,
    /// for R-03's reason one layer out: the user's action is the same for all of them, and the
    /// screen that reports them is not a network diagnostic tool.
    Offline,
    /// The service answered, and the answer was not 200.
    Http(u16),
}

impl RangeError {
    /// The word §6.9 puts in `unchecked.reason`.
    #[must_use]
    pub fn reason(&self) -> &'static str {
        match self {
            Self::Offline => "offline",
            Self::Http(_) => "http",
        }
    }
}

/// One range response, parsed into suffixes and counts.
///
/// Owns no password and nothing derived from one: every row here came off the wire and is public
/// data about a breach corpus. Which row *matched* is the secret, and that never lives in this
/// type — [`Range::count_for`] asks the query and returns a number.
///
/// `Debug` is derived, and that is safe here for the same reason it is refused on
/// [`BreachQuery`]: this is a breach corpus's public answer, not our question about it.
#[derive(Debug, PartialEq, Eq)]
pub struct Range {
    rows: Vec<(String, u64)>,
    /// How many rows carried a count of zero — HIBP's padding, and R-25's third gate line.
    decoys: usize,
}

impl Range {
    /// Parses a range body.
    ///
    /// **The line ending is CRLF and the parser must not assume it.** The live service sends
    /// `\r\n` today; `lines()` handles both, and a stray `\r` left on a suffix would make every
    /// comparison fail and every password read as unbreached — R-25's forbidden direction.
    ///
    /// **An unparseable row is skipped, not fatal.** A row this client cannot read is a row it
    /// cannot match, and the value ends up reported as not-found rather than as an error. That is
    /// the wrong-but-safe direction only because the *padding* rows are the ones most likely to
    /// change shape; a body that is entirely unparseable produces an empty range, which
    /// [`Range::is_empty`] lets the caller refuse.
    #[must_use]
    pub fn parse(body: &str) -> Self {
        let mut rows = Vec::new();
        let mut decoys = 0;
        for line in body.lines() {
            let line = line.trim();
            let Some((suffix, count)) = line.split_once(':') else {
                continue;
            };
            let Ok(count) = count.trim().parse::<u64>() else {
                continue;
            };
            if count == 0 {
                decoys += 1;
            }
            rows.push((suffix.to_owned(), count));
        }
        Self { rows, decoys }
    }

    /// How many times this query's value appears in the breach corpus, or `None` if absent.
    ///
    /// The matching is local — the whole point of k-anonymity — and it is the *query* that
    /// decides, because the suffix it holds has no accessor this module could read.
    ///
    /// **A zero count is not a hit.** Padding rows carry a count of zero and a random suffix, and
    /// a client that treated one as a match would report a password breached on the strength of a
    /// decoy. The collision is astronomically unlikely and the check costs one comparison.
    #[must_use]
    pub fn count_for(&self, query: &BreachQuery) -> Option<u64> {
        self.rows
            .iter()
            .find(|(suffix, count)| *count > 0 && query.matches(suffix))
            .map(|&(_, count)| count)
    }

    /// Rows in this response, padding included.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Whether nothing at all parsed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Rows carrying a count of zero — the padding, and **R-25's third gate line as a number**.
    ///
    /// The gate asks that a padded response carry at least one zero-count row. That is a claim
    /// about the service rather than about this code, which is exactly why it is readable here:
    /// the evidence run can assert it instead of a person counting rows in a capture. It is also
    /// why the count is not asserted as a *band* — it moved 110 → 125 → 156 → 163 for one prefix
    /// across two days of measuring (D-77).
    #[must_use]
    pub fn decoys(&self) -> usize {
        self.decoys
    }
}

/// A pooled client for the range API — R-25.
///
/// Holds one `ureq::Agent`, which holds the connection pool. **Reuse is the whole performance
/// story**: §6.9 measured 48 cold prefixes at 1.87 req/s over a reused connection, and a client
/// constructed per request would pay a TLS handshake a thousand times over.
pub struct RangeClient {
    agent: ureq::Agent,
    endpoint: String,
}

impl Default for RangeClient {
    fn default() -> Self {
        Self::new(ENDPOINT)
    }
}

impl RangeClient {
    /// A client against `endpoint`, configured as the module docs describe.
    ///
    /// The endpoint is a parameter so the offline tests can point it at a listener on localhost.
    /// It is not a setting and is not reachable from the webview: every caller in the application
    /// uses [`RangeClient::default`], and the only other argument ever passed is a `127.0.0.1`
    /// address in this file's own tests.
    #[must_use]
    pub fn new(endpoint: &str) -> Self {
        let config = ureq::Agent::config_builder()
            // Every one of these overrides a default that would widen what leaves the machine.
            // See the module docs; the tests below are what keeps them from being decoration.
            .proxy(None)
            .https_only(!endpoint.starts_with("http://127.0.0.1"))
            .max_redirects(0)
            .max_redirects_will_error(true)
            .http_status_as_error(false)
            .timeout_global(Some(REQUEST_TIMEOUT))
            .timeout_connect(Some(Duration::from_secs(10)))
            // A descriptive agent is what HIBP asks of API consumers. Measured 2026-08-16: the
            // range endpoint answers 200 with no `User-Agent` at all, so this is courtesy rather
            // than a requirement — and it carries no version of the vault, no platform, and
            // nothing that distinguishes one TrustVault user from another.
            .user_agent("TrustVault")
            .build();

        Self {
            agent: config.into(),
            endpoint: endpoint.trim_end_matches('/').to_owned(),
        }
    }

    /// Fetches one range, retrying a rate limit or a server fault with backoff.
    ///
    /// `sleep` is injected so the retry policy can be tested without a test that takes seven
    /// seconds to prove it waited. Production passes [`std::thread::sleep`].
    ///
    /// **What is retried and what is not.** 429 and 5xx are the service asking for patience, so
    /// they back off and try again; a 404 (no such prefix — not something this API returns) and a
    /// 4xx generally are answers, and repeating them is rude rather than resilient. A transport
    /// failure retries once for the case it usually is: one connection in the pool going stale
    /// mid-scan, which on a check that runs for minutes is ordinary rather than exceptional.
    fn range_with(
        &self,
        prefix: &str,
        sleep: &mut dyn FnMut(Duration),
    ) -> Result<Range, RangeError> {
        let url = format!("{}/range/{prefix}", self.endpoint);
        let mut last = RangeError::Offline;

        for attempt in 0..MAX_ATTEMPTS {
            if attempt > 0 {
                sleep(BACKOFF_BASE * 2_u32.pow(attempt - 1));
            }

            // `Add-Padding` is R-25 and it is set per request rather than as an agent default, so
            // that reading this function answers "what did we send" completely. `Accept-Encoding`
            // is not set here: the `gzip` feature adds it, and setting it twice is how a header
            // ends up duplicated on the wire.
            let sent = self.agent.get(&url).header("Add-Padding", "true").call();

            let mut response = match sent {
                Ok(response) => response,
                Err(_) => {
                    // Coarse on purpose — see `RangeError::Offline`. A refused redirect and a
                    // dead link arrive here as the same thing, and they mean the same thing to
                    // the person reading the screen: this value was not checked.
                    last = RangeError::Offline;
                    continue;
                }
            };

            let status = response.status().as_u16();
            if status == 429 || (500..600).contains(&status) {
                last = RangeError::Http(status);
                continue;
            }
            if status != 200 {
                return Err(RangeError::Http(status));
            }

            let mut body = String::new();
            let read = response
                .body_mut()
                .with_config()
                .limit(BODY_LIMIT)
                .reader()
                .read_to_string(&mut body);
            if read.is_err() {
                last = RangeError::Offline;
                continue;
            }

            let range = Range::parse(&body);
            if range.is_empty() {
                // A 200 that parses to nothing is not a range with no hits — a range always has
                // rows, padded or not. Treating it as "no match" would report every password in
                // this group unbreached on the strength of a body we could not read.
                last = RangeError::Offline;
                continue;
            }
            return Ok(range);
        }

        Err(last)
    }

    /// Fetches one range — R-25.
    pub fn range(&self, prefix: &str) -> Result<Range, RangeError> {
        self.range_with(prefix, &mut std::thread::sleep)
    }
}

/// A range service that never was, for tests that must watch where bytes would have gone.
///
/// `pub(crate)` and `#[cfg(test)]`: `commands/watchtower.rs` needs exactly this to prove that a
/// breach check with the setting off contacts nothing, and a second copy of a listener would be a
/// second definition of what "was never contacted" means.
#[cfg(test)]
pub(crate) mod testing {
    use std::io::{Read as _, Write as _};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    /// An HTTP server that answers the same thing to every connection, and reports what it was
    /// asked.
    ///
    /// Returns the address to point a client at, and a channel carrying the raw request line and
    /// headers. The request text is what the packet-capture gate line asserts by hand; having it
    /// here means the claim is also checked on every `cargo test`.
    ///
    /// **It serves in a loop rather than once**, because the client retries: a one-shot server
    /// makes every retry a connection failure, which turns a 429 into `Offline` and would have
    /// let the backoff test pass for the wrong reason.
    pub(crate) fn serving(response: &str) -> (String, mpsc::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a free port");
        let addr = listener.local_addr().expect("bound");
        let (tx, rx) = mpsc::channel();
        let response = response.to_owned();
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else {
                    return;
                };
                let mut buffer = [0_u8; 2048];
                let read = stream.read(&mut buffer).unwrap_or(0);
                if tx
                    .send(String::from_utf8_lossy(&buffer[..read]).into_owned())
                    .is_err()
                {
                    return;
                }
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });
        (format!("http://127.0.0.1:{}", addr.port()), rx)
    }

    /// A 200 carrying `body`.
    pub(crate) fn ok_response(body: &str) -> String {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{body}",
            body.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::testing::{ok_response, serving};
    use super::*;
    use trustvault_core::{Field, FieldKind, ItemKind, KdfParams, Vault, breach_queries};

    /// A real padded response for `5BAA6`, trimmed — `tests/fixtures/hibp-range-5BAA6.txt`.
    ///
    /// Trimmed rather than synthesized: the rows, the counts, the CRLF line endings and the
    /// zero-count padding all came off the live service on 2026-08-16, so a parser that only
    /// works against something this project invented cannot pass here. It is the mocked half of
    /// the gate's first line, and the offline half of the whole suite.
    const FIXTURE: &str = include_str!("../tests/fixtures/hibp-range-5BAA6.txt");

    /// A vault holding exactly one password, so `breach_queries` yields exactly one query.
    fn query_for(password: &str) -> BreachQuery {
        let (mut vault, _) =
            Vault::create("Test", "master pw", KdfParams::TESTING).expect("valid params");
        let id = vault.add_item(ItemKind::Login, "Forum");
        let item = vault.item_mut(id).expect("just added");
        item.push_field(Field::new("Password", password, true).with_kind(FieldKind::Password));
        breach_queries(&vault)
            .pop()
            .expect("one password, one query")
    }

    /// R-25's own worked example, offline: a known-pwned password is reported as pwned.
    ///
    /// This is the **mocked** half of the gate's first line. The live half is a separate run
    /// against the real service, and it stays separate because a test suite that needs the
    /// network is a test suite that fails on an aeroplane.
    #[test]
    fn a_known_pwned_password_is_found_in_the_range() {
        let query = query_for("password");
        assert_eq!(query.prefix(), "5BAA6", "the fixture is this prefix");

        let range = Range::parse(FIXTURE);
        assert_eq!(
            range.count_for(&query),
            Some(52_372_427),
            "the count the live service returned for this suffix on 2026-08-16"
        );
    }

    /// A password absent from the range is absent, not an error and not a hit.
    #[test]
    fn a_password_not_in_the_range_reports_no_count() {
        let query = query_for("qX7#vn2Lp!4dRt");
        let range = Range::parse(FIXTURE);
        assert_eq!(range.count_for(&query), None);
    }

    /// The padding is honoured and countable — R-25's third gate line, offline.
    #[test]
    fn the_padded_fixture_carries_zero_count_rows() {
        let range = Range::parse(FIXTURE);
        assert!(range.decoys() > 0, "Add-Padding produces zero-count rows");
        assert!(range.len() > range.decoys(), "and real rows beside them");
    }

    /// A decoy row is never reported as a breach.
    ///
    /// Constructed rather than waited for: a real padding collision will not happen, and the
    /// branch that would mishandle it is one line. The fixture's own suffix is reused with a
    /// count of zero, which is the exact shape of the bug.
    #[test]
    fn a_zero_count_row_is_padding_and_never_a_hit() {
        let query = query_for("password");
        let range = Range::parse("1E4C9B93F3F0682250B6CF8331B7EE68FD8:0\r\n");
        assert_eq!(
            range.count_for(&query),
            None,
            "a count of zero is a decoy, not a breach"
        );
    }

    /// CRLF is what the service sends, and a stray `\r` would make every password read as safe.
    #[test]
    fn crlf_line_endings_do_not_survive_into_a_suffix() {
        let query = query_for("password");
        let with_crlf = Range::parse("1E4C9B93F3F0682250B6CF8331B7EE68FD8:52372427\r\n");
        let with_lf = Range::parse("1E4C9B93F3F0682250B6CF8331B7EE68FD8:52372427\n");
        assert_eq!(with_crlf.count_for(&query), Some(52_372_427));
        assert_eq!(
            with_lf.count_for(&query),
            with_crlf.count_for(&query),
            "both endings, one answer"
        );
    }

    /// A body that parses to nothing is a failure, never a clean bill of health — R-25.
    #[test]
    fn an_unreadable_body_is_not_read_as_no_match() {
        let (endpoint, _rx) = serving(&ok_response("<html>captive portal</html>"));
        let client = RangeClient::new(&endpoint);
        let mut waited = Vec::new();
        assert_eq!(
            client.range_with("5BAA6", &mut |duration| waited.push(duration)),
            Err(RangeError::Offline),
            "a 200 full of HTML is a hijacked request, not an empty range"
        );
    }

    /// The request carries five characters, the padding header, and nothing else about anybody.
    ///
    /// This is R-25's packet capture, asserted where it can run on every push. The capture itself
    /// is still owed — it proves the *wire*, and this proves what we handed the client — but the
    /// thing a capture is looking for is a string that should not be in the bytes, and that is
    /// checkable here first.
    #[test]
    fn the_request_carries_the_prefix_the_padding_header_and_nothing_else() {
        let (endpoint, rx) = serving(&ok_response(FIXTURE));
        let client = RangeClient::new(&endpoint);
        let query = query_for("password");

        let range = client
            .range_with(query.prefix(), &mut |_| {})
            .expect("the fixture is served");
        assert_eq!(range.count_for(&query), Some(52_372_427));

        let request = rx.recv().expect("the server saw a request");
        assert!(
            request.starts_with("GET /range/5BAA6 "),
            "five characters and nothing more: {request}"
        );
        // Lowercased, because ureq writes header names through `http::HeaderName` and that
        // normalizes them. Header names are case-insensitive by RFC and the entry check measured
        // `vary: add-padding` coming back from the live service, so the padding is honoured as
        // sent — but a case-sensitive assertion here would have failed against a working client.
        assert!(
            request.to_ascii_lowercase().contains("add-padding: true"),
            "R-25 asks for padding on every request: {request}"
        );
        assert!(
            !request.contains("password") && !request.contains("Forum"),
            "no password and no item title: {request}"
        );
        assert!(
            !request.contains("1E4C9B93F3F0682250B6CF8331B7EE68FD8"),
            "the other 35 characters never leave: {request}"
        );
        assert!(
            !request.to_ascii_lowercase().contains("cookie"),
            "the service sets __cf_bm; we must not send it back: {request}"
        );
    }

    /// A redirect is refused rather than followed — the default this client overrides.
    ///
    /// The one configuration test that is about egress rather than about manners: a followed 302
    /// sends the prefix to a host R-25's capture never recorded, which is the difference between
    /// a k-anonymity claim and a k-anonymity hope.
    ///
    /// **Asserted by watching the redirect target, not by reading the error.** The first version
    /// of this test accepted `Http(302) | Offline`, and it passed with `max_redirects(5)` — the
    /// followed request landed on a dead port, came back `Offline`, and satisfied the disjunction.
    /// A test of an egress rule has to watch the place the bytes would have gone.
    #[test]
    fn a_redirect_is_refused_rather_than_followed() {
        let (elsewhere, saw_us) = serving(&ok_response(FIXTURE));
        let location = format!("Location: {elsewhere}/range/5BAA6");
        let redirect = format!("HTTP/1.1 302 Found\r\n{location}\r\nContent-Length: 0\r\n\r\n");
        let (endpoint, _rx) = serving(&redirect);

        let client = RangeClient::new(&endpoint);
        let refused = client.range_with("5BAA6", &mut |_| {});

        assert!(
            refused.is_err(),
            "a redirect is not a range, got {refused:?}"
        );
        assert!(
            saw_us.try_recv().is_err(),
            "the prefix reached the redirect target — a followed 302 is egress to a host \
             R-25's capture never recorded"
        );
    }

    /// A rate limit backs off and is not hammered — S-07b's "polite caller".
    #[test]
    fn a_rate_limit_backs_off_before_retrying() {
        let (endpoint, _rx) =
            serving("HTTP/1.1 429 Too Many Requests\r\nContent-Length: 0\r\n\r\n");
        let client = RangeClient::new(&endpoint);
        let mut waited: Vec<Duration> = Vec::new();

        let refused = client.range_with("5BAA6", &mut |duration| waited.push(duration));
        assert_eq!(refused, Err(RangeError::Http(429)));
        assert!(
            waited.windows(2).all(|pair| pair[1] > pair[0]),
            "each wait is longer than the last: {waited:?}"
        );
    }

    /// A 4xx that is not a rate limit is an answer, and repeating it is rude rather than robust.
    #[test]
    fn a_client_error_is_not_retried() {
        let (endpoint, _rx) = serving("HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n");
        let client = RangeClient::new(&endpoint);
        let mut waited = Vec::new();

        assert_eq!(
            client.range_with("5BAA6", &mut |duration| waited.push(duration)),
            Err(RangeError::Http(400))
        );
        assert!(waited.is_empty(), "no backoff, because there is no point");
    }

    /// Both failures answer §6.9's `unchecked.reason` vocabulary and nothing else.
    #[test]
    fn every_failure_has_a_word_the_contract_names() {
        assert_eq!(RangeError::Offline.reason(), "offline");
        assert_eq!(RangeError::Http(503).reason(), "http");
    }

    /// Where the proxy and plaintext claims are tested: `tests/hibp_egress.rs`.
    ///
    /// Not here, and the reason is a finding rather than a preference. A unit test reading
    /// `client.agent.config().proxy().is_none()` **passed with the `.proxy(None)` override
    /// deleted**, because `Proxy::try_from_env` returns `None` when the environment holds no
    /// proxy: it measured this machine's shell. The replacement sets `ALL_PROXY` to a listener it
    /// owns and asserts that listener is never contacted, which needs a process to itself.
    #[test]
    fn the_egress_rules_are_tested_where_the_environment_can_be_set() {
        assert_eq!(ENDPOINT, "https://api.pwnedpasswords.com");
    }
}
