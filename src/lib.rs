// TODO: JSON errors
// TODO: auth
// TODO: 404?

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use worker::{js_sys::encode_uri_component, *};

mod utils;

const SLUGS_KV: &str = "SLUGS";

#[derive(Clone, Debug, Deserialize, Serialize)]
struct StoredRedirect {
    pub url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
struct CreateRedirectPayload {
    pub slug: String,
    pub url: String,
}

#[event(fetch)]
pub async fn main(
    req: Request,
    env: Env,
    _ctx: worker::Context,
) -> Result<Response> {
    utils::set_panic_hook();

    let router = Router::new();

    router
        .get("/", |_, _| {
            Response::ok("ben's dumb url shortener thing lol")
        })
        .get_async("/:slug", |req, ctx| async move {
            if let Some(slug) = ctx.param("slug") {
                let kv = ctx.kv(SLUGS_KV)?;

                if let Ok(Some(redirect_to)) =
                    kv.get(slug).json::<StoredRedirect>().await
                {
                    console_log!(
                        "[slug={}] redirecting to: {}",
                        &slug,
                        &redirect_to.url
                    );

                    let url = Url::parse(&redirect_to.url)?;

                    return Response::redirect(url);
                }
            }

            // TODO: figure this out

            let mut u = req.url()?;
            u.set_path("/");

            return Response::redirect(u);
        })
        .get_async("/slugs", |_req, ctx| async move {
            // TODO: auth
            let kv = ctx.kv(SLUGS_KV)?;
            let slugs = kv.list().execute().await?;
            let keys = slugs
                .keys
                .iter()
                .map(|k| k.name.clone())
                .collect::<Vec<_>>();

            Response::from_json(&keys)
        })
        .delete_async("/slugs/:slug", |_req, ctx| async move {
            // TODO: auth

            if let Some(slug) = ctx.param("slug") {
                let kv = ctx.kv(SLUGS_KV)?;
                kv.delete(slug).await?;
            }

            Ok(Response::empty()?.with_status(204))
        })
        .post_async("/slugs", |mut req, ctx| async move {
            // TODO: auth

            match req.json::<CreateRedirectPayload>().await {
                Ok(body) => {
                    let reserved_slugs = ["slugs"];
                    if reserved_slugs.contains(&body.slug.as_str()) {
                        return Response::error(
                            "Slug is a reserved token",
                            400,
                        );
                    }

                    let slug = String::from(encode_uri_component(&body.slug));

                    if slug.len() == 0 {
                        return Response::error(
                            "Slug must be at least 1 character long",
                            400,
                        );
                    }

                    let to_store = StoredRedirect {
                        url: body.url,
                        created_at: Utc::now(),
                    };

                    let kv = ctx.kv(SLUGS_KV)?;
                    let response =
                        match kv.put(&slug, to_store)?.execute().await {
                            // is there a better way to do this
                            Ok(()) => Ok(Response::empty()?.with_status(204)),
                            Err(_) => {
                                Response::error("Internal Server Error", 500)
                            },
                        };

                    response
                },

                Err(_) => {
                    return Response::error("Bad Request", 400);
                },
            }
        })
        .run(req, env)
        .await
}
