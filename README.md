# CFDDNS
A simple, modern, and secure Cloudfare DDNS command line utility.

Featuring 

# Pre-requisite: Cloudfare Token
You will need a Cloudfare API token.
1. Create API Token https://dash.cloudflare.com/profile/api-tokens
2. Permissions: Zone | DNS | Edit
3. Save your token somewhere safe. It is a password.

# Install
1. `cargo install cloudfare-ddns`
2. Run `cfddns build config` to run an interactive configuration builder
3. Run `cfddns build inventory` to run an interactive inventory builder for your daemon

# Execution
1. Locate your `CFDDNS.toml` (config) file and your `CFDDNS_INVENTORY.yaml` (inventory) file
   - CFDDNS expects these files in the working directory, or:
     - You can set the `CFDDNS_CONFIG` environment variable or add `-c <PATH>` in the CLI to change the config location.
     - You can set the `CFDDNS_INVENTORY` environment variable or add `-i <PATH>` in the CLI to change the inventory location.
2. Run `cfddns verify` to test authentication
3. Run `cfddns check` to check outdated DNS records
4. Run `cfddns run` to commit DNS record updates found in `check`
5. Run `cfddns watch` to continually check for DNS record updates on loop

# Docker
To run this as a Cloudfare DDNS daemon in Docker, here is an example:
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
To run this as a Cloudfare DDNS daemon in a cluster, here is an example:
1. Convert your token to base64: `echo -n '<YOUR_CLOUDFARE_TOKEN>' | base64`
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
