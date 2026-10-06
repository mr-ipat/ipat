# R10.02 — Safe C320 PON/ONU REFRESH diagnostics

R10.00/R10.01 prove only fixed card and running-firmware reads on DEV-01. The existing REFRESH action still fails HTTP 503. The private reader intentionally returned only a generic connector error, so the owner could not distinguish an output-boundary failure from a real firmware table-shape mismatch.

R10.02 does not add any CLI command. The persistent connector still accepts only REFRESH, CARDS, and FIRMWARE, and REFRESH still expands only to the pre-existing fixed three read commands for PON 1/1/1. It maps exact known strict-parser exception strings to a short fixed diagnostic code. Unknown exception text remains DEVICE_CONNECTION_FAILED_UNCLASSIFIED. The Rust private API independently allowlists the same codes and maps any arbitrary value back to OWNER_READ_FAILED_OR_INVALID. No raw CLI, host, username, password, serial, subscriber value, or device address may appear.

The transactional owner-private rollout script requires explicit opt-in, refuses root/dirty source/inactive connector, backs up the previous user service unit, restores it on error or signal, verifies the private STATUS contract, then issues exactly one existing REFRESH. Output is restricted to one allowlisted diagnostic code or bounded aggregate counts plus physical_writes=0. It changes no OLT configuration, DNS, firewall, production database, public listener, credential or firmware.

Passing a diagnostic only identifies which strict parser assumption differs from the actual device. It does not validate PON/ONU support. Any later parser adaptation must preserve bounded reads, reject serial/subscriber disclosure, obtain fresh physical evidence and remain a separate reviewed milestone.
