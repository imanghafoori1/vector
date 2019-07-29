---
description: Ingests data through log records from journald and outputs `log` events.
---

<!--
     THIS FILE IS AUTOOGENERATED!

     To make changes please edit the template located at:

     scripts/generate/templates/docs/usage/configuration/sources/journald.md.erb
-->

# journald source

![][images.journald_source]

{% hint style="warning" %}
The `journald` sink is in beta. Please see the current
[enhancements][url.journald_source_enhancements] and
[bugs][url.journald_source_bugs] for known issues.
We kindly ask that you [add any missing issues][url.new_journald_source_issue]
as it will help shape the roadmap of this component.
{% endhint %}

The `journald` source ingests data through log records from journald and outputs [`log`][docs.log_event] events.

## Config File

{% code-tabs %}
{% code-tabs-item title="vector.toml (example)" %}
```coffeescript
[sources.my_journald_source_id]
  type = "journald" # must be: "journald"
  
  current_runtime_only = true # default
  local_only = true # default
  units = ["ntpd", "sysinit.target"] # default
```
{% endcode-tabs-item %}
{% code-tabs-item title="vector.toml (schema)" %}
```coffeescript
[sources.<source-id>]
  type = "journald"
  current_runtime_only = <bool>
  local_only = <bool>
  units = ["<string>", ...]
```
{% endcode-tabs-item %}
{% endcode-tabs %}

## Options

| Key  | Type  | Description |
|:-----|:-----:|:------------|
| **REQUIRED** | | |
| `type` | `string` | The component type<br />`required` `enum: "journald"` |
| **OPTIONAL** | | |
| `current_runtime_only` | `bool` | Include only entries from the current runtime (boot)<br />`default: true` |
| `local_only` | `bool` | Include only entries from the local system<br />`default: true` |
| `units` | `[string]` | The list of units names to monitor. If empty or not present, all units are accepted. Unit names lacking a `"."` will have `".service"` appended to make them a valid service unit name.<br />`default: []` |

## Examples

Given the following journald record:

{% code-tabs %}
{% code-tabs-item title="journald record" %}

```
__REALTIME_TIMESTAMP=1564173027000443
__MONOTONIC_TIMESTAMP=98694000446
_BOOT_ID=124c781146e841ae8d9b4590df8b9231
SYSLOG_FACILITY=3
_UID=0
_GID=0
_CAP_EFFECTIVE=3fffffffff
_MACHINE_ID=c36e9ea52800a19d214cb71b53263a28
_HOSTNAME=lorien.example.com
PRIORITY=6
_TRANSPORT=stdout
_STREAM_ID=92c79f4b45c4457490ebdefece29995e
SYSLOG_IDENTIFIER=ntpd
_PID=2156
_COMM=ntpd
_EXE=/usr/sbin/ntpd
_CMDLINE=ntpd: [priv]
_SYSTEMD_CGROUP=/system.slice/ntpd.service
_SYSTEMD_UNIT=ntpd.service
_SYSTEMD_SLICE=system.slice
_SYSTEMD_INVOCATION_ID=496ad5cd046d48e29f37f559a6d176f8
MESSAGE=reply from 192.168.1.2: offset -0.001791 delay 0.000176, next query 1500s
```
{% endcode-tabs-item %}
{% endcode-tabs %}

A [`log` event][docs.log_event] will be emitted with the following structure:

{% code-tabs %}
{% code-tabs-item title="log" %}
```javascript
{
  "timestamp": <2019-07-26T20:30:27.000443Z>,
  "message": "reply from 192.168.1.2: offset -0.001791 delay 0.000176, next query 1500s",
  "host": "lorien.example.com",
  "__REALTIME_TIMESTAMP": "1564173027000443",
  "__MONOTONIC_TIMESTAMP": "98694000446",
  "_BOOT_ID": "124c781146e841ae8d9b4590df8b9231",
  "SYSLOG_FACILITY": "3",
  "_UID": "0",
  "_GID": "0",
  "_CAP_EFFECTIVE": "3fffffffff",
  "_MACHINE_ID": "c36e9ea52800a19d214cb71b53263a28",
  "PRIORITY": "6",
  "_TRANSPORT": "stdout",
  "_STREAM_ID": "92c79f4b45c4457490ebdefece29995e",
  "SYSLOG_IDENTIFIER": "ntpd",
  "_PID": "2156",
  "_COMM": "ntpd",
  "_EXE": "/usr/sbin/ntpd",
  "_CMDLINE": "ntpd: [priv]",
  "_SYSTEMD_CGROUP": "/system.slice/ntpd.service",
  "_SYSTEMD_UNIT": "ntpd.service",
  "_SYSTEMD_SLICE": "system.slice",
  "_SYSTEMD_INVOCATION_ID": "496ad5cd046d48e29f37f559a6d176f8"
}
```

Vector extracts the `"MESSAGE"` field as `"message"`, `"_HOSTNAME"` as `"host"`, and parses `"_SOURCE_REALTIME_TIMESTAMP"` into `"timestamp"`. All other fields from journald are kept intact from the source record. You can further parse the `"message"` key with a [transform][docs.transforms], such as the [`regeex` transform][docs.regex_parser_transform].
{% endcode-tabs-item %}
{% endcode-tabs %}

## How It Works

### Delivery Guarantee

Due to the nature of this component, it offers a
[**best effort** delivery guarantee][docs.best_effort_delivery].

### Environment Variables

Environment variables are supported through all of Vector's configuration.
Simply add `${MY_ENV_VAR}` in your Vector configuration file and the variable
will be replaced before being evaluated.

You can learn more in the [Environment Variables][docs.configuration.environment-variables]
section.

## Troubleshooting

The best place to start with troubleshooting is to check the
[Vector logs][docs.monitoring_logs]. This is typically located at
`/var/log/vector.log`, then proceed to follow the
[Troubleshooting Guide][docs.troubleshooting].

If the [Troubleshooting Guide][docs.troubleshooting] does not resolve your
issue, please:

1. Check for any [open sink issues][url.journald_source_issues].
2. [Search the forum][url.search_forum] for any similar issues.
2. Reach out to the [community][url.community] for help.

## Resources

* [**Issues**][url.journald_source_issues] - [enhancements][url.journald_source_enhancements] - [bugs][url.journald_source_bugs]
* [**Source code**][url.journald_source_source]


[docs.best_effort_delivery]: ../../../about/guarantees.md#best-effort-delivery
[docs.configuration.environment-variables]: ../../../usage/configuration#environment-variables
[docs.log_event]: ../../../about/data-model.md#log
[docs.monitoring_logs]: ../../../usage/administration/monitoring.md#logs
[docs.regex_parser_transform]: ../../../usage/configuration/transforms/regex_parser.md
[docs.transforms]: ../../../usage/configuration/transforms
[docs.troubleshooting]: ../../../usage/guides/troubleshooting.md
[images.journald_source]: ../../../assets/journald-source.svg
[url.community]: https://vector.dev/community
[url.journald_source_bugs]: https://github.com/timberio/vector/issues?q=is%3Aopen+is%3Aissue+label%3A%22Source%3A+journald%22+label%3A%22Type%3A+Bug%22
[url.journald_source_enhancements]: https://github.com/timberio/vector/issues?q=is%3Aopen+is%3Aissue+label%3A%22Source%3A+journald%22+label%3A%22Type%3A+Enhancement%22
[url.journald_source_issues]: https://github.com/timberio/vector/issues?q=is%3Aopen+is%3Aissue+label%3A%22Source%3A+journald%22
[url.journald_source_source]: https://github.com/timberio/vector/tree/master/src/sources/journald.rs
[url.new_journald_source_issue]: https://github.com/timberio/vector/issues/new?labels=Source%3A+journald
[url.search_forum]: https://forum.vector.dev/search?expanded=true
