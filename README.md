<h1 align="center">URL Shortener 🔗</h1>

A simple URL shortener that lives on a Cloudflare Worker.

## API Reference
### Create Shortlink:
```http
POST /slugs
Authorization: <api-key>

{"url": "https://pupy.gay", "slug": "owo"}
```
**Returns**: `HTTP 204`

### Delete Shortlink:
```http
DELETE /slugs/:slug
Authorization: <api-key>
```
**Returns**: `HTTP 204`

### Get Shortlink Info:
```http
GET /slugs/:slug
Authorization: <api-key>
```

**Returns**: `{"url": "<url>", "created_at": "<ISO-8601 date>", "user": "<user>"}`

### List All Shortlinks:
```http
GET /slugs
Authorization: <api-key>
```

**Returns**: `String[]`

## Running for Yourself
1. Create a secrets for generating API keys. The OpenSSL CLI works well for this (`openssl rand -hex 64`), but any 64 character long string should work.

2. Deploy this repo to your Cloudflare Worker account using the wrangler CLI

3. Add your secret as an application variable called `JWT_SECRET` for your worker

4. Generate an API key (MacOS/Linux only. Windows users can run this with WSL :P) `URL_SHORTENER_JWT_SECRET="<your-secret>"./scripts/urlshortener create-token <username>`. The username doesn't do anything other than show who created the shortlink if you look up a link with the CLI or API (outlined above)
