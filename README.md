# ash_meet_bot

Google meets insert bot, originally written for [`ash`](https://t.me/wowjerry)

## Prepare

Go to [Google API Console](https://console.cloud.google.com/apis/credentials), apply for an Oauth 2.0 API Secret

### Desktop App
- Simple
- Does not need you follow Google's Redirect URI policy
     
for this case you may want to use a port forward tool (local -> remote)

e.g. with ssh tunnel
```bash
ssh -p 10322 -L 11451:localhost:11451 poly@神秘小机子
```
### Web App

- Redirect URI: HTTPS(CA / CloudFlare); Privacy Policy; ...

## Place secret

Download `client_secret_xxxxxxxxxxxxxxxxxxxxxxxxxxxx.json`, rename it to `client_secret.json`

## Build

```bash
cargo build --release --target-dir target
```

## Run

```bash
export TELOXIDE_TOKEN=XXXXXXXXXX:XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
./target/release/ash_meet_bot
```
