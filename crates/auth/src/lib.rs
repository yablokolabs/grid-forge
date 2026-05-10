use chrono::{Duration, Utc};
use grid_forge_common::{AppError, AppResult, AuthConfig};
use grid_forge_domain::{OrganizationId, Permission, Role, User, UserId};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: UserId,
    pub org: OrganizationId,
    pub email: String,
    pub role: Role,
    pub exp: usize,
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: UserId,
    pub organization_id: OrganizationId,
    pub email: String,
    pub role: Role,
}

impl AuthContext {
    pub fn require(&self, permission: Permission) -> AppResult<()> {
        if self.role.has_permission(permission) {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!("{permission:?}")))
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthService {
    config: AuthConfig,
}

impl AuthService {
    pub fn new(config: AuthConfig) -> Self {
        Self { config }
    }

    pub fn login_demo(&self, email: &str, password: &str) -> AppResult<(User, String)> {
        if !self.config.demo_mode {
            return Err(AppError::Unauthorized);
        }
        if password != "demo-password" {
            return Err(AppError::Unauthorized);
        }

        let role = match email {
            "engineer@cedar-rapids.example" => Role::UtilityEngineer,
            "regulatory@cedar-rapids.example" => Role::RegulatoryAnalyst,
            "customerops@cedar-rapids.example" => Role::CustomerOps,
            "opsmanager@cedar-rapids.example" => Role::OpsManager,
            "auditor@cedar-rapids.example" => Role::Auditor,
            "admin@cedar-rapids.example" => Role::Admin,
            _ => return Err(AppError::Unauthorized),
        };

        let org_id = demo_organization_id();
        let user = User {
            id: stable_user_id(email),
            organization_id: org_id,
            email: email.to_string(),
            display_name: email.split('@').next().unwrap_or("demo").replace('.', " "),
            role,
            active: true,
            created_at: Utc::now(),
        };
        let token = self.issue_token(&user)?;
        Ok((user, token))
    }

    pub fn issue_token(&self, user: &User) -> AppResult<String> {
        let claims = Claims {
            sub: user.id,
            org: user.organization_id,
            email: user.email.clone(),
            role: user.role,
            exp: (Utc::now() + Duration::hours(8)).timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|err| AppError::Internal(format!("failed to issue token: {err}")))
    }

    pub fn verify_bearer(&self, header_value: Option<&str>) -> AppResult<AuthContext> {
        let token = header_value
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized)?;

        Ok(AuthContext {
            user_id: data.claims.sub,
            organization_id: data.claims.org,
            email: data.claims.email,
            role: data.claims.role,
        })
    }
}

pub fn demo_organization_id() -> OrganizationId {
    Uuid::parse_str("aaaaaaaa-aaaa-4aaa-aaaa-aaaaaaaaaaaa").expect("valid demo org UUID")
}

fn stable_user_id(email: &str) -> UserId {
    match email {
        "engineer@cedar-rapids.example" => Uuid::parse_str("11111111-1111-4111-8111-111111111111"),
        "regulatory@cedar-rapids.example" => {
            Uuid::parse_str("22222222-2222-4222-8222-222222222222")
        }
        "customerops@cedar-rapids.example" => {
            Uuid::parse_str("33333333-3333-4333-8333-333333333333")
        }
        "opsmanager@cedar-rapids.example" => {
            Uuid::parse_str("44444444-4444-4444-8444-444444444444")
        }
        "auditor@cedar-rapids.example" => Uuid::parse_str("55555555-5555-4555-8555-555555555555"),
        "admin@cedar-rapids.example" => Uuid::parse_str("66666666-6666-4666-8666-666666666666"),
        _ => Uuid::parse_str("77777777-7777-4777-8777-777777777777"),
    }
    .expect("valid demo user UUID")
}

#[cfg(test)]
mod tests {
    use super::*;
    use grid_forge_common::AuthConfig;

    #[test]
    fn demo_login_issues_verifiable_jwt() {
        let auth = AuthService::new(AuthConfig {
            jwt_secret: "test-secret".to_string(),
            demo_mode: true,
        });
        let (_user, token) = auth
            .login_demo("engineer@cedar-rapids.example", "demo-password")
            .expect("demo login");
        let context = auth
            .verify_bearer(Some(&format!("Bearer {token}")))
            .expect("valid token");
        assert_eq!(context.role, Role::UtilityEngineer);
        assert!(context.require(Permission::RunEngineeringCopilot).is_ok());
        assert!(context.require(Permission::RunRegulatoryCopilot).is_err());
    }
}
