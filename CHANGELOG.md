Changelog
=========
This changelog follows the patterns described here: https://keepachangelog.com/en/1.0.0/.

Subheadings to categorize changes are `added, changed, deprecated, removed, fixed, security`.

## Unreleased
### added
- Added `--token` as a global CLI flag
- Added `--stdout` to `inventory build`

## 0.2.3
### fixed
- Fixed an issue where successfully updated records would log as unsuccessful

## 0.2.2
### changed
- changed "inventory is updated" to "inventory is up to date" when checking DNS records
### fixed
- Generated inventory files no longer duplicate header information on post-processing
### security
- Updated `clap` to 4.0.29
- Updated `reqwest` to 0.11.13
- Updated `serde_json` to 1.0.89
- Updated `tokio` to 1.23.0
- Updated `serde` to 1.0.150

## 0.2.1
### fixed
- `cddns` is now included in the `PATH` on docker
- Docker image now has `ca-certificates` to authenticate HTTPS requests from `reqwest`

## 0.2.1
### fixed
- `cddns` is now included in the `PATH` on docker
- Docker image now has `ca-certificates` to authenticate HTTPS requests from `reqwest`

## 0.2.0
### added
- Inventories can be built without any records
- Inventory files now save a post-processed version with alternative name/ids as comments
- Inventory files are now saved with a commented header with the date of generation
- All configuration options are now built with `config build`
- Added verbose logging with `-v`
- Added support for `RUST_LOG` environment variable to filter logging
- Added warning for empty inventory
- Provided README instructions for service deployment on Docker Compose
### changed
- The default interval for DNS refresh in `inventory watch` is now 30s, up from 5s
- Requests now have a 10s timeout
- `inventory build` now removes records as you build
- Added `inventory prune` for invalid record pruning
- Added `inventory update` for outdated record updating
- `inventory watch` uses `inventory update`, it no longer automatically prunes
- `--force` flags are now `--force true/false`
- Improved readability of command output
- Improved readability of `show` commands
- `cddns list zones -z <name|id>` now only matches one zone result
- `cddns list records -z <name|id>` now only matches one zone result
- `cddns list records -r <name|id>` now only matches one record result
- Added help link when no token or inventory is provided
### removed
- `inventory commit` is no longer a command
### fixed
- Environment variables work for all commands
- README documentation fixes

## 0.1.2
### security
- Updated clap: 4.0.18 -> 4.0.23
- Updated regex: 1.6.0 -> 1.7.0

## 0.1.1
### changed
- Configuration path no longer needs to exist

## 0.1.0
- Initialize release.