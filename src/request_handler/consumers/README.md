# RabbitMQ Consumers

## Create multisig

### Command name: `create_multisig`

### Schema example

```json
{
  "project_id": "UID"
}
```

## Add member

### Command name: `add_member`

### Schema example

```json
{
  "new_member_pubkey": "MIGeMA0GCSqGSIb3DQEBAQUAA4GMADCBiAKBgHO4H2f..."
}
```

## Remove member

### Command name: `remove_member`

### Schema example

```json
{
  "old_member_pubkey": "MIGeMA0GCSqGSIb3DQEBAQUAA4GMADCBiAKBgHO4H2f..."
}
```


## Create proposal

### Command name: `proposal_create`

### Schema example

```json
{
}
```

## Approve proposal

### Command name: `proposal_approve`

### Schema example

```json
{
}
```

## Execute config transaction

### Command name: `config_transaction_execute`

### Schema example

```json
{
  "proposal_executer_pubkey": "MIGeMA0GCSqGSIb3DQEBAQUAA4GMADCBiAKBgHO4H2f..."
}
```

## Change threshold

### Command name: `change_threshold`

### Schema example

```json
{
  "threshold_changer_pubkey": "MIGeMA0GCSqGSIb3DQEBAQUAA4GMADCBiAKBgHO4H2f...",
  "new_threshold": 1
}
```

## Change threshold

### Command name: `change_threshold`

### Schema example

```json
{
  "threshold_changer_pubkey": "MIGeMA0GCSqGSIb3DQEBAQUAA4GMADCBiAKBgHO4H2f...",
  "new_threshold": 1
}
```

## Transfer from vault

### Command name: `transfer_from_vault`

### Schema example

```json
{
  "sender_pubkey": "MIGeMA0GCSqGSIb3DQEBAQUAA4GMADCBiAKBgHO4H2f...",
  "receiver_pubkey": "MIGeMA0GCSqGSIb3DQEBAQUAA4GMADCBiAKBgHO4H2f...",
  "lamports_to_send": 1000000000
}
```

## Execute vault transaction

### Command name: `vault_transaction_execute`

### Schema example

```json
{
  "sender_pubkey": "MIGeMA0GCSqGSIb3DQEBAQUAA4GMADCBiAKBgHO4H2f...",
  "receiver_pubkey": "MIGeMA0GCSqGSIb3DQEBAQUAA4GMADCBiAKBgHO4H2f...",
  "lamports_to_send": 1000000000
}
```

# Requests

## Get multisig members

### Command name: `get_multisig_members`

### Answer schema example

```json
{
  "members": [
    {
      "pubkey": "MIGeMA0GCSqGSIb3DQEBAQUAA4GMADCBiAKBgHO4H2f...",
      "permissions": ["Propose", "Vote", "Execute"]
    },
    ...
  ]
}
```

## Get multisig threshold

### Command name: `get_threshold`

### Answer schema example

```json
{
  "threshold": 1
}
```

## Get current proposal status

### Command name: `get_current_proposal_status`

### Answer schema example

```json
{
  "proposal_status": "Approved"
}
```