# Security policy

Report a vulnerability privately through a
[private advisory](https://github.com/buildc-org/common/security/advisories/new),
or to security@buildc.org. Do not open a public issue for it.

`common` publishes no releases. `buildc` and `bootc-anywhere` each pin it by
revision. Only the revision that the latest release of each pins receives
fixes.

The [threat model](https://buildc-org.github.io/buildc/threat-model/) of
`buildc` covers this repository too. It names each trust boundary, the control
at each one, and the risks the project accepts.
