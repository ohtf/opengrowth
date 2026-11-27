# Safety

opengrowth is an anthropometric calculation tool. It calculates the SDS and centile of a child's measurement against a growth reference the user chooses, and describes the measurement's position relative to that reference's centile lines.

It **does not diagnose**, and produces no diagnosis, clinical interpretation, recommendation, referral advice, alert, or risk score. opengrowth does not interpret a measurement or decide anything about a child's care: that rests with the healthcare professional, who uses the calculated values alongside the history, examination, and other information.

The full statement, with who and what it is for and the limits of the calculation, is opengrowth's [intended purpose](https://github.com/ohtf/opengrowth/blob/main/qms/intended-purpose.md).

## Not for clinical use yet

opengrowth is pre-alpha:

- Its quality management system is drafted but not in force, and it is not registered as a medical device.
- UK-WHO depends on the UK90 tables, which are included for research and development only while a licence application to UK Research and Innovation is pending. No release will be used for patient care until it is granted.
- The UK-WHO port has not yet had its independent clinical review.

## What opengrowth checks

- A measurement more than 8 SD from the reference median, or 15 SD for BMI, is refused as a probable measurement or data-entry error. This checks the input; it is not a clinical judgement.
- Where a reference has no data for a measurement at an age, opengrowth says so and calculates nothing.
- Every reference table is verified against its recorded checksum before use. A table that does not match is refused, and opengrowth never substitutes another reference.
- Every result names the reference it was calculated against, so that a chart can confirm it is drawn from the same one.

## Reporting a problem

If anything could affect a calculated value, open an [issue](https://github.com/ohtf/opengrowth/issues) and label it `hazard`. [SAFETY.md](https://github.com/ohtf/opengrowth/blob/main/SAFETY.md) describes how hazards are handled.
