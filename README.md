# 🚀 CFDDNS (Cloudflare Dynamic DNS)
A modern, hackable, green DDNS CLI and service for Cloudflare. Built for native and scale, featuring interactive builders and layered configuration options.

# 🇺🇸 Purpose
Dynamic DNS allows experts and home users to keep services available without a static IP address. CFDDNS will support low-end hardware and is uncompromisingly green, helping you minimize costs and maximize hardware.

# 🧰 Before: Requirements
- Cloudflare Account ([Docs](https://developers.cloudflare.com/fundamentals/account-and-billing/account-setup/create-account/))
- Cloudflare API Token ([Docs](https://developers.cloudflare.com/fundamentals/api/get-started/create-token/))
- Existing DNS records ([What is a DNS record?](https://www.cloudflare.com/learning/dns/dns-records/))

# 💻 Supported Platforms (Instructions)
- Native (Windows / MacOS / Unix)
- Docker, Docker-Compose
- Kubernetes

# Native
## Installation
### Option A: Cargo
- `cargo install cloudflare-ddns`
### Option B: Binary
- Download a compatible binary from [releases](https://github.com/simbleau/cloudflare-ddns/releases)

## Getting Started
TODO


## Build your config
- Run `cfddns build config` to run an interactive configuration builder
- You can visit `CFDDNS.toml`[CFDDNS.toml] for an annotated example.

## Build your DNS record inventory
- Run `cfddns build inventory` to run an interactive inventory builder
- You can visit `CFDDNS_INVENTORY.yaml`[CFDDNS_INVENTORY.yaml] for an annotated example.

## Testing
1. Locate your `CFDDNS.toml` (config) file and your `CFDDNS_INVENTORY.yaml` (inventory) file
   - CFDDNS expects these files in the working directory, or:
     - You can set the `CFDDNS_CONFIG` environment variable or add `-c <PATH>` in the CLI to change the config location.
     - You can set the `CFDDNS_INVENTORY` environment variable or add `-i <PATH>` in the CLI to change the inventory location.
2. Run `cfddns verify` to test authentication
3. Run `cfddns list` to list managed items
4. Run `cfddns check` to check outdated DNS records
5. Run `cfddns run` to commit DNS record updates found in `check`
6. Run `cfddns watch` to continually check for DNS record updates on loop

## Configuration
<TODO: Table of env variables>

# Docker
To run this as a Cloudflare DDNS daemon in Docker, here is an example:
```bash
docker service create -d \
  --replicas=1 \
  --name cfddns-daemon \
  --mount type=bind,source="$(pwd)"/CFDDNS.toml \
  --mount type=bind,source="$(pwd)"/CFDDNS_INVENTORY.yaml \
  -e CFDDNS_WATCH_INTERVAL='5000' \
  simbleau/cfddns:latest
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
  name: cfddns-deployment
spec:
  replicas: 2
  selector:
    matchLabels:
      app: cfddns
  template:
    metadata:
      labels:
        app: cfddns
    spec:
      volumes:
        - name: inventory-volume
          hostPath:
            path: CFDDNS_INVENTORY.yaml
      containers:
      - name: cfddns
        image: simbleau/cfddns:latest
        volumeMounts:
        - name: inventory-volume
            mountPath: "CFDDNS_INVENTORY.yaml"
            readOnly: true
        env:
        - name: CFDDNS_VERIFY_TOKEN
            valueFrom:
            secretKeyRef:
                name: cf-token-secret
                key: token
    env:
    - name: CFDDNS_WATCH_INTERVAL
      value: "5000" # Interval (ms) for DNS watch
```
