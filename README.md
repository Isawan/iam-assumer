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
.iam-assumer run  --role-arn arn:aws:iam::128753716591:role/test --role-session-name test -- aws sts get-caller-identity
{
    "UserId": "AROAR36SRZVX5VRHLBHAU:test",
    "Account": "128753716591",
    "Arn": "arn:aws:sts::128753716591:assumed-role/test/test"
}
```
