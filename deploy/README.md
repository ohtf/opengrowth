# The demo server

opengrowth's public demo runs at <https://api.openhealthtechnology.org/growth/v1/>, on an OHTF server at Hetzner in Helsinki (`R58`). Caddy terminates TLS and adds the `/growth/v1` prefix that the RCPCH API uses, in front of opengrowth itself. The server holds no data: it calculates and forgets.

It is open to anyone, and pre-alpha: not for clinical use.

## What is here

| File | What it is |
| --- | --- |
| `compose.yaml` | The stack: opengrowth, pinned to the commit it was built from, behind Caddy, pinned by digest. Both run read-only and without capabilities, except the one Caddy needs to listen on ports 80 and 443. |
| `Caddyfile` | TLS, the `/growth/v1` prefix, compression, and the fix for opengrowth's trailing-slash redirects behind a prefix. |

## Deploying

Deploy the current commit, once it is pushed:

```sh
s/deploy
```

or run the `deploy` workflow in GitHub Actions, which deploys `main`. Either one builds the image, checks it with `s/check-container`, copies it to the server over SSH, restarts the stack, and waits until the public address answers with the new commit.

To roll back, check out an earlier commit and deploy it. The server keeps the images of its last three deployments.

## Looking at it

```sh
ssh -p 2227 deploy@api.openhealthtechnology.org
cd /srv/opengrowth
docker compose ps
docker compose logs --tail 100
```

opengrowth logs one line when it starts, and Caddy keeps no access log, so no request is recorded.

## Setting up a server

Any Ubuntu server will do, at Hetzner or elsewhere, if it accepts connections on ports 80 and 443, and SSH on port 2227 or the port in `OPENGROWTH_DEPLOY_PORT`.

1. Have an account on it that can `sudo` without a password, reachable with your own key.
2. Make a deploy key for the workflow, in a private directory: `ssh-keygen -t ed25519 -N "" -C opengrowth-deploy -f DIR/opengrowth-deploy`.
3. Prepare the server: `s/setup-server ADMIN_DESTINATION YOUR_KEY.pub DIR/opengrowth-deploy.pub`. It installs Docker if the server has none, makes the `deploy` user with those two keys, adds the server's host key to your `known_hosts`, and prints the workflow's `DEPLOY_KNOWN_HOSTS` line. The deploy user can run any container, which gives it control of the whole server, so keep its keys to those who deploy.
4. Point `api.openhealthtechnology.org` at the server with A and AAAA records that are not proxied (in Cloudflare, "DNS only"), so that Caddy can obtain its certificate.
5. Give the workflow its secrets, then delete the deploy key's private half, which nothing else needs:

   ```sh
   gh secret set DEPLOY_SSH_KEY --env demo < DIR/opengrowth-deploy
   gh secret set DEPLOY_KNOWN_HOSTS --env demo --body "LINE PRINTED BY s/setup-server"
   ```

   The `demo` environment admits `main` only.
6. Once the records resolve, deploy with `s/deploy`; Caddy obtains its certificate as it starts. For SSH to use your key as the deploy user, give the host an entry in `~/.ssh/config` with `User deploy`, `Port 2227`, and your `IdentityFile`.
