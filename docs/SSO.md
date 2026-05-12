# SSO / OIDC

RWFW supports SSO through OpenID Connect Authorization Code flow with `state`, `nonce`, and PKCE.

## Add a Provider

```bash
rwfw auth sso add keycloak \
  --provider keycloak \
  --issuer-url http://localhost:8081/realms/rwfw \
  --client-id-env KEYCLOAK_CLIENT_ID \
  --client-secret-env KEYCLOAK_CLIENT_SECRET
```

Supported provider presets:

- `oidc`: generic standards-compliant OpenID Connect provider.
- `keycloak`: Keycloak issuer URL, for example `http://localhost:8081/realms/rwfw`.
- `azure-b2c`: Azure AD B2C issuer URL for a user flow or custom policy.

The command updates:

- `config/development.yaml`
- `config/production.yaml`
- `.env.example`

## Runtime Config

SSO provider config lives under `auth.oidc`:

```yaml
auth:
  session_ttl: 86400
  oidc:
    redirect_base_url: "http://localhost:3000"
    providers:
      keycloak:
        provider: "keycloak"
        display_name: "Keycloak"
        issuer_url: "http://localhost:8081/realms/rwfw"
        client_id_env: "KEYCLOAK_CLIENT_ID"
        client_secret_env: "KEYCLOAK_CLIENT_SECRET"
        scopes:
          - openid
          - profile
          - email
        require_verified_email: true
```

Client secrets are not stored in YAML. The runtime reads the env vars named by `client_id_env` and `client_secret_env`.

## Routes

- `GET /auth/sso/:provider/start`: creates an OIDC state record and redirects to the provider.
- `GET /auth/sso/:provider/callback`: validates the callback, links or creates the local user, creates an RWFW session cookie, and redirects to `/home`.

## User Policy

- Existing external identities log in to the linked local user.
- Existing local users are linked by email only when the provider returns a verified email.
- New local users are created automatically when the provider returns a valid verified email.
- The first created user still receives the `admin` role.
- Local email/password login remains enabled.
