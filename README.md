# xmip-core-transport-aws

What every AWS technology speaks over HTTP: Signature Version 4, the Query API
and the JSON 1.1 protocol, both sides of each. Not a transport of its own:
[s3](https://github.com/IlleNilsson/xmip-core-transport-s3),
[aws-sqs](https://github.com/IlleNilsson/xmip-core-transport-aws-sqs),
[aws-sns](https://github.com/IlleNilsson/xmip-core-transport-aws-sns) and
[aws-kinesis](https://github.com/IlleNilsson/xmip-core-transport-aws-kinesis)
ride on it, and it rides on
[xmip-core-transport-http](https://github.com/IlleNilsson/xmip-core-transport-http).
A technology of
[xmip-core-transport](https://github.com/IlleNilsson/xmip-core-transport).

| module | what |
| --- | --- |
| `sigv4` | Signature Version 4: a Location signs, a technology's session verifies |
| `query` | the Query API, for aws-sqs and aws-sns |
| `json` | the JSON 1.1 protocol, for aws-kinesis; a `Service` names each one |

Created 2026-09-24 on the owner's ruling of 2026-09-22: what one vendor speaks
leaves the http technology for a crate of that vendor's. The signer and the
Query API lived in http from 2026-09-14, and JSON 1.1 in aws-kinesis
(ADR-0044, amendment 2026-09-24).

## Toolchain

`rust-toolchain.toml` pins the toolchain for the whole estate. Do not change it
here.

## Verification

The included workflow is manual-only and calls the versioned shared workflow at
`IlleNilsson/.github@v1`.
