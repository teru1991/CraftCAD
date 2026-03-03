Repro Template (Generated)

以下はアプリが自動生成する再現テンプレです。Issueにそのまま貼れる構造にします。

## Environment
- app_version: {{app_version}}
- build_id: {{build_id}}
- schema_version: {{schema_version}}
- os: {{os}} / {{arch}}
- locale: {{locale}}
- timezone: {{timezone}}
- determinism_tag: {{determinism_tag}}
- limits_profile: {{limits_profile}}

## Inputs (no paths)
{{#each inputs}}
- kind: {{kind}}, sha256: {{sha256}}, size_bytes: {{size_bytes}}
{{/each}}

## Steps (from OpLog)
{{#each steps}}
{{seq}}. {{text}}
{{/each}}

## Expected Result
(What you expected to happen)

## Actual Result
(What happened instead)

## Reason Codes
{{#each reasons}}
- {{code}} (count={{count}})
{{/each}}

## Attachments
- diagnostics_zip: {{zip_name}}
- diagnostics_zip_sha256: {{zip_sha256}}

NOTE:
- Do not include file paths or personal data. All fields are redacted/hashed.
