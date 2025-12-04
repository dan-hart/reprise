# Security Policy

## Credential Management

This project stores API tokens in `~/.reprise/config.toml`, which is outside the repository and never committed to version control.

**Never commit:**
- API keys, tokens, or passwords
- `.env` files with real credentials
- Private keys (`.key`, `.pem`, `.p12`)
- Database connection strings with credentials

## Pre-Commit Protection

This repository uses [git-secrets](https://github.com/awslabs/git-secrets) to prevent accidental credential commits.

### Setup for Contributors

```bash
# Install git-secrets
brew install git-secrets

# Initialize in your clone (hooks are not cloned)
git secrets --install

# Register AWS patterns
git secrets --register-aws
```

### Verification

```bash
# Scan staged files
git secrets --scan

# Scan entire history
git secrets --scan-history

# List active patterns
git secrets --list
```

## Pre-Commit Checklist

Before every commit, verify:

- [ ] No API keys, tokens, or passwords in code
- [ ] No hardcoded URLs with credentials
- [ ] No `.env` files being committed
- [ ] `git secrets --scan` passes

## Reporting Security Vulnerabilities

If you discover a security vulnerability, please report it responsibly:

1. **Do not** open a public issue
2. Email the maintainer directly with details
3. Allow reasonable time for a fix before disclosure

## Security Patterns

The following are blocked by git-secrets:

- AWS credentials (access keys, secret keys, account IDs)
- OpenAI API keys (`sk-`, `sk-proj-`)
- GitHub tokens (`ghp_`, `gho_`, `ghs_`, `github_pat_`)
- Atlassian/Jira tokens (`ATATT`)
- Google API keys (`AIza`)
- Slack tokens (`xox[baprs]`)
- Private key headers
- Generic API keys and secrets
- Database/Redis URLs
- Bearer tokens
- Password assignments
- Bitrise API tokens and secrets (`BITRISE_API_TOKEN`, `BITRISE_SECRET`)
- Bitrise token URLs (`bitrise.io.*token`)
