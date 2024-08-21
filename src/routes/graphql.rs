use askama_axum::IntoResponse;
use async_graphql::{
    http::GraphiQLSource, Context, EmptyMutation, EmptySubscription, Guard, Object, Schema, SimpleObject
};
use async_graphql_axum::{GraphQL, GraphQLSubscription};
use axum::{response, routing::get, Router};

use crate::{db, models::Advert, AppState};


#[derive(Eq, PartialEq, Clone, Copy)]
enum Role {
    Admin,
    User,
    Guest,
}


struct RoleGuard {
    role: Role
}

impl RoleGuard {
    fn new(role: Role) -> Self {
        Self { role }
    }
}

impl Guard for RoleGuard {
    async fn check(&self, ctx: &Context<'_>) -> async_graphql::Result<()> {
        let permission_repo = ctx.data_opt();
    }
}


#[derive(SimpleObject)]
struct PublicAdvert {
    pub id: i64,
    pub title: String,
    pub content: String,
}

impl Into<PublicAdvert> for &Advert {
    fn into(self) -> PublicAdvert {
        PublicAdvert {
            id: self.id,
            title: self.title.clone(),
            content: self.content.clone()
        }
    }
}

struct Query;

#[Object]
impl Query {
    async fn public_posts(&self, context: &Context<'_>, page: Option<i64>) -> Vec<PublicAdvert> {
        let state = context.data_unchecked::<AppState>();
        let db = state.db.read().await;
        let (adverts, total) = db::get_main_page(&db, 10, page.map(|page| (page - 1) * 10).unwrap_or(0)).await.unwrap_or((vec![], 0));
        adverts.iter().map(|a|a.into()).collect()
    }
}

async fn graphiql() -> impl IntoResponse {
    response::Html(
        GraphiQLSource::build()
            .endpoint("/gql")
            .subscription_endpoint("/gql/ws")
            .finish(),
    )
}

fn build_schema(state: AppState) -> Schema<Query, EmptyMutation, EmptySubscription> {
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(state)
        .finish();

    schema
}

pub fn graphql_router(state: AppState) -> Router<AppState> {
    let schema = build_schema(state);

    Router::new().route(
        "/",
        get(graphiql).post_service(GraphQL::new(schema.clone())),
    )
    .route_service("/ws", GraphQLSubscription::new(schema))
}
