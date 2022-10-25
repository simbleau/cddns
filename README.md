# cloudfare-ddns
A Cloudfare DDNS daemon purposed for Kubernetes

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
3. Run `cfddns check` to see pending DNS modifications without applying any modifications
4. Run `cfddns run` to execute the daemon
