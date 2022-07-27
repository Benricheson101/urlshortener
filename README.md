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


### List All Shortlinks:
```http
GET /slugs
Authorization: <api-key>
```

**Returns**: `String[]`

## Creating an API Key
Instructions coming soon...
