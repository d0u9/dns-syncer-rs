# dns-syncer-rs

---

This Rust-powered tool is used to synchronize your local DNS configuration file with a remote DNS provider.

**Motivation**: I want to find a Dynamic DNS (DDNS) client to automatically update my DNS record in Cloudflare based on the public IP assigned to my home broadband. However, after extensive searching on GitHub, it appears that only a Python-based tool meets my need. I'm not a fan of Python, and I prefer everything to be bundled into a statically-linked binary rather than pieces of libraries or scripts. That's why I created this project and implemented it using Rust, the high-performance language.

# Supported DNS provider

Currently, only **Cloudfare** is supported. But the it is simple to add new provider by adding a new implementation if the `backend` directory.

Provider is called **backend** in this project.

# How to use

Simply create a yaml configuration file like this:

```yaml
# The interval in seconds indicates how often this tool synchronizes with the remote.
# 'check_interval == 0' means run this tool only once and then exit.
check_interval: 30
backends:
# The only supported backed is cloudflare
- provider: cloudflare
  authentication:
    api_token: AABBCCDDEEFFGG
  zones:
  - id: 112233445566
    records:
    - type: A
      name: test1.example-au.org
      proxied: true
      content: 8.8.8.8
      comment: DNS Syncer, google dns
    - type: A
      proxied: false
      name: test2.example-au.org
      comment: test2 only
      # If "content" field is not specified, the default is to use the 
      #public IP address obtained from 'https://1.1.1.1/cdn-cgi/trace' on Cloudflare.
```

# Want to run this in a container

```
docker d0u9/dns-syncer
```

Then a compose yaml:

```yaml
services:
  dns-syncer:
    container_name: dns-syncer
    hostname: dns-syncer
    image: d0u9/dns-syncer
    restart: always
    environment:
      - CONFIG_FILE=/config/dns-syncer.yaml
    volumes:
      - type: bind
        source: /path/to/you/config/dir
        target: /config
        read_only: true
```

# Not implemented

Ipv6 currently isn't supported yet.


