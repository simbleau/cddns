# 🚀 CDDNS : Cloudflare Dynamic DNS
**cddns** is a modern, green, hackable DDNS CLI and cloud-native service for [Cloudflare](https://cloudflare.com) built in Rust, featuring interactive builders and layered configuration options.

<TODO: GIF>

---
[![Crates.io](https://img.shields.io/crates/v/cddns)](https://crates.io/crates/cddns)
[![Documentation](https://docs.rs/cddns/badge.svg)](https://docs.rs/cddns)
[![Build Status](https://github.com/simbleau/cddns/workflows/build/badge.svg)](https://github.com/simbleau/cddns/actions/workflows/build.yml)
[![dependency status](https://deps.rs/repo/github/simbleau/cddns/status.svg)](https://deps.rs/repo/github/simbleau/cddns)


# 🇺🇸 Purpose
Dynamic DNS allows experts and home users to keep services available without a static IP address. CDDNS will support low-end hardware and is uncompromisingly green, helping you minimize costs and maximize hardware.

# 🧰 Before: Requirements
- Cloudflare Account ([Docs](https://developers.cloudflare.com/fundamentals/account-and-billing/account-setup/create-account/))
- Cloudflare API Token with **Edit DNS** permissions ([Docs](https://developers.cloudflare.com/fundamentals/api/get-started/create-token/))
- A/AAAA DNS records ([What is a DNS record?](https://www.cloudflare.com/learning/dns/dns-records/))

# 💻 Supported Platforms (Instructions)
- [Native (Windows / MacOS / Unix)](#native)
- Docker, Docker-Compose
- Kubernetes

# Native
## Installation
### Option A: Cargo
- `cargo install cloudflare-ddns`
### Option B: Binary
- Download a compatible binary release from [releases](https://github.com/simbleau/cloudflare-ddns/releases)

## Getting Started
First test your Cloudflare API token with the following command:
> `cddns verify --token <YOUR_CLOUDFLARE_TOKEN>`.

On success, you may see "`This API Token is valid and active`"

## Configuration (CLI)
For CLI usage and testing, we use [TOML files](https://toml.io/en/) to save your local configuration, such as your API key. **You should restrict the permissions on this file.**

To quickly get setup, we offer an interactive configuration file builder.
> `cddns config build`

If you prefer, you can visit [`config.toml`](config.toml) for an annotated example.

### Location
By default, we check `$XDG_CONFIG_HOME/cddns/config.toml` for your configuration file.
- On Linux, this would be `$HOME/.config/cddns/config.toml`
- On MacOS, this would be `$HOME/Library/Application Support/cddns/config.toml`
- On Windows, this would be `%APPData%\cddns\config.toml`

You can set the **CDDNS_CONFIG** environment variable to manually specify the location of this file. [Click here](#environment-variables) for more environment variables.

## Inventory
We use [YAML files](https://yaml.org/) to save which DNS records to watch.

To quickly get setup, we offer an interactive inventory file builder.
> `cddns inventory build`

If you prefer, you can visit [`inventory.yaml`](inventory.yaml) for an annotated example.

### Location
By default, we check `./inventory.yaml` for your inventory file.

You can set the **CDDNS_INVENTORY** environment variable to manually specify the location of this file. [Click here](#environment-variables) for more environment variables.

# Environment Variables
<TODO: Table of env variables>

# Docker
To run this as a Cloudflare DDNS daemon in Docker, here is an example:
```bash
docker service create -d \
  --replicas=1 \
  --name cddns-daemon \
  --mount type=bind,source="$(pwd)"/inventory.yaml \
  -e CDDNS_TOKEN='<YOUR_CLOUDFLARE_TOKEN>' \
  -e CDDNS_WATCH_INTERVAL='5000' \
  simbleau/cddns:latest
```

# Kubernetes
To run this as a Cloudflare DDNS daemon in a cluster, here is an example:
1. Convert your token to base64: `echo -n '<YOUR_CLOUDFLARE_TOKEN>' | base64`
2. Create a secret for your token:
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: cf-token-secret
type: Opaque
data:
  token: MWYyZDFlMmU2N2Rm
```
3. Create a deployment for the DNS utility
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cddns-deployment
spec:
  replicas: 2
  selector:
    matchLabels:
      app: cddns
  template:
    metadata:
      labels:
        app: cddns
    spec:
      volumes:
        - name: inventory-volume
          hostPath:
            path: /path/to/my/inventory.yaml
      containers:
      - name: cddns
        image: simbleau/cddns:latest
        volumeMounts:
        - name: inventory-volume
            mountPath: "inventory.yaml"
            readOnly: true
        env:
        - name: CDDNS_TOKEN
            valueFrom: # Cloudflare API token
            secretKeyRef:
                name: cf-token-secret
                key: token
    env:
    - name: CDDNS_WATCH_INTERVAL
      value: "5000" # Interval (ms) for DNS watch
```
