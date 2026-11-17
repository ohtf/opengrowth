# Security policy

Do not report suspected vulnerabilities through public issues, discussions, or pull requests.

Report them privately through GitHub's private vulnerability reporting for this repository (the **Security** tab, then **Report a vulnerability**), or by email to <security@openhealthtechnology.org>. Include enough detail to reproduce the concern without publishing credentials, personal data, or exploit instructions.

The full OHTF security policy, including response times and coordinated disclosure, is at <https://openhealthtechnology.org/community/security/>. This file takes precedence for anything specific to opengrowth.

## Clinical safety

opengrowth calculates anthropometric values (SDS and centiles) used in children's clinical care. A vulnerability that could cause a wrong result to be returned silently, or that could let an attacker alter calculation logic, reference data, or provenance, is a clinical safety concern as well as a security one. Say so clearly in your report; it will be triaged into the hazard log. See [SAFETY.md](SAFETY.md).

## Data

This repository should contain no patient data and no credentials. If you find patient-identifiable data, credentials, or restricted reference material that should not have been committed, report it privately and do not redistribute it.

Reference datasets carry redistribution restrictions independent of any security concern. Unauthorised redistribution of restricted reference data, in particular UK90-derived tables, is a licensing issue and should be reported the same way. See [spec/standard.md](spec/standard.md) section 3.
