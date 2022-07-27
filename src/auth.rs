use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JWTClaims {
    pub user: String,
}

#[macro_export]
macro_rules! verify_jwt {
    ($req:expr, $ctx:expr) => {{
        use jwt_compact::{
            alg::{Hs256, Hs256Key},
            AlgorithmExt,
            Token,
            UntrustedToken,
        };

        use crate::auth::JWTClaims;

        let token: Token<JWTClaims> = match $req
            .headers()
            .get("authorization")?
        {
            Some(auth_header) => match $ctx.secret("JWT_SECRET") {
                Ok(secret) => {
                    let key = Hs256Key::new(secret.to_string().as_bytes());
                    match UntrustedToken::new(&auth_header) {
                        Ok(untrusted_token) => {
                            match Hs256
                                .validate_integrity(&untrusted_token, &key)
                            {
                                Ok(token) => token,
                                Err(err) => {
                                    console_error!("failed to validate token integrity: {}", err);
                                    return Response::error(
                                        "Unauthorized",
                                        401,
                                    );
                                },
                            }
                        },
                        Err(err) => {
                            console_error!("Failed to parse token: {}", err);
                            return Response::error(
                                "Internal Server Error",
                                500,
                            );
                        },
                    }
                },

                Err(_) => {
                    console_error!("Missing `JWT_SECRET` secret");
                    return Response::error("Internal Server Error", 500);
                },
            },

            None => {
                return Response::error("Unauthorized", 401);
            },
        };

        token
    }};
}
