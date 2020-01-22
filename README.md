# marco

This command-line tool does the following:

1. Queries for your public IP address.
2. Updates your Cloudflare DNS entry to match the current IP address.

Note that it was written primarily as a hobby project, for helping
automate my (very basic) home network. You might want to use a
[more mature dynamic DNS solution][ddns].

# Usage

You'll need to pass the following information via environment variables
or command-line arguments:

`CLOUDFLARE_API_TOKEN`: a [Cloudflare API token][cft] with the following
permissions: `#zone:read` and `#dns_records:edit`.

`CLOUDFLARE_ZONE`: the name of the zone you'd like to update,
e.g. `foo.com` or `bar.io`.

`CLOUDFLARE_DNS_RECORD`: the name of the record within the zone
you'd like to update, e.g. `baz.foo.com` or `www.bar.io`.

`marco` runs once and is meant to be scheduled by a tool like [cron][cron].
I run `marco` once every 5 minutes with the following cronjob:

```cron
CLOUDFLARE_API_TOKEN=<REDACTED>
CLOUDFLARE_ZONE=<REDACTED>
CLOUDFLARE_DNS_RECORD=<REDACTED>
*/5 * * * * /usr/local/bin/marco
```

# Misc.

The name refers to the game [Marco Polo][mp], which this process vaguely reminds me of.

[cft]: https://support.cloudflare.com/hc/en-us/articles/200167836-Managing-API-Tokens-and-Keys  
[cron]: https://en.wikipedia.org/wiki/Cron
[ddns]: https://wiki.archlinux.org/index.php/Dynamic_DNS
[mp]: https://en.wikipedia.org/wiki/Marco_Polo_(game)
