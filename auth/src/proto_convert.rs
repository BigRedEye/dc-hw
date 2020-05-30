use crate::models;

// FIXME(BigRedEye) CODEGEN THIS SHIT
// Simple proc macro

pub fn parse_role(role: i32) -> models::AccessLevel {
    match role {
        0 => models::AccessLevel::User,
        1 => models::AccessLevel::Admin,
        _ => unreachable!(),
    }
}

impl From<pb::RegisterRequest> for models::NewUser {
    fn from(req: pb::RegisterRequest) -> models::NewUser {
        return models::NewUser {
            password: req.password,
            phone: req.phone,
            email: req.email,
            permissions: models::AccessLevel::User,
        }
    }
}

impl From<pb::LoginRequest> for models::LoginRequest {
    fn from(req: pb::LoginRequest) -> models::LoginRequest {
        return models::LoginRequest {
            login: req.login,
            password: req.password,
        }
    }
}

impl From<models::LoginResponse> for pb::LoginResponse {
    fn from(rsp: models::LoginResponse) -> pb::LoginResponse {
        return pb::LoginResponse {
            tokens: pb::Tokens{
                access: rsp.access_token,
                refresh: rsp.refresh_token,
            }
        }
    }
}

impl From<models::LoginResponse> for pb::RefreshResponse {
    fn from(rsp: models::LoginResponse) -> pb::RefreshResponse {
        return pb::RefreshResponse {
            tokens: pb::Tokens{
                access: rsp.access_token,
                refresh: rsp.refresh_token,
            }
        }
    }
}

impl From<pb::UpdateUserRequest> for models::UpdateUserRequest {
    fn from(req: pb::UpdateUserRequest) -> models::UpdateUserRequest {
        return models::UpdateUserRequest {
            user_id: req.id,
            role: parse_role(req.role.into()),
        }
    }
}

impl From<models::ValidateTokenResponse> for pb::ValidateTokenResponse {
    fn from(rsp: models::ValidateTokenResponse) -> pb::ValidateTokenResponse {
        return pb::ValidateTokenResponse {
            valid: rsp.valid,
            role: match rsp.role {
                models::AccessLevel::User => 0,
                models::AccessLevel::Admin => 1,
            }
        }
    }
}

impl From<models::ListUsersResponse> for pb::ListUsersResponse {
    fn from(rsp: models::ListUsersResponse) -> pb::ListUsersResponse {
        return pb::ListUsersResponse {
            users: rsp.users.into_iter().map(|user| user.into()).collect()
        }
    }
}

impl From<models::User> for pb::UserInfo {
    fn from(user: models::User) -> pb::UserInfo {
        return pb::UserInfo {
            id: user.id,
            role: match user.permissions {
                models::AccessLevel::User => pb::Role::User.into(),
                models::AccessLevel::Admin => pb::Role::Admin.into(),
            },
            email: user.email,
            phone: user.phone,
        }
    }
}

impl From<pb::ListUsersRequest> for models::ListUsersRequest {
    fn from(req: pb::ListUsersRequest) -> models::ListUsersRequest {
        return models::ListUsersRequest {
            offset: req.offset,
            limit: req.limit,
        }
    }
}
