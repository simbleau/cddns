# CDDNS : Cloudflare Dynamic DNS
**cddns** is a non-complicated, uncompromisingly green DDNS CLI and cloud-native service for [Cloudflare](https://cloudflare.com) built in Rust. Featuring layered configuration and interactive file builders.

**⚠️ WARNING: This project is operational, but not yet considered production ready. [See v1.0 tracking](https://github.com/simbleau/cddns/issues/50)**

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
  - [1.3 Local Installation](#13-local-installation)
    - [Option A: Cargo (Recommended)](#option-a-cargo-recommended)
    - [Option B: Binary](#option-b-binary)
  - [1.4 Docker](#14-docker)
- [2 Quickstart](#2-quickstart)
- [3 Usage](#3-usage)
  - [3.1 Overview](#31-overview)
    - [3.1.1 API Tokens](#311-api-tokens)
    - [3.1.2 Inventory](#312-inventory)
    - [3.1.3 Configuration (Optional)](#313-configuration-optional)
    - [3.1.4 Environment Variables](#314-environment-variables)
  - [3.2 Subcommands](#32-subcommands)
    - [3.2.1 Verify](#321-verify)
    - [3.2.2 Config](#322-config)
      - [3.2.2.1 Show](#3221-show)
      - [3.2.2.2 Build](#3222-build)
    - [3.2.3 List](#323-list)
      - [3.2.3.1 Zones](#3231-zones)
      - [3.2.3.2 Records](#3232-records)
    - [3.2.4 Inventory](#324-inventory)
      - [3.2.4.1 Build](#3241-build)
      - [3.2.4.2 Show](#3242-show)
      - [3.2.4.3 Check](#3243-check)
      - [3.2.4.4 Update](#3244-update)
      - [3.2.4.5 Prune](#3245-prune)
      - [3.2.4.6 Watch](#3246-watch)
  - [3.3 Service Deployment](#33-service-deployment)
    - [3.3.1 Docker](#331-docker)
    - [3.3.2 Docker Compose](#332-docker-compose)
    - [3.3.3 Kubernetes](#333-kubernetes)
    - [3.3.4 Crontab](#334-crontab)
- [4 Purpose](#4-purpose)
- [5 License](#5-license)

# 1 Installation

## 1.1 Supported Platforms
- Command Line Utility
  - Native (Windows / MacOS / Unix)
- Service
  - Docker
  - Docker Compose
  - Kubernetes
  - Crontab

## 1.2 Requirements
- Cloudflare Account ([Help](https://developers.cloudflare.com/fundamentals/account-and-billing/account-setup/create-account/))
- Cloudflare API Token with **Edit DNS** permissions ([Help](https://developers.cloudflare.com/fundamentals/api/get-started/create-token/))

## 1.3 Local Installation
Installing the cddns CLI is a convenient way to test your configuration and build the necessary files for service deployment.

### Option A: Cargo (Recommended)
Cargo is the recommended way to install CDDNS as a CLI ([What is Cargo?](https://doc.rust-lang.org/cargo/)).
- `cargo +nightly install cddns`
### Option B: Binary
- Download a compatible binary release from [releases](https://github.com/simbleau/cloudflare-ddns/releases)

## 1.4 Docker
Any command in this document can be run or tested in a container with [Docker](https://docker.io).

```bash
docker run simbleau/cddns <SUBCOMMAND>
```

# 2 Quickstart
**Docker users: Replace "`cddns`" with "`docker run simbleau/cddns`"**

First, test your Cloudflare API token ([Help](#311-api-tokens)) with the following command:
```bash
cddns --token <YOUR_CLOUDFLARE_TOKEN> verify
```

Next, generate an inventory file ([Help](#312-inventory)) and save it:

*Note: You can add --stdout to redirect the inventory file to the terminal*
```bash
cddns \
  --token <YOUR_CLOUDFLARE_TOKEN> \
  inventory build
```

Check your inventory:
```bash
cddns \
  --token <YOUR_CLOUDFLARE_TOKEN> \
  inventory --path '/path/to/inventory.yml' \
  check
```

If the following works, continue to your OS-specific service instructions:
- [Docker](#331-docker)
- [Docker Compose](#332-docker-compose)
- [Kubernetes](#333-kubernetes)
- [Crontab](#334-crontab)

# 3 Usage
## 3.1 Overview
**cddns** is a non-complicated DDNS tool for Cloudflare, only needing a Cloudflare API token and inventory file. cddns can be run in a container or installed locally as a CLI.

To operate, cddns needs an inventory file containing your DNS records ([What is a DNS record?](https://www.cloudflare.com/learning/dns/dns-records/)), which can be generated or written manually. For configuration, cddns takes the typical layered configuration approach: The config file is the base, which is superseded by environment variables, which are superseded by CLI arguments.

### 3.1.1 API Tokens
cddns will need a valid Cloudflare API token to function ([How do I create an API token?](https://developers.cloudflare.com/fundamentals/api/get-started/create-token/)).

You can test an API token with the following command:
> `cddns verify --token <YOUR_CLOUDFLARE_TOKEN>`.

On success, you may see "`This API Token is valid and active`"

To avoid using `--token` in every command, you can save a [configuration file](#313-configuration-optional) or set the **CDDNS_VERIFY_TOKEN** environment variable to manually specify your token. [Click here](#314-environment-variables) for more environment variables.

### 3.1.2 Inventory
cddns also needs an inventory file in [YAML format](https://yaml.org/) containing the DNS records you want to watch.

By default, we check your local configuration directory for your inventory file.
- On Linux, this would be `$XDG_CONFIG_HOME/cddns/inventory.yml` or `$HOME/.config/cddns/inventory.yml`
- On MacOS, this would be `$HOME/Library/Application Support/cddns/inventory.yml`
- On Windows, this would be `%AppData%\cddns\inventory.yml`

To quickly get setup, the CLI offers an interactive inventory file builder.
> `cddns inventory build`

- **Zones** are domains, subdomains, and identities managed by Cloudflare.
- **Records** are A (IPv4) or AAAA (IPv6) DNS records managed by Cloudflare.

To see DNS records managed by your API token, the CLI also offers a list command.
> `cddns list [records/zones]`

You can visit [`inventory.yml`](inventory.yml) for an annotated example.

You can set the **CDDNS_INVENTORY_PATH** environment variable to manually specify the location of this file. [Click here](#314-environment-variables) for more environment variables.

### 3.1.3 Configuration (Optional)
You may optionally use a [TOML file](https://toml.io/en/) to save configuration, such as your API key. **You should restrict the permissions on this file if storing your API token.**

By default, we check your local configuration directory for your configuration file.
- On Linux, this would be `$XDG_CONFIG_HOME/cddns/config.toml` or `$HOME/.config/cddns/config.toml`
- On MacOS, this would be `$HOME/Library/Application Support/cddns/config.toml`
- On Windows, this would be `%AppData%\cddns\config.toml`

To quickly get setup, the CLI offers an interactive configuration file builder.
> `cddns config build`

You can also visit [`config.toml`](config.toml) for an annotated example.

You can set the **CDDNS_CONFIG** environment variable to manually specify the location of this file. [Click here](#314-environment-variables) for more environment variables.

### 3.1.4 Environment Variables
Every value which can be stored in a [configuration file](#313-configuration-optional) can be superseded or provided as an environment variable.

| Variable Name                      | Description                                                                                                                                                                                                                          | Default                                     | Example                  |
| ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------- | ------------------------ |
| **RUST_LOG**                       | [Log filtering directives](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directiveshttps://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives) | `info,cddns=trace`                          | `debug`                  |
| **CDDNS_CONFIG**                   | The path to your configuration file                                                                                                                                                                                                  | [Varies by OS](#313-configuration-optional) | `/etc/cddns/config.toml` |
| **CDDNS_VERIFY_TOKEN**             | The default Cloudflare API Token to use                                                                                                                                                                                              | None                                        | `GAWnixPCAADXRAjoK...`   |
| **CDDNS_LIST_INCLUDE_ZONES**       | Regex filters for zones to include in CLI usage                                                                                                                                                                                      | `.*` (Match all)                            | `imbleau.com,.*\.dev`    |
| **CDDNS_LIST_IGNORE_ZONES**        | Regex filters for zones to ignore in CLI usage                                                                                                                                                                                       | None                                        | `imbleau.com`            |
| **CDDNS_LIST_INCLUDE_RECORDS**     | Regex filters for records to include in CLI usage                                                                                                                                                                                    | `.*` (Match all)                            | `.*\.imbleau.com`        |
| **CDDNS_LIST_IGNORE_RECORDS**      | Regex filters for records to ignore in CLI usage                                                                                                                                                                                     | None                                        | `shop\..+\.com`          |
| **CDDNS_INVENTORY_PATH**           | The path to your inventory file                                                                                                                                                                                                      | [Varies by OS](#312-inventory)              | `MyInventory.yml`        |
| **CDDNS_INVENTORY_FORCE_UPDATE**   | Skip all prompts (force) for `inventory update`                                                                                                                                                                                      | `false`                                     | `true`                   |
| **CDDNS_INVENTORY_FORCE_PRUNE**    | Skip all prompts (force) for `inventory prune`                                                                                                                                                                                       | `false`                                     | `true`                   |
| **CDDNS_INVENTORY_WATCH_INTERVAL** | The milliseconds between checking DNS records                                                                                                                                                                                        | `30000` (30s)                               | `60000` (60s)            |


## 3.2 Subcommands
**Appending `--help` or `-h` to any command or subcommand will provide additional information.**

The CLI is useful for testing and building files for your service deployment. Below is a reference of all commands in the CLI.

*Reminder: you may add `-h` or `--help` to any subcommand to receive helpful usage information.*

### 3.2.1 Verify
**Help: `cddns verify --help`**

The `verify` command will validate your Cloudflare API token.
```bash
cddns verify [--token '<YOUR_CLOUDFLARE_TOKEN>']
```

If you do not provide `--token ...`, the token will be obtained from your [configuration file](#313-configuration-optional) or the [**CDDNS_VERIFY_TOKEN**](#314-environment-variables) environment variable.

### 3.2.2 Config
**Help: `cddns config --help`**

The `config` command will help you build or manage your configuration ([Help](#313-configuration-optional)). cddns takes the typical layered configuration approach; there are 3 layers. The config file is the base, which is superseded by environment variables, which are superseded by CLI arguments.

By default, cddns checks your [local configuration folder](#313-configuration-optional) for saved configuration.

#### 3.2.2.1 Show
To show your current configuration:

*`-c` or `--config` will show the inventory at the given path*
```bash
cddns config show
```

#### 3.2.2.2 Build
To build a configuration file:

```bash
cddns config build
```

### 3.2.3 List
**Help: `cddns list --help`**

The `list` command will print Cloudflare resources visible with your API token.

- **Zones** are domains, subdomains, and identities managed by Cloudflare.
- **Records** are A (IPv4) or AAAA (IPv6) DNS records managed by Cloudflare.

To list your zones AND records:

*`-include-zones <pattern1,pattern2,..>` will include only zones matching one of the given regex patterns*
*`-ignore-zones <pattern1,pattern2,..>` will ignore zones matching one of the given regex patterns*
*`-include-records <pattern1,pattern2,..>` will include only records matching one of the given regex patterns*
*`-ignore-records <pattern1,pattern2,..>` will ignore records matching one of the given regex patterns*
```bash
cddns list
```

#### 3.2.3.1 Zones
To list only zones:

*`-z` or `--zone` will only show the zone matching the given name or id.*
```bash
cddns list zones
```

#### 3.2.3.2 Records
To list only records:

*`-z` or `--zone` will only show the records matching the given zone's name or id.*
*`-r` or `--record` will only show the records matching the given name or id.*
```bash
cddns list records
```

### 3.2.4 Inventory
**Help: `cddns inventory --help`**

The `inventory` command has several subcommands to build and control inventory.

*`-p` or `--path` will show the inventory at the given path*

#### 3.2.4.1 Build
To build an inventory:

*`--stdout` will output the inventory to stdout*\
*`--clean` will output without post-processing*
```bash
cddns inventory build
```

#### 3.2.4.2 Show
To show your inventory:

*`--clean` will output without post-processing*
```bash
cddns inventory show
```

#### 3.2.4.3 Check
To check your DNS records, without making any changes:
```bash
cddns inventory check
```

#### 3.2.4.4 Update
To update all outdated DNS records found in `inventory check`:

*`--force-update true` will attempt to skip prompts*
```bash
cddns inventory update
```

#### 3.2.4.5 Prune
To prune all invalid DNS records found in `inventory check`:

*`--force-prune true` will attempt to skip prompts*
```bash
cddns inventory prune
```

#### 3.2.4.6 Watch
To continuously update erroneous records:

*`-w` or `--watch-interval` will change the **milliseconds** between DNS refresh*
```bash
cddns inventory watch
```

## 3.3 Service Deployment
cddns will work as a service daemon to keep DNS records up to date. The default check interval is every 30 seconds.

### 3.3.1 Docker
Currently supported architectures: `amd64`, `arm64`

Running cddns on Docker is an easy 3 step process.

1. Test your Cloudflare API token: ([Help](#311-api-tokens))
```bash
export CDDNS_VERIFY_TOKEN='...'
```
```bash
docker run  \
  -e CDDNS_VERIFY_TOKEN \
  simbleau/cddns:latest verify
```

1. Test your inventory ([Help](#312-inventory)).
```bash
export CDDNS_INVENTORY_PATH='/to/your/inventory.yml'
```
```bash
docker run \
  -e CDDNS_VERIFY_TOKEN \
  -e CDDNS_INVENTORY_PATH='/inventory.yml' \
  -v $CDDNS_INVENTORY_PATH:'/inventory.yml' \
  simbleau/cddns:latest inventory check
```

1. Deploy

*All [environment variables](#314-environment-variables) can be used for additional configuration.*
```bash
docker run \
  -e CDDNS_VERIFY_TOKEN \
  -e CDDNS_INVENTORY_PATH='/inventory.yml' \
  -v $CDDNS_INVENTORY_PATH:/inventory.yml \
  simbleau/cddns:latest
```

### 3.3.2 Docker Compose
1. Validate your configuration with the [Docker instructions](#331-docker) (above)

2. Deploy Compose file[[?](https://docs.docker.com/compose/compose-file/)]

*All [environment variables](#314-environment-variables) can be used for additional configuration.*
```yaml
# docker-compose.yaml
version: '3.3'
services:
    cddns:
        environment:
            - CDDNS_VERIFY_TOKEN
            - CDDNS_INVENTORY_PATH='/inventory.yml'
        volumes:
            - /host/path/to/inventory.yml:/inventory.yml
        image: 'simbleau/cddns:latest'
```
```bash
docker compose up
```

### 3.3.3 Kubernetes
We will eventually support standard installation techniques such as Helm. You may try a custom setup or you may follow our imperative steps with the help of the cddns CLI:

1. Create a Secret[[?](https://kubernetes.io/docs/concepts/configuration/secret/)] for your API token:
```
kubectl create secret generic cddns-api-token \
  --from-literal=token='YOUR_CLOUDFLARE_API_TOKEN'
```

2. Create a ConfigMap[[?](https://kubernetes.io/docs/concepts/configuration/configmap/)] for your inventory:
```
kubectl create configmap cddns-inventory \
  --from-file '/to/your/inventory.yml'
```

3. Create a Deployment[[?](https://kubernetes.io/docs/concepts/workloads/controllers/deployment/)]:

*All [environment variables](#314-environment-variables) can be used for additional configuration.*
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
            mountPath: /opt/etc/cddns/
            readOnly: true
        env:
        - name: CDDNS_INVENTORY_PATH
          value: /opt/etc/cddns/inventory.yml
        - name: CDDNS_VERIFY_TOKEN
            valueFrom: # Cloudflare API token
              secretKeyRef:
                  name: cddns-api-token
                  key: token
```

1. Deploy:
```
kubectl apply -f deployment.yaml
```
### 3.3.4 Crontab
1. Test your Cloudflare API token: ([Help](#311-getting-started))
```bash
cddns verify
```

1. Test your inventory ([Help](#312-inventory)).
```bash
cddns inventory show
```

1. Launch crontab editor
```bash
sudo crontab -e
```

1. Add crontab entry (e.g. every 10 minutes)
```
*/10 * * * * "cfddns inventory --force-update true update"
```

---

# 4 Purpose
cddns allows experts and home users to keep services available without a static IP address. CDDNS will support low-end hardware and is uncompromisingly green, helping you minimize costs and maximize hardware.

# 5 License
This project is dual-licensed under both [Apache 2.0](LICENSE-APACHE) and [MIT](LICENSE-MIT) licenses.