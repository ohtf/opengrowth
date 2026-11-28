# The demo server

opengrowth's public demo runs at <https://api.openhealthtechnology.org/growth/v1/>, on a Hetzner Cloud server in Helsinki (`R58`). Caddy terminates TLS and adds the `/growth/v1` prefix that the RCPCH API uses, in front of opengrowth itself. The server holds no data: it calculates and forgets.

It is open to anyone, and pre-alpha: not for clinical use.

## What is here

| File | What it is |
| --- | --- |
| `compose.yaml` | The stack: opengrowth, pinned to the commit it was built from, behind Caddy, pinned by digest. Both run read-only and without capabilities, except the one Caddy needs to listen on ports 80 and 443. |
| `Caddyfile` | TLS, the `/growth/v1` prefix, compression, and the fix for opengrowth's trailing-slash redirects behind a prefix. |
| `cloud-init.yaml` | The server's first-boot setup: Docker and Compose from the Ubuntu archive, a `deploy` user, SSH by key only, and automatic security updates, with a restart at 04:00 UTC when one needs it. |

## Deploying

Deploy the current commit, once it is pushed:

```sh
s/deploy
```

or run the `deploy` workflow in GitHub Actions, which deploys `main`. Either one builds the image, checks it with `s/check-container`, copies it to the server over SSH, restarts the stack, and waits until the public address answers with the new commit.

To roll back, check out an earlier commit and deploy it. The server keeps the images of its last three deployments.

## Looking at it

```sh
ssh deploy@api.openhealthtechnology.org
cd /srv/opengrowth
docker compose ps
docker compose logs --tail 100
```

opengrowth logs one line when it starts, and Caddy keeps no access log, so no request is recorded.

## Rebuilding the server

The server can be rebuilt from this directory and two scripts:

1. Create an hcloud context for the OHTF project, pasting its API token when asked: `hcloud context create ohtf`.
2. Have an admin key, with its public half at `~/.ssh/api.openhealthtechnology.org.pub` and its private half on disk beside it or in your SSH agent. Make a new deploy key in a private directory, for the workflow alone: `ssh-keygen -t ed25519 -N "" -C opengrowth-deploy -f DIR/opengrowth-deploy`.
3. Create the server: `s/create-server ~/.ssh/api.openhealthtechnology.org.pub DIR/opengrowth-deploy.pub`. It adds the server's host key to your `known_hosts` and prints the DNS records to create and the workflow's `DEPLOY_KNOWN_HOSTS` line.
4. Point `api.openhealthtechnology.org` at the new addresses, with the records not proxied (in Cloudflare, "DNS only"), so that Caddy can obtain its certificate.
5. Give the workflow its secrets, then delete the deploy key's private half, which nothing else needs:

   ```sh
   gh secret set DEPLOY_SSH_KEY --env demo < DIR/opengrowth-deploy
   gh secret set DEPLOY_KNOWN_HOSTS --env demo --body "LINE PRINTED BY s/create-server"
   ```

   The `demo` environment admits `main` only.
6. Deploy: `s/deploy`. For SSH to use the admin key, give the host an entry in `~/.ssh/config`, with `User deploy` and `IdentityFile ~/.ssh/api.openhealthtechnology.org.pub`.

Caddy obtains its certificate on the first request after the DNS records resolve.
