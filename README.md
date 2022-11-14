# CDDNS : Cloudflare Dynamic DNS
**cddns** is a modern, green, hackable DDNS CLI and cloud-native service for [Cloudflare](https://cloudflare.com) built in Rust, featuring interactive builders and layered configuration options.

<TODO: GIF>

---
[![Crates.io](https://img.shields.io/crates/v/cddns)](https://crates.io/crates/cddns)
[![dependency status](https://deps.rs/repo/github/simbleau/cddns/status.svg)](https://deps.rs/repo/github/simbleau/cddns)
[![CI](https://github.com/simbleau/cddns/actions/workflows/ci.yaml/badge.svg)](https://github.com/simbleau/cddns/actions/workflows/ci.yaml)

# Table of Contents
- [CDDNS : Cloudflare Dynamic DNS](#cddns--cloudflare-dynamic-dns)
- [Table of Contents](#table-of-contents)
- [1 Installation](#1-installation)
  - [1.1 Supported Platforms](#11-supported-platforms)
  - [1.2 Requirements](#12-requirements)
  - [1.3 CLI Download](#13-cli-download)
    - [Option A: Cargo](#option-a-cargo)
    - [Option B: Binary](#option-b-binary)
- [2 Usage](#2-usage)
  - [2.1 Overview](#21-overview)
    - [2.1.1 Getting Started](#211-getting-started)
    - [2.1.2 Configuration](#212-configuration)
    - [2.1.3 Environment Variables](#213-environment-variables)
    - [2.1.4 Inventory](#214-inventory)
  - [2.2 CLI Commands](#22-cli-commands)
    - [2.2.1 Verify](#221-verify)
    - [2.2.2 Config](#222-config)
    - [2.2.3 List](#223-list)
    - [2.2.4 Inventory](#224-inventory)
  - [2.3 Service Deployment](#23-service-deployment)
    - [2.3.1 Docker](#231-docker)
    - [2.3.2 Kubernetes](#232-kubernetes)
    - [2.3.3 Crontab](#233-crontab)
- [3 Purpose](#3-purpose)
- [4 License](#4-license)

# 1 Installation

## 1.1 Supported Platforms
- Command Line Utility
  - Native (Windows / MacOS / Unix)
- Service
  - Docker
  - Kubernetes
  - Crontab (Native)

## 1.2 Requirements
- Cloudflare Account ([Help](https://developers.cloudflare.com/fundamentals/account-and-billing/account-setup/create-account/))
- Cloudflare API Token with **Edit DNS** permissions ([Help](https://developers.cloudflare.com/fundamentals/api/get-started/create-token/))
- A/AAAA DNS records ([What is a DNS record?](https://www.cloudflare.com/learning/dns/dns-records/))

## 1.3 CLI Download
### Option A: Cargo
Cargo is the recommended way to install CDDNS as a CLI ([What is Cargo?](https://doc.rust-lang.org/cargo/)).
- `cargo install cddns`
### Option B: Binary
- Download a compatible binary release from [releases](https://github.com/simbleau/cloudflare-ddns/releases)

# 2 Usage

## 2.1 Overview
**cddns** uses a Cloudflare API token to see and edit your DNS records. In both usage as a CLI and a service, an inventory file is used to know which DNS records to watch. For configuration, cddns takes the typical layered configuration approach. There are 3 layers. The config file is the base, which is then superseded by environment variables, which are finally superseded by CLI arguments and options.

### 2.1.1 Getting Started
**Appending `--help` to any command or subcommand will provide additional information.**

First test a Cloudflare API token with the following command:
> `cddns verify --token <YOUR_CLOUDFLARE_TOKEN>`.

On success, you may see "`This API Token is valid and active`"

You can also set the **CDDNS_VERIFY_TOKEN** environment variable to manually specify your token. [Click here](#213-environment-variables) for more environment variables.

### 2.1.2 Configuration
For CLI usage and testing, you may use a [TOML file](https://toml.io/en/) to save configuration, such as your API key. **You should restrict the permissions on this file if storing your API token.**

By default, we check your local configuration directory for your configuration file.
- On Linux, this would be `$XDG_CONFIG_HOME/cddns/config.toml` or `$HOME/.config/cddns/config.toml`
- On MacOS, this would be `$HOME/Library/Application Support/cddns/config.toml`
- On Windows, this would be `%AppData%\cddns\config.toml`

To quickly get setup, we offer an interactive configuration file builder.
> `cddns config build`

You can also visit [`config.toml`](config.toml) for an annotated example.

You can set the **CDDNS_CONFIG** environment variable to manually specify the location of this file. [Click here](#213-environment-variables) for more environment variables.

### 2.1.3 Environment Variables
Every value which can be stored in a [configuration file](#212-configuration) can be superseded or provided as an environment variable.

| Variable Name                  | Description                                         | Default                            | Example                  |
| ------------------------------ | --------------------------------------------------- | ---------------------------------- | ------------------------ |
| **CDDNS_CONFIG**               | The path to your configuration file                 | [Varies by OS](#212-configuration) | `/etc/cddns/config.toml` |
| **CDDNS_VERIFY_TOKEN**         | The default Cloudflare API Token to use             | None                               | `GAWnixPCAADXRAjoK...`   |
| **CDDNS_INVENTORY_PATH**       | The path to your inventory file                     | `inventory.yaml`                   | `MyInventory.yml`        |
| **CDDNS_LIST_INCLUDE_ZONES**   | Regex filters for zones to include in CLI usage     | `.*` (Match all)                   | `imbleau.com,.*\.dev`    |
| **CDDNS_LIST_INCLUDE_RECORDS** | Regex filters for records to include in CLI usage   | `.*` (Match all)                   | `.*\.imbleau.com`        |
| **CDDNS_LIST_IGNORE_ZONES**    | Regex filters for zones to ignore in CLI usage      | None                               | `imbleau.com`            |
| **CDDNS_LIST_IGNORE_RECORDS**  | Regex filters for records to ignore in CLI usage    | None                               | `shop\..+\.com`          |
| **CDDNS_COMMIT_FORCE**         | Force commit (Do not prompt) for `inventory commit` | `false`                            | `true`                   |
| **CDDNS_WATCH_INTERVAL**       | The milliseconds between checking DNS records       | `5000` (5s)                        | `60000` (60s)            |

### 2.1.4 Inventory
cddns uses [YAML files](https://yaml.org/) to save which DNS records to watch.

- **Zones** are domains, subdomains, and identities managed by Cloudflare.
- **Records** are A (IPv4) or AAAA (IPv6) DNS records managed by Cloudflare.

To see DNS records managed by your API token, we offer a list command.
> `cddns list [records/zones]`

To quickly get setup, we offer an interactive inventory file builder.
> `cddns inventory build`

If you prefer, you can visit [`inventory.yaml`](inventory.yaml) for an annotated example.

By default, we check the current directory for an `inventory.yaml` file.

You can set the **CDDNS_INVENTORY** environment variable to manually specify the location of this file. [Click here](#213-environment-variables) for more environment variables.

## 2.2 CLI Commands

### 2.2.1 Verify
**Help: `cddns verify --help`**

The `verify` command will attempt to authenticate using your Cloudflare API token.

If you do not provide `--token ...`, the token will be obtained from your [configuration file](#212-configuration) or the [**CDDNS_VERIFY_TOKEN**](#213-environment-variables) environment variable.

Example:
```bash
cddns verify --token 'YOUR_CLOUDFLARE_TOKEN'
```

### 2.2.2 Config
**Help: `cddns config --help`**

The `config` command will help you build or manage your configuration ([Configuration help](#212-configuration)). cddns takes the typical layered configuration approach. There are 3 layers. The config file is the base, which is then superseded by environment variables, which are finally superseded by CLI arguments and options.

To show your configuration:
Example:
```bash
cddns config show
```

To build a configuration file:
Example:
```bash
cddns config build
```

By default, cddns checks your [local configuration folder](#212-configuration) for saved configuration ([More](#212-configuration)).

### 2.2.3 List
**Help: `cddns list --help`**

The `list` command will print Cloudflare resources.

To show your zones AND records:
Example:
```bash
cddns list
```

To show only zones:
Example:
```bash
cddns list zones
```

To show only records:
Example:
```bash
cddns list records
```

### 2.2.4 Inventory
**Help: `cddns inventory --help`**

The `inventory` command has several subcommands to manage, build, or show your inventory.

To build an inventory:
Example:
```bash
cddns inventory build
```

To show or validate an inventory:
Example:
```bash
cddns inventory [--path 'inventory.yaml'] show
```

To check an inventory, without making any changes:
Example:
```bash
cddns inventory check
```

To fix erroneous records discovered via `check`:
Example:
```bash
cddns inventory commit [--force]
```

To continuously fix erroneous records:
Example:
```bash
cddns inventory watch [--interval 5000]
```

## 2.3 Service Deployment
cddns will work as a service daemon to keep DNS records up to date. The default check interval is every 5 seconds.

### 2.3.1 Docker
Once you have tested your token and built an inventory file with the CLI, you can deploy via Docker.
1. Test your token ([Help](#211-getting-started)).
```bash
docker run  \
  -e CDDNS_VERIFY_TOKEN='<YOUR_CLOUDFLARE_TOKEN>' \
  simbleau/cddns:latest verify
```
2. Test your inventory ([Help](#214-inventory)).
```bash
docker run \
  -e CDDNS_VERIFY_TOKEN='<YOUR_CLOUDFLARE_TOKEN>' \
  -v $(pwd)/inventory.yaml:/inventory.yaml \
  simbleau/cddns:latest inventory show
```
3. Deploy
```bash
docker run \
  -e CDDNS_VERIFY_TOKEN='<YOUR_CLOUDFLARE_TOKEN>' \
  -v $(pwd)/inventory.yaml:/inventory.yaml \
  simbleau/cddns:latest
```

### 2.3.2 Kubernetes
We will eventually support standard installation techniques such as Helm. You may try a custom setup or you may follow our imperative steps:

1. Ensure your token is valid with the CLI ([Help](#211-getting-started)).
```bash
cddns verify --token 'YOUR_CLOUDFLARE_TOKEN'
```
2. Ensure your inventory is valid with the CLI ([Help](#214-inventory)).
```bash
cddns inventory --path '/to/your/inventory.yaml'  show
```
3. Create secret for your API token:
```
kubectl create secret generic cddns-api-token \
  --from-literal=token='YOUR_CLOUDFLARE_API_TOKEN'
```
1. Create config map from your DNS record inventory:
```
kubectl create configmap cddns-inventory \
  --from-file '/to/your/inventory.yaml'
```
5. Save deployment YAML
```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cddns-deployment
spec:
  replicas: 1
  selector:
    matchLabels:
      app: cddns
  template:
    metadata:
      labels:
        app: cddns
    spec:
      volumes: # Expose inventory as volume
        - name: "inventory"
          configMap:
            name: "cddns-inventory"
      containers:
      - name: cddns
        image: simbleau/cddns:latest
        volumeMounts:
        - name: inventory # Mount inventory file
            mountPath: "inventory.yaml"
            readOnly: true
        env:
        - name: CDDNS_VERIFY_TOKEN
            valueFrom: # Cloudflare API token
            secretKeyRef:
                name: cddns-api-token
                key: token
```
6. Apply deployment
```
kubectl apply -f deployment.yaml
```
### 2.3.3 Crontab
1. Ensure your token is valid ([Help](#211-getting-started)).
```bash
cddns verify
```
2. Ensure your inventory is valid ([Help](#214-inventory)).
```bash
cddns inventory show
```
3. Launch crontab editor
```bash
sudo crontab -e
```
4. Add crontab entry (e.g. every 10 minutes)
```
*/10 * * * * "cfddns inventory commit --force"
```

---

# 3 Purpose
Dynamic DNS allows experts and home users to keep services available without a static IP address. CDDNS will support low-end hardware and is uncompromisingly green, helping you minimize costs and maximize hardware.

# 4 License
This project is dual-licensed under both [Apache 2.0](LICENSE-APACHE) and [MIT](LICENSE-MIT) licenses.