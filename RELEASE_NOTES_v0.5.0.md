# TCHLauncher v0.5.0

This release adds real local network information to the server overview.

## Changes

- The connection section now reads the server's configured port and listening address from `server.properties`, instead of displaying a fixed `localhost` address.
- When a LAN address can be detected, it is shown in a selectable field with a Copy button. The local TCP port is checked and labeled as responding, available, occupied, or not yet responding.
- An optional public IP lookup is available inside the internet connection guide. It runs only when requested and contacts ipify; it does not open a router port or prove that the server can be reached from outside the LAN.
- The guide explains the manual TCP port forwarding step and the need to allow Java through the firewall.

## Verification

- TypeScript compilation and Rust tests passed. The network tests cover occupied and available local ports, address formatting, and a loopback-only server binding.
- The Windows installer was built locally. Visual review and installation on a separate machine remain to be done.

## Known limitations

- Automatic UPnP/NAT-PMP mapping and remote port verification are not implemented yet. LAN discovery uses the selected IPv4 route and may need manual checking on computers with several active network adapters.
- The Windows installer is not digitally signed.
