use auth::JWTClaims;
use chrono::{DateTime, Utc};
use jwt_compact::{
    alg::{Hs256, Hs256Key},
    AlgorithmExt,
    Claims,
    Header,
    Token,
    UntrustedToken,
};
use serde::{Deserialize, Serialize};
use worker::{js_sys::encode_uri_component, *};

mod auth;
mod utils;

const SLUGS_KV: &str = "SLUGS";

#[derive(Clone, Debug, Deserialize, Serialize)]
struct StoredRedirect {
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub user: String,
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
        .get("/", |_, ctx| {
            let claims = JWTClaims {
                user: "ben".into(),
            };

            let h = Header::default();
            let c = Claims::new(claims);
            let k = Hs256Key::new(ctx.secret("JWT_SECRET").unwrap().to_string().as_bytes());

            let tkn = Hs256.token(h, &c, &k).unwrap();
            console_log!("token = {}", tkn);

            let t = UntrustedToken::new("eyJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjoiYmVuIn0.P__gYKgAMyoqNNdR4FnpyttnadpqAJhrMBDFAxFXf18").unwrap();
            let t: Token<JWTClaims> = Hs256.validate_integrity(&t, &k).unwrap();

            console_log!("{:?}", t.claims());

            Response::ok("ben's dumb url shortener thing lol")
        })
        .get_async("/:slug", |_req, ctx| async move {
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

            return Response::error("Not Found", 404);
        })
        .get_async("/slugs", |req, ctx| async move {
            verify_jwt!(req, ctx);

            let kv = ctx.kv(SLUGS_KV)?;
            let slugs = kv.list().execute().await?;
            let keys = slugs
                .keys
                .iter()
                .map(|k| k.name.clone())
                .collect::<Vec<_>>();

            Response::from_json(&keys)
        })
        .delete_async("/slugs/:slug", |req, ctx| async move {
            verify_jwt!(req, ctx);

            if let Some(slug) = ctx.param("slug") {
                let kv = ctx.kv(SLUGS_KV)?;
                kv.delete(slug).await?;
            }

            Ok(Response::empty()?.with_status(204))
        })
        .post_async("/slugs", |mut req, ctx| async move {
            let token = verify_jwt!(req, ctx);
            let claims = token.claims();

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
                        user: claims.custom.user.clone(),
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
