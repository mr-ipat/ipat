# R10.04 exact-current-main CI re-verification

Canonical main 2fcee917fd16755a124b8ba0d0654bac3d54cbfe already contains the bounded DEV-01 active-alarm command-shape evidence source. This commit intentionally changes documentation only so GitHub Actions evaluates that exact source tree plus this marker before any source-acceptance closure is claimed.

R10.04 remains read-only and evidence-bounded: exact fixed command only, command echo removed only on exact byte equality, no raw alarm/ONU/serial/subscriber persistence, no physical writes, and no inference that an empty post-echo payload means there are no alarms. The capability remains DEGRADED with no executable alarm endpoint.

Acceptance rule: do not promote R10.04 source or device capability from this marker. Exact-head GitHub CI must pass all canonical jobs. Physical alarm semantics, optics/SNMP, native restore/write and other initial devices remain independently open.
