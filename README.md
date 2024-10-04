# IAM Assumer

Performs assume role with continuously refreshing assume role credentials. Basically a replacement for:

```
export $(printf "AWS_ACCESS_KEY_ID=%s AWS_SECRET_ACCESS_KEY=%s AWS_SESSION_TOKEN=%s" \
$(aws sts assume-role \
--role-arn arn:aws:iam::123456789012:role/MyAssumedRole \
--role-session-name MySessionName \
--query "Credentials.[AccessKeyId,SecretAccessKey,SessionToken]" \
--output text))
```

## Why does this exist?

Wrapper for long running commands that need to perform `AssumeRole` to an IAM role, but does not have support for doing so.

## How do I use it?

Like so:

```
./iam-assumer run  --role-arn arn:aws:iam::128753716591:role/test --role-session-name test -- aws sts get-caller-identity
{
    "UserId": "AROAR36SRZVX5VRHLBHAU:test",
    "Account": "128753716591",
    "Arn": "arn:aws:sts::128753716591:assumed-role/test/test"
}
```

## How does it work?

It spins up a lightweight embeded HTTP server implementing the [container credential provider](https://docs.aws.amazon.com/sdkref/latest/guide/feature-container-credentials.html).
The URI is passed to the command as an environment variable.
On fetching a token from the embedded webserver, an AssumeRole call is made to AWS and the returned credentials as passed to the application.
Since the container credential provider implements refreshing, we're able to use this to refresh the assume role credentials.

# Logging?

You can set the `RUST_LOG` environment variable to one of `error`, `warn`, `info`, `debug`, `trace` for different verbosity levels.
