# The demo server

opengrowth's public demo runs at <https://api.openhealthtechnology.org/growth/v1/>, on an OHTF server at Hetzner in Helsinki (`R58`). Caddy terminates TLS, under the `/growth/v1` prefix that the RCPCH API uses. Behind it, APISIX strips the prefix and limits how many requests each client makes, in front of opengrowth itself. The server holds no data: it calculates and forgets.

It is open to anyone, and pre-alpha: not for clinical use.

## What is here

| File | What it is |
| --- | --- |
| `compose.yaml` | The stack: opengrowth, pinned to the commit it was built from, behind APISIX and Caddy, pinned by digest. All three run without capabilities, except the one Caddy needs to listen on ports 80 and 443; opengrowth and Caddy also run read-only. |
| `Caddyfile` | TLS, compression, and the fix for opengrowth's trailing-slash redirects behind a prefix. |
| `apisix-config.yaml` | APISIX in standalone mode, with no etcd and no Admin API, keeping no access log. |
| `apisix-routes.yaml` | The route to opengrowth, which strips the prefix, and the rate limit. |

## Deploying

Deploy the current commit, once it is pushed:

```sh
s/deploy
```

or run the `deploy` workflow in GitHub Actions, which deploys `main`. Either one builds the image, checks it with `s/check-container`, copies it to the server over SSH, restarts the stack, and waits until the public address answers with the new commit.

To roll back, check out an earlier commit and deploy it. The server keeps the images of its last three deployments.

## Rate limits

Each client address can make 600 requests a minute, counted over a sliding window. Beyond that, APISIX answers 429 with a JSON message, until the rate falls. Caddy tells APISIX the client's address, so a client cannot claim another one.

To change the limit, edit `limit-count` in `apisix-routes.yaml` and deploy. API keys, and quotas for each key, would be APISIX consumers in the same file, with the keys themselves kept out of the repository.

## Looking at it

```sh
ssh -p 2227 deploy@api.openhealthtechnology.org
cd /srv/opengrowth
docker compose ps
docker compose logs --tail 100
```

opengrowth logs one line when it starts. Neither Caddy nor APISIX keeps an access log, or logs the requests the rate limit refuses, so the requests the server receives leave no record. Only a request that fails on the server, for example while a deploy restarts opengrowth, can leave a line in Caddy's or APISIX's error log, naming the client's address and the path, never what the request contained.

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
