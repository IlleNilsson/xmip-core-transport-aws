//! The AWS JSON 1.1 protocol: an action named in `X-Amz-Target`, its
//! parameters as one JSON document in a `POST` to `/`, answered in JSON.
//!
//! Kinesis speaks it, as `DynamoDB` does, so both sides are here — a
//! Location forms a request and judges the answer, a far end reads the
//! document back and forms an error. The error names its exception in
//! `__type`, sometimes behind a `#`, and the retry rule reads that name.
//!
//! What differs between services is their [`Service`]: the prefix each
//! target carries and the exceptions worth repeating. aws-kinesis carried
//! this file with its own service baked in until the owner's ruling of
//! 2026-09-22 moved what AWS speaks into the AWS crate (ADR-0044).

use serde_json::{Value, json};
use transport::error::{Result, protocol_error};

use http::status;
use net::http::{Request, Response};

/// The content type every JSON 1.1 request and answer carries.
pub const CONTENT_TYPE: &str = "application/x-amz-json-1.1";

/// One service that speaks JSON 1.1.
#[derive(Clone, Copy, Debug)]
pub struct Service {
    /// How an answer's failure names the service: `Kinesis`.
    pub name: &'static str,
    /// The service and API version every target names:
    /// `Kinesis_20131202`.
    pub target: &'static str,
    /// The exceptions that say come back, beside what HTTP's status says.
    pub repeatable: &'static [&'static str],
}

impl Service {
    /// One request: `document` as the body of a `POST` to `/`, the action
    /// named in `X-Amz-Target`.
    #[must_use]
    pub fn request(&self, action: &str, document: &Value) -> Request {
        Request::new("POST", "/")
            .header("Content-Type", CONTENT_TYPE)
            .header("X-Amz-Target", &format!("{}.{action}", self.target))
            .body(document.to_string().as_bytes())
    }

    /// The far end's side: the action a request names, where it names one
    /// of this service's.
    #[must_use]
    pub fn action<'a>(&self, request: &'a Request) -> Option<&'a str> {
        request
            .header_value("x-amz-target")?
            .strip_prefix(self.target)?
            .strip_prefix('.')
    }

    /// A 2xx answer as the document it carries — `Null` where it carries
    /// none; anything else as a failure naming the status and the
    /// exception, retryable where HTTP or the service says come back.
    ///
    /// # Errors
    /// Where the status is not 2xx, or the answer is not JSON.
    pub fn judge(&self, response: Response) -> Result<Value> {
        let answer = status::judge(self.name, response, kind, |kind| {
            self.repeatable.contains(&kind)
        })?;
        if answer.body.is_empty() {
            return Ok(Value::Null);
        }
        serde_json::from_slice(&answer.body)
            .map_err(|e| protocol_error(format!("an answer that is not JSON: {e}")))
    }
}

/// The far end's side: the document a request carries.
///
/// # Errors
/// Where the body is not JSON.
pub fn document(request: &Request) -> Result<Value> {
    serde_json::from_slice(&request.body)
        .map_err(|e| protocol_error(format!("a request that is not JSON: {e}")))
}

/// The exception an error answer names in `__type`, or nothing.
fn kind(response: &Response) -> String {
    serde_json::from_slice::<Value>(&response.body)
        .ok()
        .and_then(|error| error["__type"].as_str().map(exception))
        .unwrap_or_default()
}

/// The exception a `__type` names, without the namespace some services
/// put before a `#`.
fn exception(kind: &str) -> String {
    kind.rsplit('#').next().unwrap_or(kind).to_string()
}

/// The far end's answer that is a result.
#[must_use]
pub fn answer(document: &Value) -> Response {
    Response::new(200)
        .header("Content-Type", CONTENT_TYPE)
        .body(document.to_string().as_bytes())
}

/// The far end's answer that is not a result: the exception `kind`, with
/// `status`.
#[must_use]
pub fn error(status: u16, kind: &str, message: &str) -> Response {
    let body = json!({ "__type": kind, "message": message });
    Response::new(status)
        .header("Content-Type", CONTENT_TYPE)
        .body(body.to_string().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    const KINESIS: Service = Service {
        name: "Kinesis",
        target: "Kinesis_20131202",
        repeatable: &["ProvisionedThroughputExceededException", "InternalFailure"],
    };

    #[test]
    fn a_request_is_formed_as_the_json_protocol_wants_it_and_reads_back() {
        let sent = KINESIS.request(
            "PutRecord",
            &json!({ "StreamName": "orders", "Data": "VU5B" }),
        );
        assert_eq!(sent.method, "POST");
        assert_eq!(sent.path, "/");
        assert_eq!(sent.header_value("content-type"), Some(CONTENT_TYPE));
        assert_eq!(
            sent.header_value("x-amz-target"),
            Some("Kinesis_20131202.PutRecord")
        );
        assert_eq!(KINESIS.action(&sent), Some("PutRecord"));
        assert_eq!(
            document(&sent).expect("JSON")["StreamName"].as_str(),
            Some("orders")
        );
        assert_eq!(KINESIS.action(&Request::new("POST", "/")), None);
        let other = Request::new("POST", "/").header("X-Amz-Target", "DynamoDB_20120810.Query");
        assert_eq!(KINESIS.action(&other), None);
        assert!(document(&Request::new("POST", "/").body(b"{")).is_err());
    }

    #[test]
    fn a_throttle_or_a_server_failure_is_worth_repeating_and_a_client_one_is_not() {
        assert!(
            KINESIS
                .judge(Response::new(503))
                .expect_err("server")
                .retryable
        );
        assert!(
            KINESIS
                .judge(Response::new(429))
                .expect_err("throttled")
                .retryable
        );
        let throttled = error(400, "ProvisionedThroughputExceededException", "slow down");
        assert!(KINESIS.judge(throttled).expect_err("throughput").retryable);
        let missing = KINESIS
            .judge(error(400, "ResourceNotFoundException", "no stream"))
            .expect_err("not found");
        assert!(!missing.retryable);
        assert_eq!(
            missing.message,
            "Kinesis answered 400 ResourceNotFoundException"
        );
        let namespaced = Response::new(400)
            .body(br#"{"__type":"com.amazon.coral.validate#ValidationException"}"#);
        assert_eq!(
            KINESIS.judge(namespaced).expect_err("validation").message,
            "Kinesis answered 400 ValidationException"
        );
        assert_eq!(
            KINESIS.judge(Response::new(200)).expect("empty"),
            Value::Null
        );
        assert_eq!(
            KINESIS
                .judge(answer(&json!({ "ShardId": "s" })))
                .expect("ok")["ShardId"],
            "s"
        );
        assert!(KINESIS.judge(Response::new(200).body(b"{")).is_err());
    }
}
